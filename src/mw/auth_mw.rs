use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{Algorithm, DecodingKey, TokenData, Validation, decode, decode_header};
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::{
    auth::models::{Claims, SubjectId},
    config::config::CONFIG,
    integration::models::IntegrationName,
    server::{
        app_state::{AppState, Jwks},
        error::ServerError,
    },
};

static AUTH0_WEBHOOK_KEY: &str = "Auth0-Webhook-Key";

pub async fn auth_mw(
    State(state): State<Arc<AppState>>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, ServerError> {
    let auth_header = extract_header(AUTHORIZATION.as_str(), req.headers());
    let auth0_webhook_key = extract_header(&AUTH0_WEBHOOK_KEY, req.headers());

    if auth_header.is_none() && auth0_webhook_key.is_none() {
        error!("Missing authentication headers");
        return Err(ServerError::Api(
            StatusCode::UNAUTHORIZED,
            "Missing authorization header".into(),
        ));
    }

    if auth_header.is_some() {
        let (subject, permissions) =
            get_subject_and_permissions(auth_header.unwrap(), state.get_jwks()).await?;

        debug!("Request by subject: {:?}", &subject);
        req.extensions_mut().insert(permissions);
        req.extensions_mut().insert(subject);

        return Ok(next.run(req).await);
    }

    if auth0_webhook_key.is_some() {
        let current_key = auth0_webhook_key.unwrap();
        let valid_key = CONFIG.auth0.webhook_key.to_string();

        if current_key != valid_key {
            return Err(ServerError::Api(StatusCode::UNAUTHORIZED, "".into()));
        }

        let subject = SubjectId::Integration(IntegrationName::Auth0);
        info!("Request by subject: {:?}", subject);
        req.extensions_mut().insert(subject);
    }

    Ok(next.run(req).await)
}

async fn get_subject_and_permissions(
    header_value: String,
    jwks: &Jwks,
) -> Result<(SubjectId, Claims), ServerError> {
    if let Some(token) = header_value.strip_prefix("Bearer ") {
        debug!("Recieved token: {}", token);

        let token_data = verify_jwt(token, jwks).await?;
        let claims: Claims = serde_json::from_value(token_data.claims)?;
        let subject = SubjectId::Registered(claims.sub.clone());

        return Ok((subject, claims));
    }

    if let Some(value) = header_value.strip_prefix("Guest ") {
        let id: Uuid = value.parse().map_err(|_| {
            ServerError::Api(StatusCode::BAD_REQUEST, "Failed to parse header".into())
        })?;
        return Ok((SubjectId::Guest(id), Claims::empty()));
    }

    Err(ServerError::Api(
        StatusCode::UNAUTHORIZED,
        "Not allowed".into(),
    ))
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
