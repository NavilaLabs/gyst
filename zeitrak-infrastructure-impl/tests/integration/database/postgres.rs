use sqlx::AssertSqlSafe;
use tracing::info;
use url::Url;
use zeitrak_infrastructure::{
    config::CONFIG,
    database::{
        DatabaseUri, Migrate, TenantDatabaseNameBuilder, TenantDatabaseNameConcreteBuilder,
        TenantDatabaseNameDirector,
    },
};
use zeitrak_infrastructure_impl::{
    Error, {Pool, ScopeAdmin, ScopeTenant, StateConnected},
};

use super::{ConnectedDefaultPool, initialize_databases};

/// Escapes a `PostgreSQL` quoted identifier by doubling any embedded double-quote
/// characters.  The resulting string is safe to embed between `"..."` delimiters
/// in a dynamically-built SQL statement.
///
/// `PostgreSQL` does not support parameterised identifiers (only values can be
/// bound with `$N`), so this is the correct way to handle untrusted or
/// config-sourced database names.
fn escape_pg_identifier(name: &str) -> String {
    name.replace('"', "\"\"")
}

/// Escapes a string for use as the left-hand side of a `PostgreSQL` `LIKE` pattern
/// by prefixing `%`, `_`, and `\` with a backslash escape character.
fn escape_pg_like_prefix(prefix: &str) -> String {
    prefix
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

#[allow(dead_code)]
async fn reset_entire_database(pool: &ConnectedDefaultPool) -> Result<(), Error> {
    sqlx::query(
        "SELECT pg_terminate_backend(pid) FROM pg_stat_activity
         WHERE datname LIKE 'test_zeitrak_%' AND pid <> pg_backend_pid()",
    )
    .execute(pool.as_ref())
    .await?;

    let admin_database_name = CONFIG.database().databases().admin().name();
    let safe_admin_name = escape_pg_identifier(admin_database_name);
    sqlx::query(AssertSqlSafe(
        format!("DROP DATABASE IF EXISTS \"{safe_admin_name}\"").as_str(),
    ))
    .execute(pool.as_ref())
    .await?;

    let tenant_database_name_prefix = CONFIG.database().databases().tenant().name_prefix();
    let safe_prefix = escape_pg_like_prefix(tenant_database_name_prefix);
    let tenants: Vec<(String,)> = sqlx::query_as(AssertSqlSafe(
        format!(
            "SELECT datname::TEXT FROM pg_database WHERE datname LIKE '{safe_prefix}%' ESCAPE '\\'"
        )
        .as_str(),
    ))
    .fetch_all(pool.as_ref())
    .await?;

    for (tenant_name,) in tenants {
        let safe_tenant_name = escape_pg_identifier(&tenant_name);
        let drop_query = format!("DROP DATABASE CASCADE IF EXISTS \"{safe_tenant_name}\"");
        sqlx::query(AssertSqlSafe(drop_query.as_str()))
            .execute(pool.as_ref())
            .await?;
    }

    Ok(())
}

#[allow(dead_code)]
async fn uefault_pool() -> Result<ConnectedDefaultPool, Error> {
    let database_url = "postgres://postgres:postgres@postgres-test:5432/postgres";
    Pool::connect(&DatabaseUri::from(Url::parse(database_url).unwrap())).await
}

#[allow(dead_code)]
async fn get_admin_pool() -> Result<Pool<ScopeAdmin, StateConnected>, Error> {
    let admin_database_name = CONFIG.database().databases().admin().name();
    let database_url =
        format!("postgres://postgres:postgres@postgres-test:5432/{admin_database_name}");
    Pool::connect(&DatabaseUri::from(Url::parse(&database_url).unwrap())).await
}

#[allow(dead_code)]
async fn get_tenant_pool(tenant_token: &str) -> Result<Pool<ScopeTenant, StateConnected>, Error> {
    let mut database_name_builder = TenantDatabaseNameConcreteBuilder::new();
    TenantDatabaseNameDirector::construct(&mut database_name_builder, tenant_token);
    let database_name = database_name_builder.tenant_database_name();
    let database_url = format!("postgres://postgres:postgres@postgres-test:5432/{database_name}");
    Pool::connect(&DatabaseUri::from(Url::parse(&database_url).unwrap())).await
}

#[allow(dead_code)]
pub async fn refresh_databases(
    pool: &ConnectedDefaultPool,
    tenant_token: &str,
) -> Result<(), Error> {
    reset_entire_database(pool).await?;
    info!("Database successfully reseted!");
    initialize_databases(pool, tenant_token).await?;
    info!("Database successfully initialized!");

    let admin_pool = get_admin_pool().await?;
    admin_pool.migrate_database().await?;
    let tenant_template_pool = get_tenant_pool(tenant_token).await?;
    tenant_template_pool.migrate_database().await?;
    info!("Database successfully refreshed!");

    Ok(())
}

pub mod tests {
    #[allow(unused_imports)]
    use serial_test::serial;
    #[allow(unused_imports)]
    use with_lifecycle::with_lifecycle;

    #[allow(unused_imports)]
    use crate::database::test_lifecycle;

    #[allow(unused_imports)]
    use super::*;

    // #[serial]
    // #[with_lifecycle(test_lifecycle)]
    // #[tokio::test]
    // async fn test_setup_postgres_database() -> Result<(), Error> {
    //     let default_pool = get_default_pool().await?;
    //     refresh_databases(&default_pool, "test_token").await?;

    //     Ok(())
    // }
}

// ── Unit tests for SQL identifier / LIKE-pattern escaping ─────────────────────

#[cfg(test)]
mod escape_tests {
    use super::{escape_pg_identifier, escape_pg_like_prefix};

    // ── escape_pg_identifier ──────────────────────────────────────────────────

    #[test]
    fn plain_name_is_unchanged() {
        assert_eq!(escape_pg_identifier("zeitrak_admin"), "zeitrak_admin");
    }

    #[test]
    fn embedded_double_quote_is_doubled() {
        // A name like `zeitrak"admin` becomes `zeitrak""admin` inside "..." delimiters.
        assert_eq!(escape_pg_identifier("zeitrak\"admin"), "zeitrak\"\"admin");
    }

    #[test]
    fn multiple_double_quotes_are_all_doubled() {
        assert_eq!(escape_pg_identifier("a\"b\"c"), "a\"\"b\"\"c");
    }

    /// Injection attempt: `" DROP DATABASE real_db; --`
    ///
    /// After escaping the `"` becomes `""`.  When embedded as `"<escaped>"` the
    /// `PostgreSQL` parser reads `""` as a single escaped double-quote character
    /// inside the identifier, not as a closing delimiter.  The rest of the
    /// injection string becomes part of the (weirdly named) identifier, not
    /// SQL that executes.
    ///
    /// We verify the invariant directly: every `"` in the input must become `""`
    /// in the output, so the quote count doubles.
    #[test]
    fn injection_via_quote_is_neutralised() {
        let malicious = "\" DROP DATABASE real_db; --";
        let escaped = escape_pg_identifier(malicious);

        let original_quotes = malicious.chars().filter(|&c| c == '"').count();
        let escaped_quotes = escaped.chars().filter(|&c| c == '"').count();

        assert_eq!(
            escaped_quotes,
            original_quotes * 2,
            "each '\"' in the original must become '\"\"' in the escaped output: {escaped:?}"
        );
    }

    // ── escape_pg_like_prefix ─────────────────────────────────────────────────

    #[test]
    fn plain_prefix_is_unchanged() {
        assert_eq!(escape_pg_like_prefix("test_zeitrak_"), "test\\_zeitrak\\_");
    }

    #[test]
    fn percent_in_prefix_is_escaped() {
        assert_eq!(escape_pg_like_prefix("foo%bar"), "foo\\%bar");
    }

    #[test]
    fn underscore_in_prefix_is_escaped() {
        assert_eq!(escape_pg_like_prefix("foo_bar"), "foo\\_bar");
    }

    #[test]
    fn backslash_in_prefix_is_escaped_first() {
        // `\` must be escaped before `%` and `_` so that the added `\` characters
        // are not themselves re-escaped.
        assert_eq!(escape_pg_like_prefix("a\\b%c_d"), "a\\\\b\\%c\\_d");
    }

    /// Injection via LIKE wildcards: `%` alone would match all databases.
    /// After escaping it becomes `\%` which matches only a literal `%`.
    #[test]
    fn wildcard_injection_is_neutralised() {
        let malicious = "%";
        let escaped = escape_pg_like_prefix(malicious);
        assert_eq!(escaped, "\\%");
    }
}
