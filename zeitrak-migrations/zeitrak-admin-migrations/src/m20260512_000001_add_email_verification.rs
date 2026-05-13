use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_database_backend();
        let conn = manager.get_connection();

        // SQLite does not support IF NOT EXISTS on ADD COLUMN; swallow duplicate errors.
        if db == sea_orm::DatabaseBackend::Sqlite {
            let _ = conn
                .execute_unprepared(
                    "ALTER TABLE projections__users ADD COLUMN verification_token TEXT",
                )
                .await;
            let _ = conn
                .execute_unprepared(
                    "ALTER TABLE projections__users ADD COLUMN is_verified INTEGER NOT NULL DEFAULT 0",
                )
                .await;
        } else {
            conn.execute_unprepared(
                "ALTER TABLE projections__users ADD COLUMN IF NOT EXISTS verification_token TEXT",
            )
            .await?;
            conn.execute_unprepared(
                "ALTER TABLE projections__users ADD COLUMN IF NOT EXISTS is_verified BOOLEAN NOT NULL DEFAULT FALSE",
            )
            .await?;
        }

        // Index on verification_token for fast lookups.
        let _ = conn
            .execute_unprepared(
                "CREATE INDEX IF NOT EXISTS idx_users_verification_token ON projections__users (verification_token)",
            )
            .await;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_database_backend();
        let conn = manager.get_connection();

        // SQLite 3.35+ supports DROP COLUMN but not DROP INDEX via ALTER TABLE in sea-query.
        let _ = conn
            .execute_unprepared("DROP INDEX IF EXISTS idx_users_verification_token")
            .await;

        if db != sea_orm::DatabaseBackend::Sqlite {
            conn.execute_unprepared(
                "ALTER TABLE projections__users DROP COLUMN IF EXISTS verification_token",
            )
            .await?;
            conn.execute_unprepared(
                "ALTER TABLE projections__users DROP COLUMN IF EXISTS is_verified",
            )
            .await?;
        }

        Ok(())
    }
}
