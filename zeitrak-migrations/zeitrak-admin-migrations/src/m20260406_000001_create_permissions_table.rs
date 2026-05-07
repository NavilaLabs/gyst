use sea_orm_migration::{
    prelude::*,
    schema::{pk_uuid, string},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("permissions")
                    .if_not_exists()
                    .col(pk_uuid("id"))
                    .col(string("name"))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("permissions").to_owned())
            .await
    }
}
