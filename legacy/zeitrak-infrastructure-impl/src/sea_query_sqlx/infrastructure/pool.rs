use std::{fmt::Display, marker::PhantomData};

use sea_query::{PostgresQueryBuilder, SqliteQueryBuilder};
use sea_query_sqlx::{SqlxBinder, SqlxValues};
use sqlx::types::Uuid;
use zeitrak_infrastructure::ImplError;

pub type ConnectedAdminPool = Pool<ScopeAdmin, StateConnected>;
pub type ConnectedTenantPool = Pool<ScopeTenant, StateConnected>;

#[derive(Debug, Clone)]
pub struct ScopeDefault;

#[derive(Debug, Clone)]
pub struct ScopeAdmin;

#[derive(Debug, Clone)]
pub struct ScopeTenant;

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
    scope: PhantomData<Scope>,
    tenant_id: Option<Uuid>,
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
    pub const fn new(state: State, database_type: DatabaseType, tenant_id: Option<Uuid>) -> Self {
        Self {
            state,
            database_type,
            scope: PhantomData,
            tenant_id,
        }
    }

    pub const fn database_type(&self) -> &DatabaseType {
        &self.database_type
    }

    pub const fn tenant_id(&self) -> Option<&Uuid> {
        self.tenant_id.as_ref()
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
    pub fn uri(&self) -> url::Url {
        self.state.pool.connect_options().database_url.clone()
    }
}

// ── ConnectionProvider ────────────────────────────────────────────────────────

/// Minimal abstraction over anything that can produce a `sqlx::AnyPool`.
///
/// Implemented by [`ConnectedAdminPool`] and [`ConnectedTenantPool`] so that
/// generic infrastructure code and plugin authors can accept a pool without
/// caring about the scope marker.  Also trivially implementable by test stubs.
pub trait ConnectionProvider: Send + Sync + Clone {
    fn any_pool(&self) -> &sqlx::AnyPool;
    fn build_query<S: SqlxBinder>(&self, statement: &S) -> (String, SqlxValues);
}

impl<Scope: Send + Sync + Clone> ConnectionProvider for Pool<Scope, StateConnected> {
    fn any_pool(&self) -> &sqlx::AnyPool {
        &self.state.pool
    }

    fn build_query<S: SqlxBinder>(&self, statement: &S) -> (String, SqlxValues) {
        self.database_type.build_query(statement)
    }
}
