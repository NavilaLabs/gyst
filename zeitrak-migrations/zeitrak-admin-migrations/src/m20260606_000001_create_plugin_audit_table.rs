use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_database_backend();
        let conn = manager.get_connection();

        let create_sql = if db == sea_orm::DatabaseBackend::Sqlite {
            "CREATE TABLE IF NOT EXISTS plugin_audit (
                id            INTEGER PRIMARY KEY AUTOINCREMENT,
                occurred_at   TEXT    NOT NULL,
                plugin_id     TEXT    NOT NULL,
                function_name TEXT    NOT NULL,
                user_id       TEXT,
                workspace_id  TEXT,
                trust_tier    TEXT    NOT NULL,
                outcome       TEXT    NOT NULL,
                error_message TEXT,
                duration_ms   INTEGER
            )"
        } else {
            "CREATE TABLE IF NOT EXISTS plugin_audit (
                id            SERIAL  PRIMARY KEY,
                occurred_at   TEXT    NOT NULL,
                plugin_id     TEXT    NOT NULL,
                function_name TEXT    NOT NULL,
                user_id       TEXT,
                workspace_id  TEXT,
                trust_tier    TEXT    NOT NULL,
                outcome       TEXT    NOT NULL,
                error_message TEXT,
                duration_ms   INTEGER
            )"
        };
        conn.execute_unprepared(create_sql).await?;

        conn.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_plugin_audit_plugin_id
             ON plugin_audit (plugin_id)",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_plugin_audit_occurred_at
             ON plugin_audit (occurred_at)",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS plugin_audit")
            .await?;

        Ok(())
    }
}
