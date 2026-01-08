use std::fmt::Display;

use async_trait::async_trait;

use crate::commands;

#[async_trait]
pub trait Migrate {
    type Error: Display + Send + Sync;

    async fn run<P: Send + Sync, T: infrastructure::database::Connection<P>>(
        &self,
        pool: &T,
        command: commands::database::RunMigrations,
    ) -> Result<(), Self::Error>;
}
