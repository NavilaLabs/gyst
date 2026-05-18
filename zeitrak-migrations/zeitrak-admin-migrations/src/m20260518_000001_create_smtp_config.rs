use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        conn.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS smtp_config (
                id                       INTEGER PRIMARY KEY CHECK (id = 1),
                auth_method              TEXT NOT NULL DEFAULT 'password',
                host                     TEXT NOT NULL DEFAULT '',
                port                     INTEGER NOT NULL DEFAULT 587,
                username                 TEXT NOT NULL DEFAULT '',
                from_address             TEXT NOT NULL DEFAULT '',
                use_tls                  INTEGER NOT NULL DEFAULT 1,
                encrypted_password       TEXT,
                password_nonce           TEXT,
                oauth2_client_id         TEXT,
                oauth2_tenant_id         TEXT,
                oauth2_smtp_email        TEXT,
                encrypted_client_secret  TEXT,
                client_secret_nonce      TEXT,
                encrypted_refresh_token  TEXT,
                refresh_token_nonce      TEXT,
                oauth2_state             TEXT,
                oauth2_authorized        INTEGER NOT NULL DEFAULT 0,
                updated_at               TEXT NOT NULL
            )",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS smtp_config")
            .await?;

        Ok(())
    }
}
