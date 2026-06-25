use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("projections__invitations"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("id")).string().not_null().primary_key())
                    .col(ColumnDef::new(Alias::new("workspace_id")).string().not_null())
                    .col(ColumnDef::new(Alias::new("invited_by")).string().not_null())
                    .col(ColumnDef::new(Alias::new("email")).string().not_null())
                    .col(ColumnDef::new(Alias::new("workspace_role_id")).string().not_null())
                    .col(
                        ColumnDef::new(Alias::new("token"))
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("status"))
                            .string()
                            .not_null()
                            .default("pending"),
                    )
                    .col(ColumnDef::new(Alias::new("expires_at")).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                Alias::new("projections__invitations"),
                                Alias::new("workspace_id"),
                            )
                            .to(Alias::new("projections__workspaces"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                Alias::new("projections__invitations"),
                                Alias::new("invited_by"),
                            )
                            .to(Alias::new("projections__users"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                Alias::new("projections__invitations"),
                                Alias::new("workspace_role_id"),
                            )
                            .to(Alias::new("projections__workspace_roles"), Alias::new("id")),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_invitations_token")
                    .table(Alias::new("projections__invitations"))
                    .col(Alias::new("token"))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_invitations_email")
                    .table(Alias::new("projections__invitations"))
                    .col(Alias::new("email"))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(Alias::new("projections__invitations"))
                    .to_owned(),
            )
            .await
    }
}
