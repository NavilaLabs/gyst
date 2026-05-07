use sea_orm_migration::prelude::*;

/// Seeds the canonical permission records into the `permissions` table.
///
/// UUIDs are fixed and deterministic so the migration is fully idempotent.
/// Each entry maps a UUID to its `<aggregate>.<action>` permission name.
///
/// The operation is safe to re-run: `SQLite` uses `INSERT OR IGNORE`, and
/// `PostgreSQL` uses `ON CONFLICT (id) DO NOTHING`.
#[derive(DeriveMigrationName)]
pub struct Migration;

// Fixed deterministic IDs for each permission.
// Format: 01100000-0000-7000-8000-0000000000XX (XX = ordinal in hex)
const PERMISSIONS: &[(&str, &str)] = &[
    ("01100000-0000-7000-8000-000000000001", "customer.create"),
    ("01100000-0000-7000-8000-000000000002", "customer.update"),
    ("01100000-0000-7000-8000-000000000003", "project.create"),
    ("01100000-0000-7000-8000-000000000004", "project.update"),
    ("01100000-0000-7000-8000-000000000005", "activity.create"),
    ("01100000-0000-7000-8000-000000000006", "activity.update"),
    ("01100000-0000-7000-8000-000000000007", "timesheet.create"),
    ("01100000-0000-7000-8000-000000000008", "timesheet.update"),
    ("01100000-0000-7000-8000-000000000009", "timesheet.export"),
    ("01100000-0000-7000-8000-00000000000a", "tag.manage"),
    ("01100000-0000-7000-8000-00000000000b", "rate.manage"),
];

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_database_backend();
        let conn = manager.get_connection();

        for (id, name) in PERMISSIONS {
            let sql = if db == sea_orm::DatabaseBackend::Sqlite {
                format!("INSERT OR IGNORE INTO permissions (id, name) VALUES ('{id}', '{name}')")
            } else {
                format!(
                    "INSERT INTO permissions (id, name) VALUES ('{id}', '{name}') \
                     ON CONFLICT (id) DO NOTHING"
                )
            };
            conn.execute_unprepared(&sql).await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        for (id, _) in PERMISSIONS {
            conn.execute_unprepared(&format!("DELETE FROM permissions WHERE id = '{id}'"))
                .await?;
        }

        Ok(())
    }
}
