use sea_orm_migration::prelude::*;

/// Seeds permissions that were added after the initial seed migration.
///
/// Includes `activity.delete`, `timesheet.cancel` (present in core `ALL` but not
/// previously seeded) and the new `member.manage`, `role.manage` permissions.
///
/// UUIDs follow the same deterministic scheme: 01100000-0000-7000-8000-0000000000XX.
/// The operation is idempotent: SQLite uses `INSERT OR IGNORE`, PostgreSQL uses
/// `ON CONFLICT (id) DO NOTHING`.
#[derive(DeriveMigrationName)]
pub struct Migration;

const PERMISSIONS: &[(&str, &str)] = &[
    ("01100000-0000-7000-8000-00000000000d", "activity.delete"),
    ("01100000-0000-7000-8000-00000000000e", "timesheet.cancel"),
    ("01100000-0000-7000-8000-00000000000f", "member.manage"),
    ("01100000-0000-7000-8000-000000000010", "role.manage"),
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
