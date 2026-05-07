use sea_orm_migration::prelude::*;

/// Adds user-specific settings columns to `projections__users`.
///
/// Defaults are provided so the migration is safe to apply to existing rows.
/// `SQLite` ignores `IF NOT EXISTS` on `ALTER TABLE ADD COLUMN` — errors are
/// intentionally swallowed so re-running the migration is idempotent.
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
            ("language", "en"),
        ] {
            let sql = if db == sea_orm::DatabaseBackend::Sqlite {
                // SQLite does not support IF NOT EXISTS; swallow duplicate-column errors.
                format!(
                    "ALTER TABLE projections__users ADD COLUMN {col} TEXT NOT NULL DEFAULT '{default}'"
                )
            } else {
                format!(
                    "ALTER TABLE projections__users ADD COLUMN IF NOT EXISTS {col} TEXT NOT NULL DEFAULT '{default}'"
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
            for col in ["timezone", "date_format", "language"] {
                conn.execute_unprepared(&format!(
                    "ALTER TABLE projections__users DROP COLUMN IF EXISTS {col}"
                ))
                .await?;
            }
        }
        Ok(())
    }
}
