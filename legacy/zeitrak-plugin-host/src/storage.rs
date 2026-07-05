//! Plugin storage services (Phase F — §10).
//!
//! Provides two storage services — one per pool scope — so that pool-scope
//! enforcement is **compile-time**, not runtime branching:
//!
//! | Service | Pool | Allowed tiers |
//! |---|---|---|
//! | [`PluginStorageService`] | `Pool<ScopeTenant, StateConnected>` | Tenant, Instance, Signed |
//! | [`PluginAdminStorageService`] | `Pool<ScopeAdmin, StateConnected>` | Instance, Signed only |
//!
//! Both services offer the same capabilities:
//! - **KV state** — wraps the `dioxus-extism` runtime's global-state map.
//! - **Migrations** — run arbitrary DDL against the tenant / admin DB.
//! - **`query_raw`** — parameterised `SELECT` with a mandatory table-prefix guard.
//!
//! ## Table prefix guard (§10, `sqlparser`)
//!
//! `query_raw` parses the SQL with [`sqlparser`] and rejects any statement
//! that references a table whose name does not start with
//! `plugin_<sanitised_plugin_id>__`.  Non-`SELECT` statements are also
//! rejected.  The sanitised prefix is computed by [`make_plugin_prefix`].
//!
//! ## `migrate`
//!
//! Executes the supplied DDL inside a single transaction.  If any statement
//! fails the transaction is rolled back and an error is returned.  Callers
//! should run this at plugin install time.

use std::ops::ControlFlow;
use std::sync::Arc;

use dioxus_extism_host::PluginRuntime;
use dioxus_extism_protocol::PluginId;
use sqlparser::ast::{Statement, visit_relations};
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;
use sqlx::AssertSqlSafe;
use sqlx::Column as _;
use sqlx::Row as _;
use thiserror::Error;
use zeitrak_infrastructure_impl::{ConnectedAdminPool, ConnectedTenantPool};

use crate::error::PluginHostError;
use crate::host_ctx::ZeitrakHostCtx;

// ── make_plugin_prefix ────────────────────────────────────────────────────────

/// Compute the required SQL table prefix for `plugin_id`.
///
/// Sanitises the ID by replacing `/`, `-`, and `.` with `_`, then wraps it:
/// `plugin_<sanitised>__`.
///
/// | `plugin_id` | Result |
/// |---|---|
/// | `my-org/leave-guard` | `plugin_my_org_leave_guard__` |
/// | `acme.corp` | `plugin_acme_corp__` |
#[must_use]
pub fn make_plugin_prefix(plugin_id: &str) -> String {
    let sanitized = plugin_id.replace(['/', '-', '.'], "_");
    format!("plugin_{sanitized}__")
}

// ── StorageError ──────────────────────────────────────────────────────────────

/// Errors returned by [`PluginStorageService`] and [`PluginAdminStorageService`].
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum StorageError {
    /// `query_raw` received a statement that is not a `SELECT`.
    #[error("only SELECT statements are allowed in query_raw; got: {0}")]
    NonSelectStatement(String),

    /// `query_raw` referenced a table not covered by the plugin's prefix.
    #[error("table '{table}' is not in the allowed prefix '{prefix}' for plugin '{plugin_id}'")]
    TableAccessDenied {
        /// Plugin that issued the query.
        plugin_id: String,
        /// The table that was referenced.
        table: String,
        /// The allowed prefix.
        prefix: String,
    },

    /// The SQL string could not be parsed.
    #[error("failed to parse SQL: {0}")]
    ParseError(String),

    /// A database operation failed.
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    /// An unsupported `StateScope` was requested for a KV operation.
    #[error(
        "KV operation with PerSession scope requires a session_id; use global scope or pass session_id"
    )]
    UnsupportedScope,
}

// ── Table-prefix guard ────────────────────────────────────────────────────────

