use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{Algorithm, DecodingKey, TokenData, Validation, decode, decode_header};
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    auth::models::{Claims, SubjectId},
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
        // Unauthorized
        (None, None, None) => {
            error!("Missing authentication method");
            return Err(ServerError::Api(
                StatusCode::UNAUTHORIZED,
                "Missing authorization header".into(),
            ));
        }
        // Auth0
        (None, None, Some(webhook_key)) => {
            if webhook_key != CONFIG.auth0.webhook_key {
                return Err(ServerError::Api(
                    StatusCode::UNAUTHORIZED,
                    "Invalid webhook key".into(),
                ));
            }

            let subject = SubjectId::Integration(IntegrationName::Auth0);
            info!("Request by subject: {:?}", subject);
            req.extensions_mut().insert(subject);
        }
        // Auth0 User
        (Some(guest_header), Some(token_header), None) => {
            let Some(token) = token_header.strip_prefix("Bearer ") else {
                return Err(ServerError::Api(
                    StatusCode::UNAUTHORIZED,
                    "Missing auth token".into(),
                ));
            };

            let token_data = verify_jwt(token, state.get_jwks()).await?;
            let claims: Claims = serde_json::from_value(token_data.claims)?;
            let subject = SubjectId::Registered(claims.sub.clone());
            info!("Request by subject: {:?}", subject);
            req.extensions_mut().insert(subject);
            req.extensions_mut().insert(claims);

            // TODO - get user from db, and maybe sync users
        }
        // Guest user
        (Some(guest_header), None, None) => {
            let uuid = to_uuid(guest_header)?;

            let subject = SubjectId::Guest(uuid);
            info!("Request by subject: {:?}", subject);
            req.extensions_mut().insert(subject);
            req.extensions_mut().insert(Claims::empty());
        }
        // Unauthorized error
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
                .file_name("auth_mw.rs")
                .log_async();

            return Err(ServerError::Api(
                StatusCode::UNAUTHORIZED,
                "Wierd error, please contact".into(),
            ));
        }
    };

    Ok(next.run(req).await)
}

fn handle_webhook() -> Result<(), ServerError> {
    //
}

fn handle_guest() -> Result<(), ServerError> {
    //
}

fn handle_token() -> Result<(), ServerError> {
    //
}

fn handle_edge_case() -> Result<(), ServerError> {
    //
}

fn to_uuid(value: String) -> Result<Uuid, ServerError> {
    value.parse().map_err(|_| {
        ServerError::Api(
            StatusCode::UNAUTHORIZED,
            "Guest id is invalid format".into(),
        )
    })
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
