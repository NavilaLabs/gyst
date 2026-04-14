//! Thin helper for admin projection read-model queries.
//!
//! Admin repositories implement [`loom_infrastructure::query::Query`] on top
//! of `sea-query` + `sqlx::AnyPool`. All eight method bodies follow the same
//! three-step pattern:
//!
//! 1. Build a `SelectStatement` (with or without a filter / count function).
//! 2. Compile it to `(sql, SqlxValues)` via [`ConnectedAdminPool::build_query`].
//! 3. Execute it and return the row(s).
//!
//! [`SeaQueryReadModel`] captures the pool reference and the table name so
//! each query method becomes a one-liner in the concrete repository.

use sea_query::{Expr, Func, SelectStatement};
use sqlx::{Row, any::AnyRow};

use crate::ConnectedAdminPool;

/// Read-model query helper for a single admin projection table.
///
/// Construct one per method via [`crate::sea_query_sqlx::admin`] repositories'
/// private `read_model()` accessors:
///
/// ```ignore
/// fn read_model(&self) -> SeaQueryReadModel<'_> {
///     SeaQueryReadModel::new(&self.store.pool, TABLE)
/// }
/// ```
pub struct SeaQueryReadModel<'a> {
    pool: &'a ConnectedAdminPool,
    table: &'static str,
}

impl<'a> SeaQueryReadModel<'a> {
    #[must_use]
    pub const fn new(pool: &'a ConnectedAdminPool, table: &'static str) -> Self {
        Self { pool, table }
    }

    /// `SELECT * FROM <table>` base statement.
    #[must_use]
    pub fn select(&self) -> SelectStatement {
        sea_query::Query::select()
            .expr(Expr::col(sea_query::Asterisk))
            .from(self.table)
            .to_owned()
    }

    /// `SELECT COUNT(*) FROM <table>` base statement.
    #[must_use]
    pub fn select_count(&self) -> SelectStatement {
        sea_query::Query::select()
            .expr(Func::count(Expr::col(sea_query::Asterisk)))
            .from(self.table)
            .to_owned()
    }

    /// Execute a `SELECT` statement and return exactly one row.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails or no row is found.
    pub async fn fetch_one_row(
        &self,
        stmt: &SelectStatement,
    ) -> Result<AnyRow, crate::Error> {
        let (sql, args) = self.pool.build_query(stmt);
        Ok(sqlx::query_with(&sql, args)
            .fetch_one(self.pool.as_ref())
            .await?)
    }

    /// Execute a `SELECT` statement and return at most one row.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn fetch_optional_row(
        &self,
        stmt: &SelectStatement,
    ) -> Result<Option<AnyRow>, crate::Error> {
        let (sql, args) = self.pool.build_query(stmt);
        Ok(sqlx::query_with(&sql, args)
            .fetch_optional(self.pool.as_ref())
            .await?)
    }

    /// Execute a `SELECT` statement and return all matching rows.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn fetch_all_rows(
        &self,
        stmt: &SelectStatement,
    ) -> Result<Vec<AnyRow>, crate::Error> {
        let (sql, args) = self.pool.build_query(stmt);
        Ok(sqlx::query_with(&sql, args)
            .fetch_all(self.pool.as_ref())
            .await?)
    }

    /// Execute a `SELECT COUNT(*)` statement and return the count.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn count_rows(&self, stmt: &SelectStatement) -> Result<u64, crate::Error> {
        let (sql, args) = self.pool.build_query(stmt);
        let row = sqlx::query_with(&sql, args)
            .fetch_one(self.pool.as_ref())
            .await?;
        let n: i64 = row.try_get(0)?;
        #[allow(clippy::cast_sign_loss)]
        Ok(n as u64)
    }
}
