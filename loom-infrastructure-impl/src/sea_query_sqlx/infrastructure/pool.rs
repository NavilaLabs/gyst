use std::fmt::Display;

use loom_infrastructure::ImplError;
use sea_query::{PostgresQueryBuilder, SqliteQueryBuilder};
use sea_query_sqlx::{SqlxBinder, SqlxValues};
use sqlx::types::Uuid;
use url::Url;

pub type ConnectedAdminPool = Pool<ScopeAdmin, StateConnected>;
pub type ConnectedTenantPool = Pool<ScopeTenant, StateConnected>;

impl ConnectedTenantPool {
    pub fn tenant_id(&self) -> Uuid {
        self._scope
    }
}

#[derive(Debug, Clone)]
pub struct ScopeDefault;

#[derive(Debug, Clone)]
pub struct ScopeAdmin;

#[derive(Debug, Clone)]
pub struct ScopeTenant {
    tenant_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct StateConnected {
    pool: sqlx::AnyPool,
}

impl StateConnected {
    #[must_use]
    pub const fn new(pool: sqlx::AnyPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, Clone)]
pub struct StateDisconnected;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DatabaseType {
    Postgres,
    Sqlite,
}

impl Display for DatabaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Postgres => write!(f, "postgres"),
            Self::Sqlite => write!(f, "sqlite"),
        }
    }
}

impl DatabaseType {
    pub(crate) fn build_query<S: SqlxBinder>(&self, statement: &S) -> (String, SqlxValues) {
        match self {
            Self::Postgres => statement.build_sqlx(PostgresQueryBuilder),
            Self::Sqlite => statement.build_sqlx(SqliteQueryBuilder),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Pool<Scope, State = StateDisconnected> {
    state: State,
    database_type: DatabaseType,
    scope: Scope,
}

impl<Scope, State> ImplError for Pool<Scope, State> {
    type Error = crate::Error;
}

impl<Scope> AsRef<sqlx::AnyPool> for Pool<Scope, StateConnected> {
    fn as_ref(&self) -> &sqlx::AnyPool {
        &self.state.pool
    }
}

impl<Scope, State> Pool<Scope, State>
where
    Self: Sized,
{
    pub const fn new(state: State, database_type: DatabaseType, scope: Scope) -> Self {
        Self {
            state,
            database_type,
            scope,
        }
    }

    pub const fn get_database_type(&self) -> &DatabaseType {
        &self.database_type
    }

    pub fn build_query<S: SqlxBinder>(&self, statement: &S) -> (String, SqlxValues) {
        self.database_type.build_query(statement)
    }
}

impl<Scope> Pool<Scope, StateConnected> {
    #[must_use]
    pub fn into_pool(self) -> sqlx::AnyPool {
        self.state.pool
    }

    #[must_use]
    pub fn get_uri(&self) -> Url {
        self.state.pool.connect_options().database_url.clone()
    }
}
