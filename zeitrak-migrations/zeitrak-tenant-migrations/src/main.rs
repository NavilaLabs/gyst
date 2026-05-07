use sea_orm_migration::prelude::*;

#[tokio::main]
async fn main() {
    cli::run_cli(zeitrak_tenant_migrations::Migrator).await;
}
