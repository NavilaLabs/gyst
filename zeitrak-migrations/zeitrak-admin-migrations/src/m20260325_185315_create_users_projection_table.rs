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
                    .table("projections__users")
                    .if_not_exists()
                    .col(pk_uuid("id"))
                    .col(string("name"))
                    .col(string("email"))
                    .col(string("password"))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("projections__users").to_owned())
            .await
    }
}
