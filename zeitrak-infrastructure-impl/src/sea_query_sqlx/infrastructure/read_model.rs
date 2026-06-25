use sea_query::{Alias, Expr, Func, SelectStatement};
use sqlx::{AssertSqlSafe, Row, any::AnyRow};

use super::{ConnectedAdminPool, ConnectionProvider};

/// Read-model query helper for a single projection table.
///
/// Generic over the pool type `P` so it can serve both admin and tenant
/// repositories.  The default `P = ConnectedAdminPool` keeps existing admin
/// repository code unchanged.
pub struct SeaQueryReadModel<'a, P = ConnectedAdminPool> {
    pool: &'a P,
    table: &'static str,
}

impl<'a, P: ConnectionProvider> SeaQueryReadModel<'a, P> {
    #[must_use]
    pub const fn new(pool: &'a P, table: &'static str) -> Self {
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

    /// `SELECT <columns> FROM <table>` base statement.
    ///
    /// Use instead of [`select`](Self::select) when the table contains
    /// column types the `SQLx` `Any` driver cannot decode (e.g. Postgres
    /// `TIMESTAMPTZ`).
    #[must_use]
    pub fn select_columns(&self, columns: &[&str]) -> SelectStatement {
        let mut stmt = sea_query::Query::select();
        for col in columns {
            stmt.column(Alias::new(*col));
        }
        stmt.from(self.table).to_owned()
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
    pub async fn fetch_one_row(&self, stmt: &SelectStatement) -> Result<AnyRow, crate::Error> {
        let (sql, args) = self.pool.build_query(stmt);
        Ok(sqlx::query_with(AssertSqlSafe(sql.as_str()), args)
            .fetch_one(self.pool.any_pool())
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
        Ok(sqlx::query_with(AssertSqlSafe(sql.as_str()), args)
            .fetch_optional(self.pool.any_pool())
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
        Ok(sqlx::query_with(AssertSqlSafe(sql.as_str()), args)
            .fetch_all(self.pool.any_pool())
            .await?)
    }

    /// Execute a `SELECT COUNT(*)` statement and return the count.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn count_rows(&self, stmt: &SelectStatement) -> Result<u64, crate::Error> {
        let (sql, args) = self.pool.build_query(stmt);
        let row = sqlx::query_with(AssertSqlSafe(sql.as_str()), args)
            .fetch_one(self.pool.any_pool())
            .await?;
        let n: i64 = row.try_get(0usize)?;
        #[allow(clippy::cast_sign_loss)]
        Ok(n as u64)
    }
}
