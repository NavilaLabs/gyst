use crate::{commands, ports};

pub struct Migrate<T: ports::database::Migrate> {
    port: T,
}

impl<T: ports::database::Migrate> Migrate<T> {
    pub fn new(port: T) -> Self {
        Self { port }
    }

    pub async fn handle<P: Send + Sync, C: infrastructure::database::Connection<P>>(
        &self,
        pool: &C,
        command: commands::database::Database,
    ) -> Result<(), <T as ports::database::Migrate>::Error> {
        match command {
            commands::database::Database::RunMigrations(command) => {
                self.port.run(pool, command).await
            }
        }
    }
}
