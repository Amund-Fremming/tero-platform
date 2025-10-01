use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{Algorithm, DecodingKey, TokenData, Validation, decode, decode_header};
use sqlx::{Pool, Postgres};
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    auth::{
        db::{get_user_id_from_auth0_id, get_user_id_from_guest_id},
        models::{Claims, SubjectId},
    },
    config::config::CONFIG,
    integration::models::IntegrationName,
    server::{
        app_state::{AppState, Jwks},
        error::ServerError,
    },
    system_log::models::LogCeverity,
};

static AUTH0_WEBHOOK_KEY: &str = "Auth0-Webhook-Key";
static GUEST_AUTHORIZATION: &str = "X-Guest-Authentication";

pub async fn auth_mw(
    State(state): State<Arc<AppState>>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, ServerError> {
    let guest_header = extract_header(AUTHORIZATION.as_str(), req.headers());
    let token_header = extract_header(GUEST_AUTHORIZATION, req.headers());
    let auth0_webhook_key = extract_header(&AUTH0_WEBHOOK_KEY, req.headers());

    match (guest_header, token_header, auth0_webhook_key) {
        (None, None, None) => {
            error!("Missing authentication method");
            return Err(ServerError::Api(
                StatusCode::UNAUTHORIZED,
                "Missing authorization header".into(),
            ));
        }
        (None, None, Some(webhook_key)) => handle_webhook(&mut req, &webhook_key)?,
        (Some(guest_header), Some(token_header), None) => {
            handle_token_user(state.clone(), &mut req, &token_header, &guest_header).await?;
        }
        (Some(guest_header), None, None) => {
            handle_guest_user(state.get_pool(), &mut req, &guest_header).await?;
        }
        (o1, o2, o3) => {
            let error_msg = format!(
                "Wierd error, might be because token is present and guest id is not. Values: {:?}, {:?}, {:?}.",
                o1, o2, o3
            );

            error!(error_msg);
            state
                .audit()
                .ceverity(LogCeverity::Critical)
                .description(&error_msg)
                .function_name("auth_mw")
                .log_async();

            return Err(ServerError::Api(
                StatusCode::UNAUTHORIZED,
                "Wierd error, please contact".into(),
            ));
        }
    };

    Ok(next.run(req).await)
}

fn handle_webhook(request: &mut Request<Body>, webhook_key: &str) -> Result<(), ServerError> {
    if webhook_key != CONFIG.auth0.webhook_key {
        return Err(ServerError::Api(
            StatusCode::UNAUTHORIZED,
            "Invalid webhook key".into(),
        ));
    }

    let subject = SubjectId::Integration(IntegrationName::Auth0);
    info!("Request by subject: {:?}", subject);
    request.extensions_mut().insert(subject);

    Ok(())
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

async fn handle_token_user(
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

    let auth0_id = claims.sub.clone();
    let guest_id = to_uuid(guest_header)?;
    let user = get_user_id_from_auth0_id(state.get_pool(), &auth0_id).await?;
    
    if user
    /*
    TODO
    - if user has not guest_id set, but guest_id exists => async sync users
    - if user has both set, inject id
     */

    let user_id = get_user_id_from_auth0_id(state.get_pool(), &auth0_id).await?;
    let subject = SubjectId::Registered(user_id);
    info!("Request by subject: {:?}", subject);

    request.extensions_mut().insert(claims);
    request.extensions_mut().insert(subject);

    Ok(())
}

fn to_uuid(value: &str) -> Result<Uuid, ServerError> {
    let Ok(guest_id) = value.parse() else {
        return Err(ServerError::Api(
            StatusCode::UNAUTHORIZED,
            "Guest id is invalid".into(),
        ));
    };
    Ok(guest_id)
}

fn extract_header(key: &str, header_map: &HeaderMap) -> Option<String> {
    header_map
        .get(key)
        .and_then(|header| header.to_str().ok())
        .map(|s| s.to_owned())
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
