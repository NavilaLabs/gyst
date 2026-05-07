use sea_orm_migration::prelude::*;

/// Adds workspace-level settings columns to `projections__workspaces`.
///
/// `SQLite` swallows duplicate-column errors so the migration is idempotent.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_database_backend();
        let conn = manager.get_connection();

        for (col, default) in [
            ("timezone", "Europe/Berlin"),
            ("date_format", "%Y-%m-%d"),
            ("currency", "EUR"),
            ("week_start", "monday"),
        ] {
            let sql = if db == sea_orm::DatabaseBackend::Sqlite {
                format!(
                    "ALTER TABLE projections__workspaces ADD COLUMN {col} TEXT NOT NULL DEFAULT '{default}'"
                )
            } else {
                format!(
                    "ALTER TABLE projections__workspaces ADD COLUMN IF NOT EXISTS {col} TEXT NOT NULL DEFAULT '{default}'"
                )
            };
            let _ = conn.execute_unprepared(&sql).await;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_database_backend();
        if db != sea_orm::DatabaseBackend::Sqlite {
            let conn = manager.get_connection();
            for col in ["timezone", "date_format", "currency", "week_start"] {
                conn.execute_unprepared(&format!(
                    "ALTER TABLE projections__workspaces DROP COLUMN IF EXISTS {col}"
                ))
                .await?;
            }
        }
        Ok(())
    }
}
