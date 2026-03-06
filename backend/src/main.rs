//! Punto de entrada del backend Rockola.

use std::path::Path;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use rockola_backend::{create_app, AppState, Config};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();
    let config = Config::from_env()?;

    if let Some(path) = config.database_url.strip_prefix("sqlite:") {
        let db_path = Path::new(path);
        if let Some(parent) = db_path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        tracing::info!("database: {}", path);
    }

    std::fs::create_dir_all(&config.media_root).ok();

    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect(&config.database_url)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    rockola_backend::ensure_default_credits(&pool).await?;

    let state = AppState {
        pool: pool.clone(),
        config: config.clone(),
        admin_sessions: std::sync::Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        admin_failed: std::sync::Arc::new(tokio::sync::Mutex::new(rockola_backend::AdminFailedAttempts::default())),
    };

    // Worker de descargas en background: procesa jobs en downloads_queue.
    let pool_worker = pool.clone();
    let config_worker = config.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(3));
        loop {
            interval.tick().await;
            if let Ok(Some(job)) = rockola_backend::get_next_download_job(&pool_worker).await {
                if let Err(e) = rockola_backend::run_download_job(
                    pool_worker.clone(),
                    &config_worker,
                    &job,
                )
                .await
                {
                    tracing::error!("download job {} failed: {}", job.id, e);
                }
            }
        }
    });

    let app = create_app(state);
    let port = config.port;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("listening on {}", addr);
    axum::serve(
        tokio::net::TcpListener::bind(addr).await?,
        app,
    )
    .await?;
    Ok(())
}
