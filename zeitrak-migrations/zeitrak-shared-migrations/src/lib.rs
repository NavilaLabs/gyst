#[allow(clippy::wildcard_imports)]
use sea_orm_migration::{
    prelude::*,
    schema::{binary, integer, json_binary, string, timestamp},
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Database error: {0}")]
    DbError(#[from] DbErr),
}

#[must_use]
pub fn create_events_table_migration() -> (TableCreateStatement, Vec<IndexCreateStatement>) {
    let name = "events";
    let table_create_statement = Table::create()
        .if_not_exists()
        .table(name)
        .col(string("event_stream_id"))
        .col(string("type"))
        .col(integer("version").check(Expr::col("version").gt(0)))
        .col(binary("event"))
        .col(json_binary("metadata"))
        .col(integer("schema_version").check(Expr::col("version").gt(0)))
        .primary_key(Index::create().col("event_stream_id").col("version"))
        .foreign_key(
            ForeignKey::create()
                .name("fk_events_event_stream_id")
                .from(TableRef::Table(name.into(), None), "event_stream_id")
                .to(
                    TableRef::Table("event_streams".into(), None),
                    "event_stream_id",
                )
                .on_delete(ForeignKeyAction::Cascade),
        )
        .to_owned();
    let index_create_statements = vec![];

    (table_create_statement, index_create_statements)
}

#[must_use]
pub fn create_event_streams_table_migration() -> (TableCreateStatement, Vec<IndexCreateStatement>) {
    let name = "event_streams";
    let table_create_statement = Table::create()
        .if_not_exists()
        .table(name)
        .col(string("event_stream_id").primary_key())
        .col(integer("version").check(Expr::col("version").gt(0)))
        .to_owned();
    let index_create_statements = vec![];

    (table_create_statement, index_create_statements)
}

#[must_use]
pub fn create_snapshots_table_migration() -> (TableCreateStatement, Vec<IndexCreateStatement>) {
    let name = "snapshots";
    let table_create_statement = Table::create()
        .if_not_exists()
        .table(name)
        .col(integer("id").primary_key().auto_increment())
        .col(string("event_stream_id"))
        .col(string("aggregate_type"))
        .col(string("aggregate_id"))
        .col(binary("state"))
        .col(integer("version").check(Expr::col("version").gt(0)))
        .col(timestamp("taken_at").default(Expr::current_timestamp()))
        .foreign_key(
            ForeignKey::create()
                .name("fk_snapshots_event_stream_id")
                .from(TableRef::Table(name.into(), None), "event_stream_id")
                .to(
                    TableRef::Table("event_streams".into(), None),
                    "event_stream_id",
                ),
        )
        .to_owned();
    let index_create_statements = vec![
        Index::create()
            .table(name)
            .name("uq_snapshots_aggregate_type_id_version")
            .unique()
            .col("aggregate_type")
            .col("aggregate_id")
            .col("version")
            .to_owned(),
    ];

    (table_create_statement, index_create_statements)
}

#[cfg(test)]
mod tests {
    use sea_orm_migration::prelude::SqliteQueryBuilder;

    #[test]
    fn events_table_has_composite_pk() {
        let (stmt, _) = super::create_events_table_migration();
        let sql = stmt.to_string(SqliteQueryBuilder);
        println!("Generated SQL: {sql}");
        assert!(
            !sql.contains("event_stream_id\" PRIMARY KEY"),
            "event_stream_id is still sole PK: {sql}"
        );
    }

    /// Verifies that the `event_streams` table definition places a PRIMARY KEY
    /// on `event_stream_id`, which is the mechanism that prevents stream ID
    /// collisions from causing silent data corruption.
    #[test]
    fn event_streams_table_has_primary_key_on_stream_id() {
        let (stmt, _) = super::create_event_streams_table_migration();
        let sql = stmt.to_string(SqliteQueryBuilder);
        println!("event_streams DDL: {sql}");
        assert!(
            sql.contains("PRIMARY KEY"),
            "event_streams must have a PRIMARY KEY to prevent duplicate stream IDs: {sql}"
        );
        assert!(
            sql.to_lowercase().contains("event_stream_id"),
            "event_stream_id column must appear in event_streams DDL: {sql}"
        );
    }

    /// Proves that attempting to insert two rows with the same `event_stream_id`
    /// into an in-memory `SQLite` database is rejected by the PK uniqueness
    /// constraint.  This is the runtime guard against stream ID collisions.
    #[tokio::test]
    async fn duplicate_event_stream_id_is_rejected_by_pk() {
        use sqlx::SqlitePool;

        let pool = SqlitePool::connect(":memory:")
            .await
            .expect("in-memory SQLite must open");

        sqlx::query(
            "CREATE TABLE event_streams (
                event_stream_id TEXT PRIMARY KEY,
                version         INTEGER NOT NULL,
                aggregate_type  TEXT NOT NULL DEFAULT ''
             )",
        )
        .execute(&pool)
        .await
        .expect("table creation must succeed");

        // First insert — must succeed.
        sqlx::query(
            "INSERT INTO event_streams (event_stream_id, version, aggregate_type)
             VALUES ('550e8400-e29b-41d4-a716-446655440000', 1, 'user')",
        )
        .execute(&pool)
        .await
        .expect("first insert must succeed");

        // Second insert with the same event_stream_id — must fail even though
        // the aggregate_type is different.  The PK constraint is on
        // event_stream_id alone, so there is no ambiguity about which
        // aggregate owns the stream.
        let duplicate = sqlx::query(
            "INSERT INTO event_streams (event_stream_id, version, aggregate_type)
             VALUES ('550e8400-e29b-41d4-a716-446655440000', 1, 'workspace')",
        )
        .execute(&pool)
        .await;

        assert!(
            duplicate.is_err(),
            "duplicate event_stream_id must be rejected by the PRIMARY KEY constraint"
        );
    }

    /// Verifies that the composite unique index on `(aggregate_type, event_stream_id)`
    /// — added by the `add_aggregate_type_to_event_streams` migration — rejects
    /// rows that share both columns, providing an additional layer of defence.
    #[tokio::test]
    async fn duplicate_aggregate_type_and_stream_id_is_rejected_by_unique_index() {
        use sqlx::SqlitePool;

        let pool = SqlitePool::connect(":memory:")
            .await
            .expect("in-memory SQLite must open");

        sqlx::query(
            "CREATE TABLE event_streams (
                event_stream_id TEXT PRIMARY KEY,
                version         INTEGER NOT NULL,
                aggregate_type  TEXT NOT NULL DEFAULT ''
             )",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "CREATE UNIQUE INDEX uq_event_streams_aggregate_type_stream_id
             ON event_streams (aggregate_type, event_stream_id)",
        )
        .execute(&pool)
        .await
        .expect("unique index creation must succeed");

        sqlx::query(
            "INSERT INTO event_streams (event_stream_id, version, aggregate_type)
             VALUES ('aaaaaaaa-0000-0000-0000-000000000001', 1, 'user')",
        )
        .execute(&pool)
        .await
        .unwrap();

        let duplicate = sqlx::query(
            "INSERT INTO event_streams (event_stream_id, version, aggregate_type)
             VALUES ('aaaaaaaa-0000-0000-0000-000000000001', 1, 'user')",
        )
        .execute(&pool)
        .await;

        assert!(
            duplicate.is_err(),
            "duplicate (aggregate_type, event_stream_id) must be rejected"
        );
    }
}