/// Parse `sql` and verify that every referenced table starts with `prefix`.
///
/// Returns `Err(StorageError)` on any violation.
fn check_table_prefix(plugin_id: &str, sql: &str) -> Result<Vec<Statement>, StorageError> {
    let dialect = GenericDialect {};
    let statements =
        Parser::parse_sql(&dialect, sql).map_err(|e| StorageError::ParseError(e.to_string()))?;

    let prefix = make_plugin_prefix(plugin_id);

    for stmt in &statements {
        // Reject non-SELECT statements.
        if !matches!(stmt, Statement::Query(_)) {
            return Err(StorageError::NonSelectStatement(
                stmt.to_string().chars().take(60).collect(),
            ));
        }

        // Walk every relation (table reference) in the statement.
        let mut violation: Option<String> = None;
        let _ = visit_relations(stmt, |rel| {
            let name = rel.0.last().map_or("", |ident| ident.value.as_str());
            if !name.starts_with(prefix.as_str()) {
                violation = Some(name.to_string());
                return ControlFlow::Break(());
            }
            ControlFlow::Continue(())
        });

        if let Some(bad_table) = violation {
            return Err(StorageError::TableAccessDenied {
                plugin_id: plugin_id.to_string(),
                table: bad_table,
                prefix,
            });
        }
    }

    Ok(statements)
}

// ── PluginStorageService ──────────────────────────────────────────────────────

/// Tenant-scoped plugin storage service.
///
/// Holds a [`Pool<ScopeTenant, StateConnected>`][ConnectedTenantPool] —
/// never grants access to admin-scope data.
///
/// Construct one instance per plugin at install time and keep it alive as long
/// as the plugin is loaded.
pub struct PluginStorageService {
    pool: sqlx::AnyPool,
    runtime: Arc<PluginRuntime<ZeitrakHostCtx>>,
}

impl std::fmt::Debug for PluginStorageService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginStorageService")
            .finish_non_exhaustive()
    }
}

impl PluginStorageService {
    /// Construct a new `PluginStorageService`.
    ///
    /// Takes the **tenant** pool — never admin.
    #[must_use]
    pub fn new(pool: &ConnectedTenantPool, runtime: Arc<PluginRuntime<ZeitrakHostCtx>>) -> Self {
        Self {
            pool: pool.as_ref().clone(),
            runtime,
        }
    }

    /// Read a value from the plugin's global state.
    ///
    /// Returns `None` if the key does not exist.
    ///
    /// # Errors
    ///
    /// Returns [`PluginHostError`] if the scope is `PerSession` (use the admin
    /// variant for session-scoped state, or use `StateScope::Global`).
    pub async fn kv_get(
        &self,
        plugin: &PluginId,
        key: &str,
    ) -> Result<Option<serde_json::Value>, PluginHostError> {
        Ok(self.runtime.global_state_json(plugin, key).await)
    }

    /// Write a value to the plugin's global state.
    ///
    /// # Errors
    ///
    /// Returns [`PluginHostError`] on internal state errors (currently
    /// infallible, but the signature leaves room for persistence errors).
    pub async fn kv_set(
        &self,
        plugin: &PluginId,
        key: &str,
        value: serde_json::Value,
    ) -> Result<(), PluginHostError> {
        // dioxus-extism global state: write via set_plugin_state with a
        // stable synthetic session_id for cross-session persistence.
        self.runtime
            .set_plugin_state(plugin, &plugin_global_session(plugin), key, value)
            .await;
        Ok(())
    }

    /// Delete a key from the plugin's global state.
    ///
    /// No-op if the key does not exist.
    ///
    /// # Errors
    ///
    /// Currently infallible; signature is compatible with future persistence backends.
    pub async fn kv_delete(&self, plugin: &PluginId, key: &str) -> Result<(), PluginHostError> {
        // Clear the key by writing Value::Null; the host treats Null as absent.
        self.runtime
            .set_plugin_state(
                plugin,
                &plugin_global_session(plugin),
                key,
                serde_json::Value::Null,
            )
            .await;
        Ok(())
    }

