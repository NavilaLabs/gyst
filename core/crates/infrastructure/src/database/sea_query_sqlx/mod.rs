mod connect;
mod initialize;
mod migrate;

use std::marker::PhantomData;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Migrate error: {0}")]
    MigrateError(#[from] sqlx::migrate::MigrateError),
    #[error("SQLx error: {0}")]
    SqlxError(#[from] sqlx::Error),
    #[error("SeaORM error: {0}")]
    SeaOrmError(#[from] sea_orm::DbErr),
    #[error("SeaQuery error: {0}")]
    SeaQueryError(#[from] sea_query::error::Error),
    #[error("Unsupported database type")]
    UnsupportedDatabaseType,
    #[error("No tenant token provided")]
    NoTenantTokenProvided,
}

impl From<sqlx::migrate::MigrateError> for crate::Error {
    fn from(err: sqlx::migrate::MigrateError) -> Self {
        Error::MigrateError(err).into()
    }
}

impl From<sqlx::Error> for crate::Error {
    fn from(err: sqlx::Error) -> Self {
        Error::SqlxError(err).into()
    }
}

impl From<sea_orm::DbErr> for crate::Error {
    fn from(err: sea_orm::DbErr) -> Self {
        Error::SeaOrmError(err).into()
    }
}

impl From<sea_query::error::Error> for crate::Error {
    fn from(err: sea_query::error::Error) -> Self {
        Error::SeaQueryError(err).into()
    }
}

pub type ConnectedAdminPool = Pool<ScopeAdmin, StateConnected>;
pub type ConnectedTenantPool = Pool<ScopeTenant, StateConnected>;

#[derive(Debug)]
pub struct ScopeDefault;

#[derive(Debug)]
pub struct ScopeAdmin;

#[derive(Debug)]
pub struct ScopeTenant;

#[derive(Debug)]
pub struct StateConnected {
    pool: sqlx::AnyPool,
}

#[derive(Debug)]
pub struct StateDisconnected;

#[derive(Debug)]
pub enum DatabaseType {
    Postgres,
    Sqlite,
}

#[derive(Debug)]
pub struct Pool<Scope, State = StateDisconnected> {
    state: State,
    database_type: DatabaseType,
    _scope: PhantomData<Scope>,
}

impl<Scope> AsRef<sqlx::AnyPool> for Pool<Scope, StateConnected> {
    fn as_ref(&self) -> &sqlx::AnyPool {
        &self.state.pool
    }
}
