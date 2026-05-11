pub use sea_orm_migration::prelude::*;
pub use zeitrak_shared_migrations::Error;

mod m20251214_102154_create_event_streams_table;
mod m20251214_102155_create_events_table;
mod m20251215_183826_create_snapshots_table;
mod m20260325_145345_add_global_position;
mod m20260408_000001_create_activities_projection_tables;
mod m20260408_000002_create_timesheets_projection_table;
mod m20260408_000003_create_timesheet_tags_projection_table;
mod m20260416_000001_add_soft_delete_columns;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20251214_102154_create_event_streams_table::Migration),
            Box::new(m20251214_102155_create_events_table::Migration),
            Box::new(m20251215_183826_create_snapshots_table::Migration),
            Box::new(m20260325_145345_add_global_position::Migration),
            Box::new(m20260408_000001_create_activities_projection_tables::Migration),
            Box::new(m20260408_000002_create_timesheets_projection_table::Migration),
            Box::new(m20260408_000003_create_timesheet_tags_projection_table::Migration),
            Box::new(m20260416_000001_add_soft_delete_columns::Migration),
        ]
    }
}
