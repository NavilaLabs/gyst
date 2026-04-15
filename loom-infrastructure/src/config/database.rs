use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Database {
    base_uri: String,
    databases: Databases,
    pool: Pool,
}

impl Database {
    pub fn base_uri(&self) -> &str {
        &self.base_uri
    }

    pub const fn databases(&self) -> &Databases {
        &self.databases
    }

    pub const fn pool(&self) -> &Pool {
        &self.pool
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Databases {
    admin: AdminDatabase,
    tenant: TenantDatabase,
}

impl Databases {
    pub const fn admin(&self) -> &AdminDatabase {
        &self.admin
    }

    pub const fn tenant(&self) -> &TenantDatabase {
        &self.tenant
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminDatabase {
    name: String,
}

impl AdminDatabase {
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantDatabase {
    name_prefix: String,
}

impl TenantDatabase {
    pub fn name_prefix(&self) -> &str {
        &self.name_prefix
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pool {
    max_size: u32,
    min_size: u32,
    timeout_seconds: u64,
}

impl Pool {
    pub const fn max_size(&self) -> u32 {
        self.max_size
    }

    pub const fn min_size(&self) -> u32 {
        self.min_size
    }

    pub const fn timeout_seconds(&self) -> u64 {
        self.timeout_seconds
    }
}
