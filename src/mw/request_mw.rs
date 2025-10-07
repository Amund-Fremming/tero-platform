use axum::{
    body::Body,
    http::{Request, Response},
    middleware::Next,
};
use tracing::debug;

#[allow(dead_code)]
pub async fn request_mw(req: Request<Body>, next: Next) -> Response<Body> {
    let method = req.method().to_string();
    let uri = req.uri().to_string();
    debug!("Incomming request: method={} uri={}", method, uri);

    let res = next.run(req).await;

    debug!("Outgoing request: method={} uri={}", method, uri);
    return res;
}
