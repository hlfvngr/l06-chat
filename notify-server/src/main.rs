use anyhow::Result;
use notify_server::{AppConfig, AppState, get_router, start_background_task};
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let app_config = AppConfig::load()?;
    let state = AppState::try_new(app_config.clone()).await?;

    start_background_task(state.clone()).await?;

    let app = get_router(state);

    let addr = format!("0.0.0.0:{}", app_config.port);
    let listener = TcpListener::bind(&addr).await?;
    info!("listening on {}", addr);
    axum::serve(listener, app).await?;
    Ok(())
}
