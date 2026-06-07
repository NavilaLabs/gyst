use sea_orm_migration::prelude::*;

/// Seeds the `admin.bypass` permission (UUID `0x11`).
///
/// This permission replaces the hardcoded `"admin"` role-name check in the
/// authorization service.  Any workspace role that carries this permission
/// gains unconditional access to every operation.
///
/// The operation is idempotent: `SQLite` uses `INSERT OR IGNORE`, `PostgreSQL`
/// uses `ON CONFLICT (id) DO NOTHING`.
#[derive(DeriveMigrationName)]
pub struct Migration;

const ID: &str = "01100000-0000-7000-8000-000000000011";
const NAME: &str = "admin.bypass";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_database_backend();
        let conn = manager.get_connection();
        let sql = if db == sea_orm::DatabaseBackend::Sqlite {
            format!("INSERT OR IGNORE INTO permissions (id, name) VALUES ('{ID}', '{NAME}')")
        } else {
            format!(
                "INSERT INTO permissions (id, name) VALUES ('{ID}', '{NAME}') \
                 ON CONFLICT (id) DO NOTHING"
            )
        };
        conn.execute_unprepared(&sql).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(&format!("DELETE FROM permissions WHERE id = '{ID}'"))
            .await?;
        Ok(())
    }
}
