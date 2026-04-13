use std::sync::Arc;

use anyhow::Result;
use loom_core::admin::user::{UserCommand, UserId, UserQuery, UserRepository};

pub struct UserController<R: UserRepository, P: Send + Sync> {
    repository: Arc<R>,
    commands: Arc<UserCommand>,
    queries: Arc<UserQuery<P>>, // TODO: this is not a query root
}

impl<R: UserRepository, P: Send + Sync> UserController<R, P> {
    pub const fn new(
        repository: Arc<R>,
        commands: Arc<UserCommand>,
        queries: Arc<UserQuery<P>>,
    ) -> Self {
        Self {
            repository,
            commands,
            queries,
        }
    }

    pub async fn create_user(
        &self,
        id: UserId,
        name: String,
        email: String,
        password: String,
    ) -> Result<()> {
        let mut root = self.commands.create(id, name, email, password)?;
        self.repository.save(&mut root).await?;

        Ok(())
    }
}
