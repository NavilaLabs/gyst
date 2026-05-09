use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let (table_create_statement, index_create_statements) =
            zeitrak_shared_migrations::create_events_table_migration();
        manager.create_table(table_create_statement).await?;

        for statement in index_create_statements {
            manager.create_index(statement).await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("events").to_owned())
            .await
    }
}
