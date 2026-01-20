mod connect;
mod initiate;
mod migrate;


use std::marker::PhantomData;

use crate::config;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Configuration error: {0}")]
    ConfigError(#[from] config::Error),
    #[error("SQLx error: {0}")]
    SqlxError(#[from] sqlx::Error),
    #[error("URL parse error: {0}")]
    UrlParseError(#[from] url::ParseError),
}

#[derive(Debug)]
pub struct ScopeDefault;

#[derive(Debug)]
pub struct ScopeAdmin;

#[derive(Debug)]
pub struct ScopeTenant;

#[derive(Debug)]
pub struct StateConnected;

#[derive(Debug)]
pub struct StateDisconnected;

#[derive(Debug)]
pub struct Pool<Scope, State = StateDisconnected> {
    pool: Option<sqlx::AnyPool>,
    _scope: PhantomData<Scope>,
    _state: PhantomData<State>,
}

impl<Scope> AsRef<sqlx::AnyPool> for Pool<Scope, StateConnected> {
    fn as_ref(&self) -> &sqlx::AnyPool {
        self.pool.as_ref().unwrap()
    }
}
