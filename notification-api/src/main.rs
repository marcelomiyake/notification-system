use notification_api::{AppState, connect_pool, router, seed_caller};
use sqlx::migrate;
use std::{env, net::SocketAddr};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is required");
    let api_key = env::var("CALLER_API_KEY").unwrap_or_else(|_| "local-dev-api-key".into());
    let simulation_enabled =
        env::var("DEMO_SIMULATION_ENABLED").unwrap_or_else(|_| "true".into()) == "true";
    let pool = connect_pool(&database_url).await?;
    migrate!("../database/migrations").run(&pool).await?;
    seed_caller(&pool, &api_key).await?;
    let state = AppState::new(pool, simulation_enabled);
    let address: SocketAddr = env::var("BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8080".into())
        .parse()?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    tracing::info!(%address, "notification API listening");
    axum::serve(listener, router(state)).await?;
    Ok(())
}
