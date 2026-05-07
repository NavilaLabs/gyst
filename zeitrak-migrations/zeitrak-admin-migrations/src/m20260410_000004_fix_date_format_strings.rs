use sea_orm_migration::prelude::*;

/// Fixes rows that have the old-style `YYYY-MM-DD` date-format value instead of
/// the chrono-compatible `%Y-%m-%d`.  This can happen when the settings columns
/// were added with the wrong default or when the value was stored before the
/// format-string convention was finalised.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        conn.execute_unprepared(
            "UPDATE projections__users \
             SET date_format = '%Y-%m-%d' \
             WHERE date_format = 'YYYY-MM-DD'",
        )
        .await?;

        conn.execute_unprepared(
            "UPDATE projections__workspaces \
             SET date_format = '%Y-%m-%d' \
             WHERE date_format = 'YYYY-MM-DD'",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // No meaningful rollback — we do not want to restore the broken values.
        Ok(())
    }
}
