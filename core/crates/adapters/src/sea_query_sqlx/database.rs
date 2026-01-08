use application::ports;
use async_trait::async_trait;

#[derive(Debug, thiserror::Error)]
pub enum Error {}

#[async_trait]
impl<T> ports::database::Migrate for T {
    type Error = Error;

    async fn run(&self) -> Result<(), Error> {
        Ok(())
    }
}
