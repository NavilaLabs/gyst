use sea_orm_migration::{
    prelude::*,
    schema::{pk_uuid, string_null},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("projections__workspaces")
                    .if_not_exists()
                    .col(pk_uuid("id"))
                    .col(string_null("name"))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("projections__workspaces").to_owned())
            .await
    }
}
