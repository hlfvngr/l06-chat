use axum::{middleware::from_fn, Router};
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::Level;

use crate::middlewares::{request_id::set_request_id, server_time::ServerTimeLayer};

pub(crate) mod request_id;
pub(crate) mod server_time;

const REQUEST_ID_HEADER: &str = "X-Request-Id";
const SERVER_TIME_HEADER: &str = "X-Server-Time";

pub(crate) fn set_common_layer(app: Router) -> Router {
    app.layer(
        ServiceBuilder::new()
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::new().include_headers(true))
                    .on_request(DefaultOnRequest::new().level(Level::INFO))
                    .on_response(
                        DefaultOnResponse::new()
                            .level(Level::INFO)
                            .latency_unit(LatencyUnit::Micros),
                    ),
            )
            .layer(CompressionLayer::new().br(true).deflate(true).gzip(true))
            .layer(from_fn(set_request_id))
            .layer(ServerTimeLayer),
    )
}
