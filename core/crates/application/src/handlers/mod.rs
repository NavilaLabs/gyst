pub mod database;

pub trait Handler<Pool, Command>
where
    Pool: Send + Sync,
{
    type Error;

    async fn handle(&self, pool: &Pool, command: Command) -> Result<(), Self::Error>;
}