    /// Execute one or more DDL statements against the tenant database.
    ///
    /// All statements run inside a single transaction.  If any statement
    /// fails the transaction is rolled back and an error is returned.
    ///
    /// # Errors
    ///
    /// Returns [`StorageError::Database`] on SQL execution failure.
    pub async fn migrate(&self, sql: &str) -> Result<(), StorageError> {
        let mut tx = self.pool.begin().await?;
        for stmt in sql.split(';').map(str::trim).filter(|s| !s.is_empty()) {
            sqlx::query(AssertSqlSafe(stmt)).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    /// Execute a `SELECT` statement against the tenant database.
    ///
    /// Every table referenced in `sql` must start with
    /// `plugin_<sanitised_plugin_id>__`; any other reference returns
    /// [`StorageError::TableAccessDenied`].
    ///
    /// `params` are bound in order as `$1`, `$2`, … (Postgres) or `` ? `` (`SQLite`).
    ///
    /// # Errors
    ///
    /// Returns [`StorageError`] on parse failure, access violation, or SQL error.
    pub async fn query_raw(
        &self,
        plugin: &PluginId,
        sql: &str,
        params: Vec<serde_json::Value>,
    ) -> Result<Vec<serde_json::Value>, StorageError> {
        check_table_prefix(&plugin.0, sql)?;
        execute_query_raw(&self.pool, sql, params).await
    }
}

// ── PluginAdminStorageService ─────────────────────────────────────────────────

/// Admin-scoped plugin storage service.
///
/// Holds a [`Pool<ScopeAdmin, StateConnected>`][ConnectedAdminPool].
/// Only construct this for plugins whose
/// [`ZeitrakTrustTier`][crate::trust::ZeitrakTrustTier] is
/// `Instance` or `SignedInstance`.
///
/// The type system prevents any tenant-scoped code from obtaining an
/// `PluginAdminStorageService` accidentally.
pub struct PluginAdminStorageService {
    pool: sqlx::AnyPool,
    runtime: Arc<PluginRuntime<ZeitrakHostCtx>>,
}

impl std::fmt::Debug for PluginAdminStorageService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginAdminStorageService")
            .finish_non_exhaustive()
    }
}

impl PluginAdminStorageService {
    /// Construct a new `PluginAdminStorageService`.
    ///
    /// Takes the **admin** pool — never tenant.
    #[must_use]
    pub fn new(pool: &ConnectedAdminPool, runtime: Arc<PluginRuntime<ZeitrakHostCtx>>) -> Self {
        Self {
            pool: pool.as_ref().clone(),
            runtime,
        }
    }

    /// Read a value from the plugin's global state (admin context).
    ///
    /// Returns `None` if the key does not exist.
    ///
    /// # Errors
    ///
    /// Currently infallible; signature is compatible with future persistence backends.
    pub async fn kv_get(
        &self,
        plugin: &PluginId,
        key: &str,
    ) -> Result<Option<serde_json::Value>, PluginHostError> {
        Ok(self.runtime.global_state_json(plugin, key).await)
    }

    /// Write a value to the plugin's global state (admin context).
    ///
    /// # Errors
    ///
    /// Currently infallible; signature is compatible with future persistence backends.
    pub async fn kv_set(
        &self,
        plugin: &PluginId,
        key: &str,
        value: serde_json::Value,
    ) -> Result<(), PluginHostError> {
        self.runtime
            .set_plugin_state(plugin, &plugin_global_session(plugin), key, value)
            .await;
        Ok(())
    }

    /// Delete a key from the plugin's global state (admin context).
    ///
    /// No-op if the key does not exist.
    ///
    /// # Errors
    ///
    /// Currently infallible; signature is compatible with future persistence backends.
    pub async fn kv_delete(&self, plugin: &PluginId, key: &str) -> Result<(), PluginHostError> {
        self.runtime
            .set_plugin_state(
                plugin,
                &plugin_global_session(plugin),
                key,
                serde_json::Value::Null,
            )
            .await;
        Ok(())
    }

