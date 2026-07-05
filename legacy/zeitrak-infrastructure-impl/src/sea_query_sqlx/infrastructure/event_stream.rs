use sea_query::{Alias, Expr, ExprTrait};
use sqlx::{AssertSqlSafe, Row as _};

use crate::sea_query_sqlx::infrastructure::pool::ConnectionProvider;

/// Returns the current event-stream version for the given aggregate stream ID.
///
/// Queries the `event_streams` table directly so that projection-based
/// repository reads can stamp the correct version onto the rehydrated
/// [`eventually::aggregate::Root`].
///
/// Returns `0` if no stream row exists yet (aggregate has never been written).
///
/// # Errors
///
/// Returns an error if the database query fails.
pub async fn current_stream_version(
    provider: &impl ConnectionProvider,
    stream_id: &str,
) -> Result<u64, crate::Error> {
    let stmt = sea_query::Query::select()
        .expr(Expr::col(Alias::new("version")))
        .from(Alias::new("event_streams"))
        .and_where(Expr::col(Alias::new("event_stream_id")).eq(stream_id))
        .to_owned();
    let (sql, arguments) = provider.build_query(&stmt);
    let row = sqlx::query_with(AssertSqlSafe(sql.as_str()), arguments)
        .fetch_optional(provider.any_pool())
        .await?;
    Ok(row.map_or(0, |r| {
        r.try_get::<i64, _>(0usize).map_or(0, i64::cast_unsigned)
    }))
}
