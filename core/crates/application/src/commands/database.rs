pub enum Connection {
    /// Command to disconnect from the database.
    Disconnect,
}

pub enum Initialize {
    /// Command to initialize the admin database.
    InitializeAdminDatabase,
    /// Command to initialize a tenant database.
    InitializeTenantDatabase { tenant_token: Option<String> },
}

pub enum RunMigrations {
    /// Command to migrate the admin database.
    MigrateAdminDatabase,
    /// Command to migrate a tenant database.
    MigrateTenantDatabase { tenant_token: Option<String> },
}
