use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Database {
    base_uri: String,
    databases: Databases,
    pool: Pool,
}

impl Default for Database {
    fn default() -> Self {
        Self {
            base_uri: "sqlite:///workspaces/zeitrak/.devcontainer/database".to_string(),
            databases: Databases::default(),
            pool: Pool::default(),
        }
    }
}

impl Database {
    #[must_use]
    pub fn base_uri(&self) -> &str {
        &self.base_uri
    }

    #[must_use]
    pub const fn databases(&self) -> &Databases {
        &self.databases
    }

    #[must_use]
    pub const fn pool(&self) -> &Pool {
        &self.pool
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Databases {
    admin: AdminDatabase,
    tenant: TenantDatabase,
}

impl Databases {
    #[must_use]
    pub const fn admin(&self) -> &AdminDatabase {
        &self.admin
    }

    #[must_use]
    pub const fn tenant(&self) -> &TenantDatabase {
        &self.tenant
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminDatabase {
    name: String,
}

impl Default for AdminDatabase {
    fn default() -> Self {
        Self {
            name: "zeitrak_admin".to_string(),
        }
    }
}

impl AdminDatabase {
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantDatabase {
    name_prefix: String,
}

impl Default for TenantDatabase {
    fn default() -> Self {
        Self {
            name_prefix: "zeitrak_tenant_".to_string(),
        }
    }
}

impl TenantDatabase {
    #[must_use]
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

impl Default for Pool {
    fn default() -> Self {
        Self {
            max_size: 20,
            min_size: 5,
            timeout_seconds: 30,
        }
    }
}

impl Pool {
    #[must_use]
    pub const fn max_size(&self) -> u32 {
        self.max_size
    }

    #[must_use]
    pub const fn min_size(&self) -> u32 {
        self.min_size
    }

    #[must_use]
    pub const fn timeout_seconds(&self) -> u64 {
        self.timeout_seconds
    }
}
