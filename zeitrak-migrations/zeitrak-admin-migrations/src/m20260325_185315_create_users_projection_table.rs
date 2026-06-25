use sea_orm_migration::{prelude::*, schema::string};

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
                    .col(ColumnDef::new(Alias::new("id")).string().not_null().primary_key())
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
