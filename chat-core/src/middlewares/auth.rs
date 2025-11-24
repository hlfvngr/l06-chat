use axum::{
    extract::{FromRequestParts, Query, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use serde::Deserialize;
use tracing::{info, warn};

use crate::TokenVerify;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Params {
    pub token: String,
}

pub async fn verify_token<T>(State(state): State<T>, request: Request, next: Next) -> Response
where
    T: TokenVerify + Send + Sync + 'static,
{
    let (mut parts, body) = request.into_parts();
    let token =
        match TypedHeader::<Authorization<Bearer>>::from_request_parts(&mut parts, &state).await {
            Ok(TypedHeader(Authorization(b))) => b.token().to_string(),
            Err(e) => {
                if e.is_missing() {
                    match Query::<Params>::from_request_parts(&mut parts, &state).await {
                        Ok(Query(params)) => {
                            info!("get token from query params:{}", params.token);
                            params.token.clone()
                        }
                        Err(e) => {
                            warn!("parse query params failed: {}", e);
                            let msg = format!("parse query params token failed: {}", e);
                            warn!(msg);
                            return (StatusCode::UNAUTHORIZED, msg).into_response();
                        }
                    }
                } else {
                    let msg = format!("parse authorization header failed: {}", e);
                    warn!(msg);
                    return (StatusCode::UNAUTHORIZED, msg).into_response();
                }
            }
        };

    let req = match state.verify_token(&token) {
        Ok(u) => {
            let mut request = Request::from_parts(parts, body);
            request.extensions_mut().insert(u);
            request
        }
        Err(e) => {
            let msg = format!("verify token failed: {}", e);
            warn!(msg);
            return (StatusCode::UNAUTHORIZED, msg).into_response();
        }
    };

    next.run(req).await
}
