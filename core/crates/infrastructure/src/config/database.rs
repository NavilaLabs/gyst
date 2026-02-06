use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Database {
    base_uri: String,
    databases: Databases,
    pool: Pool,
}

impl Database {
    pub fn get_base_uri(&self) -> &str {
        &self.base_uri
    }

    pub fn get_databases(&self) -> &Databases {
        &self.databases
    }

    pub fn get_pool(&self) -> &Pool {
        &self.pool
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Databases {
    admin: AdminDatabase,
    tenant: TenantDatabase,
}

impl Databases {
    pub fn get_admin(&self) -> &AdminDatabase {
        &self.admin
    }

    pub fn get_tenant(&self) -> &TenantDatabase {
        &self.tenant
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminDatabase {
    name: String,
}

impl AdminDatabase {
    pub fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantDatabase {
    name_prefix: String,
}

impl TenantDatabase {
    pub fn get_name_prefix(&self) -> &str {
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
    pub fn get_max_size(&self) -> u32 {
        self.max_size
    }

    pub fn get_min_size(&self) -> u32 {
        self.min_size
    }

    pub fn get_timeout_seconds(&self) -> u64 {
        self.timeout_seconds
    }
}
