use sea_orm_migration::{
    prelude::*,
    schema::{pk_uuid, string, uuid},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("projections__timesheet_tags")
                    .if_not_exists()
                    .col(pk_uuid("id"))
                    .col(string("name").unique_key())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table("projections__timesheet_has_tags")
                    .if_not_exists()
                    .col(uuid("timesheet_id"))
                    .col(uuid("timesheet_tag_id"))
                    .primary_key(Index::create().col("timesheet_id").col("timesheet_tag_id"))
                    // No FK on timesheet_id: timesheets may not yet be in the projection table
                    // when a tag event arrives; referential integrity is maintained at the app level.
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_timesheet_has_tags_tag_id")
                            .from(
                                TableRef::Table("projections__timesheet_has_tags".into(), None),
                                "timesheet_tag_id",
                            )
                            .to(
                                TableRef::Table("projections__timesheet_tags".into(), None),
                                "id",
                            )
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table("projections__timesheet_has_tags")
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table("projections__timesheet_tags")
                    .to_owned(),
            )
            .await
    }
}
