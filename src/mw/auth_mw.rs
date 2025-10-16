use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Request, State},
    http::{StatusCode, header::AUTHORIZATION},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{Algorithm, DecodingKey, TokenData, Validation, decode, decode_header};
use sqlx::{Pool, Postgres};
use tracing::{error, info};

use crate::{
    auth::{
        db::{get_user_id_from_guest_id, get_user_keys_from_auth0_id},
        models::{Claims, Jwks, SubjectId},
    },
    common::{app_state::AppState, error::ServerError},
    config::config::CONFIG,
    integration::models::{INTEGRATION_NAMES, IntegrationName},
    mw::common::{extract_header, to_uuid},
};

static GUEST_AUTHORIZATION: &str = "X-Guest-Authentication";

pub async fn auth_mw(
    State(state): State<Arc<AppState>>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, ServerError> {
    let guest_header = extract_header(GUEST_AUTHORIZATION, req.headers());
    let token_header = extract_header(AUTHORIZATION.as_str(), req.headers());

    match (guest_header, token_header) {
        (Some(guest_header), None) => {
            handle_guest_user(state.get_pool(), &mut req, &guest_header).await?;
        }
        (Some(guest_header), Some(token_header)) => {
            handle_user_token(state.clone(), &mut req, &token_header, &guest_header).await?;
        }
        (None, Some(token_header)) => {
            handle_m2m_token(state.clone(), &mut req, &token_header).await?;
        }
        (None, None) => {
            error!("Unauthorized request");
            return Err(ServerError::AccessDenied);
        }
    };

    Ok(next.run(req).await)
}

async fn handle_guest_user(
    pool: &Pool<Postgres>,
    request: &mut Request<Body>,
    guest_header: &str,
) -> Result<(), ServerError> {
    let guest_id = to_uuid(guest_header)?;

    let Some(user_id) = get_user_id_from_guest_id(pool, &guest_id).await? else {
        return Err(ServerError::Api(
            StatusCode::UNAUTHORIZED,
            "User with guest id does not exist".into(),
        ));
    };

    let subject = SubjectId::Guest(user_id);
    info!("Request by subject: {:?}", subject);

    request.extensions_mut().insert(subject);
    request.extensions_mut().insert(Claims::empty());

    Ok(())
}

async fn handle_m2m_token(
    state: Arc<AppState>,
    request: &mut Request<Body>,
    token_header: &str,
) -> Result<(), ServerError> {
    let Some(token) = token_header.strip_prefix("Bearer ") else {
        return Err(ServerError::Api(
            StatusCode::UNAUTHORIZED,
            "Missing auth token".into(),
        ));
    };

    let token_data = verify_jwt(token, state.get_jwks()).await?;
    let claims: Claims = serde_json::from_value(token_data.claims)?;

    if !claims.is_machine() {
        error!("M2M token handler recieved a user token");
        return Err(ServerError::AccessDenied);
    }

    let option: Option<IntegrationName> =
        IntegrationName::from_subject(&claims.sub, &INTEGRATION_NAMES).await;

    let Some(int_name) = option else {
        error!("Unknown integration subject: {}", claims.sub);
        return Err(ServerError::AccessDenied);
    };

    let subject = SubjectId::Integration(int_name);
    info!("Request by integration subject: {:?}", subject);

    request.extensions_mut().insert(claims);
    request.extensions_mut().insert(subject);

    Ok(())
}

async fn handle_user_token(
    state: Arc<AppState>,
    request: &mut Request<Body>,
    token_header: &str,
    guest_header: &str,
) -> Result<(), ServerError> {
    let Some(token) = token_header.strip_prefix("Bearer ") else {
        return Err(ServerError::Api(
            StatusCode::UNAUTHORIZED,
            "Missing auth token".into(),
        ));
    };

    let token_data = verify_jwt(token, state.get_jwks()).await?;
    let claims: Claims = serde_json::from_value(token_data.claims)?;

    if claims.is_machine() {
        error!("User token handler recieved a m2m token");
        return Err(ServerError::AccessDenied);
    }

    let guest_id = to_uuid(guest_header)?;
    let user_keys = get_user_keys_from_auth0_id(state.get_pool(), claims.auth0_id()).await?;

    if Some(guest_id) != user_keys.guest_id {
        info!("Starting user sync for user id: {}", user_keys.user_id);
        // TODO TURN ON SYNC
        //state.sync_user(user_keys.user_id, guest_id);
    }

    let subject = SubjectId::Registered(user_keys.user_id);
    info!("Request by subject: {:?}", subject);

    request.extensions_mut().insert(claims);
    request.extensions_mut().insert(subject);

    return Ok(());
}

// Warning: 65% AI generated code
async fn verify_jwt(token: &str, jwks: &Jwks) -> Result<TokenData<serde_json::Value>, ServerError> {
    let header = decode_header(token)
        .map_err(|e| ServerError::JwtVerification(format!("Failed to decode header: {}", e)))?;

    let kid = header
        .kid
        .ok_or_else(|| ServerError::JwtVerification("Missing JWT kid".into()))?;

    let jwk = jwks
        .keys
        .iter()
        .find(|jwk| jwk.kid == kid)
        .ok_or_else(|| ServerError::JwtVerification("JWK is not well known".into()))?;

    let decoding_key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)
        .map_err(|e| ServerError::JwtVerification(format!("Failed to get decoding key: {}", e)))?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[&CONFIG.auth0.audience]);
    validation.set_issuer(&[&CONFIG.auth0.domain]);

    decode::<serde_json::Value>(token, &decoding_key, &validation)
        .map_err(|e| ServerError::JwtVerification(format!("Failed to validate token: {}", e)))
}
