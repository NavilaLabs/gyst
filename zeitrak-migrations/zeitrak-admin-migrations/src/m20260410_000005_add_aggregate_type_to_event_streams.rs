use sea_orm_migration::prelude::*;

/// Adds an `aggregate_type` column to `event_streams` and a unique index on
/// `(aggregate_type, event_stream_id)`.
///
/// **Why:** The original schema used `event_stream_id` as a bare primary key,
/// which is globally unique but does not encode which aggregate type owns the
/// stream.  Two different aggregate types that coincidentally generated the same
/// UUID would collide on the PK and produce a hard error (not silent
/// corruption), but there was no way to inspect the table and know whether a
/// given stream belonged to a `User`, a `Workspace`, etc.
///
/// Adding `aggregate_type` provides:
/// 1. Observability — each row is self-describing.
/// 2. Defence-in-depth — the unique index on `(aggregate_type, event_stream_id)`
///    means that even if the PK constraint were somehow relaxed in future, the
///    per-type uniqueness guarantee remains.
/// 3. A migration path toward making the composite `(aggregate_type,
///    event_stream_id)` the canonical identifier if the schema ever needs to
///    allow the same UUID to appear under two different aggregate types.
///
/// Existing rows are assigned the empty string as their `aggregate_type` so
/// that the `NOT NULL` constraint is satisfied without requiring a full backfill.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // Add the column with a safe default so the migration is non-destructive
        // on existing rows.
        conn.execute_unprepared(
            "ALTER TABLE event_streams ADD COLUMN aggregate_type TEXT NOT NULL DEFAULT ''",
        )
        .await?;

        // Unique index on (aggregate_type, event_stream_id) as defence-in-depth.
        // Since event_stream_id is already the PRIMARY KEY this does not change
        // the effective uniqueness guarantee for existing rows (all have
        // aggregate_type = ''), but it anchors the constraint explicitly and
        // will enforce type-scoped uniqueness once aggregate_type is populated.
        manager
            .create_index(
                Index::create()
                    .table("event_streams")
                    .name("uq_event_streams_aggregate_type_stream_id")
                    .unique()
                    .col("aggregate_type")
                    .col("event_stream_id")
                    .if_not_exists()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("uq_event_streams_aggregate_type_stream_id")
                    .table("event_streams")
                    .to_owned(),
            )
            .await?;

        // SQLite does not support DROP COLUMN before version 3.35; use
        // execute_unprepared so the statement is passed through as-is.
        // For Postgres this is also fine.
        manager
            .get_connection()
            .execute_unprepared("ALTER TABLE event_streams DROP COLUMN aggregate_type")
            .await?;

        Ok(())
    }
}
