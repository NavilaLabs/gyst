use std::sync::Arc;

use crate::{commands, ports};

pub struct Connection {
    port: Arc<dyn ports::database::Connection>,
}

impl<Pool> super::Handler<Pool, commands::database::Connection> for Connection
where
    Pool: ports::database::Connection,
{
    type Error = ports::database::Error;

    async fn handle(
        &self,
        pool: &Pool,
        command: commands::database::Connection,
    ) -> Result<(), Self::Error> {
        match command {
            commands::database::Connection::Disconnect => {
                self.port.close_connection(pool).await;
            }
        }
        Ok(())
    }
}