    /// Execute DDL against the admin database.
    ///
    /// # Errors
    ///
    /// Returns [`StorageError::Database`] on SQL execution failure.
    pub async fn migrate(&self, sql: &str) -> Result<(), StorageError> {
        let mut tx = self.pool.begin().await?;
        for stmt in sql.split(';').map(str::trim).filter(|s| !s.is_empty()) {
            sqlx::query(AssertSqlSafe(stmt)).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    /// Execute a `SELECT` against the admin database with the table-prefix guard.
    ///
    /// # Errors
    ///
    /// Returns [`StorageError`] on parse failure, access violation, or SQL error.
    pub async fn query_raw(
        &self,
        plugin: &PluginId,
        sql: &str,
        params: Vec<serde_json::Value>,
    ) -> Result<Vec<serde_json::Value>, StorageError> {
        check_table_prefix(&plugin.0, sql)?;
        execute_query_raw(&self.pool, sql, params).await
    }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Stable per-plugin "session" used for global-state reads and writes from the
/// host side.  Using the plugin ID itself as the session ID means global state
/// is durable across actual user sessions.
fn plugin_global_session(plugin: &PluginId) -> dioxus_extism_protocol::SessionId {
    dioxus_extism_protocol::SessionId(plugin.0.clone())
}

/// Execute a parameterised query and return results as JSON values.
///
/// Parameters are bound as positional arguments in declaration order.
/// Result rows are returned as `serde_json::Value::Object` maps.
///
/// # Errors
///
/// Returns [`StorageError::Database`] on SQL execution or fetch failure.
async fn execute_query_raw(
    pool: &sqlx::AnyPool,
    sql: &str,
    params: Vec<serde_json::Value>,
) -> Result<Vec<serde_json::Value>, StorageError> {
    let mut query = sqlx::query(AssertSqlSafe(sql));
    for param in params {
        match param {
            serde_json::Value::Null => query = query.bind(Option::<String>::None),
            serde_json::Value::Bool(b) => query = query.bind(b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    query = query.bind(i);
                } else if let Some(f) = n.as_f64() {
                    query = query.bind(f);
                } else {
                    query = query.bind(n.to_string());
                }
            }
            serde_json::Value::String(s) => query = query.bind(s),
            other => query = query.bind(other.to_string()),
        }
    }

    let rows = query.fetch_all(pool).await?;
    let mut results = Vec::with_capacity(rows.len());

    for row in &rows {
        let mut obj = serde_json::Map::new();
        for col in row.columns() {
            let name = col.name();
            let value: serde_json::Value = row
                .try_get::<Option<String>, _>(name)
                .ok()
                .flatten()
                .map(serde_json::Value::String)
                .or_else(|| {
                    row.try_get::<Option<i64>, _>(name)
                        .ok()
                        .flatten()
                        .map(|n| serde_json::Value::Number(n.into()))
                })
                .or_else(|| {
                    row.try_get::<Option<f64>, _>(name)
                        .ok()
                        .flatten()
                        .and_then(|f| {
                            serde_json::Number::from_f64(f).map(serde_json::Value::Number)
                        })
                })
                .or_else(|| {
                    row.try_get::<Option<bool>, _>(name)
                        .ok()
                        .flatten()
                        .map(serde_json::Value::Bool)
                })
                .unwrap_or(serde_json::Value::Null);
            obj.insert(name.to_string(), value);
        }
        results.push(serde_json::Value::Object(obj));
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_plugin_prefix_sanitizes_slashes_and_hyphens() {
        assert_eq!(
            make_plugin_prefix("my-org/leave-guard"),
            "plugin_my_org_leave_guard__"
        );
    }

    #[test]
    fn make_plugin_prefix_sanitizes_dots() {
        assert_eq!(make_plugin_prefix("acme.corp"), "plugin_acme_corp__");
    }

    #[test]
    fn check_table_prefix_allows_valid_table() {
        let result = check_table_prefix(
            "my-org/leave-guard",
            "SELECT id FROM plugin_my_org_leave_guard__leave_requests WHERE status = 'open'",
        );
        assert!(result.is_ok(), "{result:?}");
    }

    #[test]
    fn check_table_prefix_rejects_foreign_table() {
        let result = check_table_prefix("my-org/leave-guard", "SELECT * FROM users");
        assert!(
            matches!(result, Err(StorageError::TableAccessDenied { .. })),
            "expected TableAccessDenied, got: {result:?}"
        );
    }

    #[test]
    fn check_table_prefix_rejects_non_select() {
        let result = check_table_prefix(
            "my-org/leave-guard",
            "DELETE FROM plugin_my_org_leave_guard__leave_requests",
        );
        assert!(
            matches!(result, Err(StorageError::NonSelectStatement(_))),
            "expected NonSelectStatement, got: {result:?}"
        );
    }

    #[test]
    fn check_table_prefix_rejects_malformed_sql() {
        let result = check_table_prefix("myplugin", "THIS IS NOT SQL @@@@");
        assert!(matches!(result, Err(StorageError::ParseError(_))));
    }

    #[test]
    fn check_table_prefix_allows_multiple_tables_with_correct_prefix() {
        let result = check_table_prefix(
            "myplugin",
            "SELECT a.id, b.name FROM plugin_myplugin__table_a AS a JOIN plugin_myplugin__table_b AS b ON a.id = b.fk",
        );
        assert!(result.is_ok(), "{result:?}");
    }

    #[test]
    fn check_table_prefix_rejects_mixed_tables() {
        let result = check_table_prefix(
            "myplugin",
            "SELECT a.id FROM plugin_myplugin__table_a AS a JOIN other_table AS b ON a.id = b.fk",
        );
        assert!(
            matches!(result, Err(StorageError::TableAccessDenied { .. })),
            "{result:?}"
        );
    }
}
