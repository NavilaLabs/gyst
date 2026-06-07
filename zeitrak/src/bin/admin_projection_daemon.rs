use std::time::Duration;

use anyhow::{Result, anyhow};
use tracing::warn;
use zeitrak::infrastructure::{
    BackoffConfig, Pool, ProjectionDaemon, ProjectionRunner, ProjectionSource, SqlCheckpoint,
    admin::projectors::AdminProjector,
};
use zeitrak_infrastructure_impl::ConnectedAdminPool;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let mut pool: Option<ConnectedAdminPool> = None;
    let mut is_initialized = false;
    while !is_initialized {
        if let Ok(connected_pool) = Pool::connect_admin().await {
            pool = Some(connected_pool);
            is_initialized = true;
        } else {
            warn!(
                "Failed establishing connection to the admin database. This is ok if your have not set up yet."
            );
            tokio::time::sleep(Duration::from_secs(3)).await;
        }
    }
    if pool.is_none() {
        return Err(anyhow!("expected connected admin pool"));
    }
    let pool = pool.unwrap();

    // Add the global_position column + trigger that the projection runner needs.
    // This is idempotent — safe to call on every startup.
    ProjectionRunner::new(pool.clone().into_pool(), ProjectionSource::AllStreams)
        .await
        .run_migrations()
        .await?;

    let backoff = BackoffConfig {
        min_idle: Duration::from_millis(20),
        max_idle: Duration::from_millis(200),
        ..Default::default()
    };

    let mut daemon = ProjectionDaemon::new();

    // All admin projectors run under a single runner with a single checkpoint.
    // This guarantees events are applied sequentially across all projection
    // tables, preventing FK race conditions between independent runners.
    daemon.register_with_config(
        ProjectionRunner::new(pool.clone().into_pool(), ProjectionSource::AllStreams).await,
        AdminProjector::new(pool.clone()),
        SqlCheckpoint::new(pool.clone().into_pool(), "admin_projection").await?,
        backoff,
    );

    daemon.run_until_cancelled().await;

    Ok(())
}
