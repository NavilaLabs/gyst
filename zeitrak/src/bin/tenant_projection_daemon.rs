use std::time::Duration;

use anyhow::{Result, anyhow};
use zeitrak::infrastructure::{
    BackoffConfig, Pool, ProjectionDaemon, ProjectionRunner, ProjectionSource, SqlCheckpoint,
    tenant::projectors::TenantProjector,
};
use zeitrak_infrastructure::repository::ReadRepository;
use zeitrak_infrastructure_impl::ConnectedAdminPool;
use tracing::warn;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Discover all workspace (tenant) IDs from the admin database.
    let mut admin_pool: Option<ConnectedAdminPool> = None;
    let mut is_initialized = false;
    while !is_initialized {
        if let Ok(connected_pool) = Pool::connect_admin().await {
            admin_pool = Some(connected_pool);
            is_initialized = true;
        } else {
            warn!(
                "Failed establishing connection to the admin database. This is ok if your have not set up yet."
            );
            tokio::time::sleep(Duration::from_secs(3)).await;
        }
    }
    if admin_pool.is_none() {
        return Err(anyhow!("expected connected admin pool"));
    }
    let admin_pool = admin_pool.unwrap();

    let workspace_repo =
        zeitrak::infrastructure::admin::workspace::repositories::WorkspaceRepository::from_pool(
            admin_pool,
        )
        .await?;
    let mut workspaces = workspace_repo.all().await?;

    while workspaces.is_empty() {
        tracing::warn!("No workspaces found in admin database — nothing to project.");
        tokio::time::sleep(Duration::from_secs(3)).await;
        workspaces = workspace_repo.all().await?;
    }

    tracing::info!(
        count = workspaces.len(),
        "Discovered workspaces; registering one TenantProjector per tenant database."
    );

    let backoff = BackoffConfig {
        min_idle_ms: 20,
        max_idle_ms: 200,
        ..Default::default()
    };

    let mut daemon = ProjectionDaemon::new();

    for workspace in workspaces {
        let tenant_token = workspace.id().to_string();

        let pool = match Pool::connect_tenant(&tenant_token).await {
            Ok(p) => p,
            Err(e) => {
                tracing::error!(
                    tenant_token = %tenant_token,
                    error = %e,
                    "Failed to connect to tenant database — skipping."
                );
                continue;
            }
        };

        // Run the projection runner migrations once per tenant database so the
        // `global_position` column and trigger are in place before we start.
        ProjectionRunner::new(pool.clone().into_pool(), ProjectionSource::AllStreams)
            .run_migrations()
            .await?;

        let checkpoint_name = format!("tenant_projection_{tenant_token}");
        let checkpoint = SqlCheckpoint::new(pool.clone().into_pool(), &checkpoint_name).await?;

        daemon.register_with_config(
            ProjectionRunner::new(pool.clone().into_pool(), ProjectionSource::AllStreams),
            TenantProjector::new(&pool),
            checkpoint,
            backoff.clone(),
        );

        tracing::info!(tenant_token = %tenant_token, "Registered TenantProjector.");
    }

    daemon.run_until_cancelled().await;

    Ok(())
}
