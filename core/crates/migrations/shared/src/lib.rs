use sea_orm_migration::{prelude::*, schema::*};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Database error: {0}")]
    DbError(#[from] DbErr),
}

#[must_use]
pub fn create_events_table_migration(
    name: Option<&'static str>,
) -> (TableCreateStatement, Vec<IndexCreateStatement>) {
    let name = name.unwrap_or("events");
    let table_create_statement = Table::create()
        .if_not_exists()
        .table(name)
        // event columns
        .col(uuid("event_id").primary_key())
        .col(string("event_type"))
        .col(integer("event_version"))
        // aggregate columns
        .col(string("aggregate_type"))
        .col(uuid("aggregate_id"))
        .col(integer("aggregate_version"))
        // data columns
        .col(json_binary("data"))
        .col(json_binary_null("metadata"))
        // timestamp columns
        .col(timestamp("created_at").default(Expr::current_timestamp()))
        .col(timestamp_null("effective_at"))
        // user columns
        .col(uuid("created_by"))
        .col(uuid_null("owned_by"))
        // tracing columns
        .col(uuid_null("correlation_id"))
        .col(uuid_null("causation_id"))
        // integrity columns
        .col(binary("hash"))
        .col(binary("previous_hash"))
        .to_owned();
    let index_create_statements = vec![
        Index::create()
            .table(name)
            .name("uq_events_aggregate_type_id_version")
            .unique()
            .col("aggregate_type")
            .col("aggregate_id")
            .col("aggregate_version")
            .to_owned(),
        Index::create()
            .table(name)
            .name("uq_events_hash")
            .unique()
            .col("hash")
            .to_owned(),
        Index::create()
            .table(name)
            .name("uq_events_previous_hash")
            .unique()
            .col("previous_hash")
            .to_owned(),
        Index::create()
            .table(name)
            .name("idx_events_correlation_id")
            .col("correlation_id")
            .to_owned(),
        Index::create()
            .table(name)
            .name("idx_events_causation_id")
            .col("causation_id")
            .to_owned(),
        Index::create()
            .table(name)
            .name("idx_events_event_type")
            .col("event_type")
            .to_owned(),
        Index::create()
            .table(name)
            .name("idx_events_event_type_version")
            .col("event_type")
            .col("event_version")
            .to_owned(),
    ];

    (table_create_statement, index_create_statements)
}

#[must_use]
pub fn create_snapshots_table_migration(
    name: Option<&'static str>,
) -> (TableCreateStatement, Vec<IndexCreateStatement>) {
    let name = name.unwrap_or("snapshots");
    let table_create_statement = Table::create()
        .if_not_exists()
        .table(name)
        // snapshot columns
        .col(uuid("snapshot_id").primary_key())
        .col(string("aggregate_type"))
        .col(uuid("aggregate_id"))
        .col(integer("aggregate_version"))
        .col(json_binary("data"))
        .col(json_binary("metadata"))
        .col(timestamp("created_at").default(Expr::current_timestamp()))
        .to_owned();
    let index_create_statements = vec![
        Index::create()
            .table(name)
            .name("uq_snapshots_aggregate_type_id_version")
            .unique()
            .col("aggregate_type")
            .col("aggregate_id")
            .col("aggregate_version")
            .to_owned(),
        Index::create()
            .table(name)
            .name("idx_snapshots_lookup")
            .col("aggregate_type")
            .col("aggregate_id")
            .to_owned(),
        Index::create()
            .table(name)
            .name("idx_snapshots_created_at")
            .col("created_at")
            .to_owned(),
    ];

    (table_create_statement, index_create_statements)
}
