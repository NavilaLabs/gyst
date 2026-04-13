use anyhow::{Result, anyhow};
use loom_infrastructure_impl::{ConnectedAdminPool, ConnectedTenantPool, Pool};
use std::collections::HashMap;
use std::sync::OnceLock;
use tokio::sync::RwLock;

static ADMIN_POOL: OnceLock<ConnectedAdminPool> = OnceLock::new();
static TENANT_POOLS: OnceLock<RwLock<HashMap<String, ConnectedTenantPool>>> = OnceLock::new();

pub async fn init(workspaces: &[String]) -> Result<()> {
    let admin = Pool::connect_admin().await?;
    ADMIN_POOL.set(admin).ok();

    let mut map = HashMap::new();
    for token in workspaces {
        map.insert(token.clone(), Pool::connect_tenant(token).await?);
    }
    TENANT_POOLS.set(RwLock::new(map)).ok();
    Ok(())
}

pub fn admin_pool() -> &'static ConnectedAdminPool {
    ADMIN_POOL.get().expect("pools not initialized")
}

pub async fn tenant_pool(workspace_id: &str) -> Result<ConnectedTenantPool> {
    let pools = TENANT_POOLS.get().expect("pools not initialized");
    let read = pools.read().await;
    read.get(workspace_id)
        .cloned()
        .ok_or_else(|| anyhow!("unknown workspace: {workspace_id}"))
}
