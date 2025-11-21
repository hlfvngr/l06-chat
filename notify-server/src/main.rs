use anyhow::Result;
use notify_server::{AppConfig, get_router};
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::EnvFilter;
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let app_config = AppConfig::load()?;
    let addr = format!("0.0.0.0:{}", app_config.port);
    let listener = TcpListener::bind(&addr).await?;
    info!("listening on {}", addr);

    let app = get_router();
    axum::serve(listener, app).await?;
    Ok(())
}
