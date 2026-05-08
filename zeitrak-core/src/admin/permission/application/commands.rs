use std::fmt::Debug;

use async_trait::async_trait;
use eventually::aggregate::Root;

use crate::admin::permission::{
    domain::{
        aggregates::{Permission, PermissionId},
        events::PermissionEvent,
        interfaces::PermissionRepository,
    },
};

#[async_trait]
pub trait PermissionCommandTrait<R> {
    type Error: Debug + Sync + Send;

    async fn create(&self, id: PermissionId, name: String) -> Result<Root<Permission>, Self::Error>;
}

#[derive(Debug)]
pub struct PermissionCommand<Repo> {
    repository: Repo,
}

impl<Repo> PermissionCommand<Repo> {
    pub fn new(repository: Repo) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<Repo, R> PermissionCommandTrait<R> for PermissionCommand<Repo>
where
    R: Debug,
    Repo: Debug + PermissionRepository<R>,
{
    type Error = crate::Error<Repo, Permission, R>;

    /// # Errors
    ///
    /// Returns an error if the domain event cannot be applied or the root cannot be saved.
    async fn create(
        &self,
        id: PermissionId,
        name: String,
    ) -> Result<Root<Permission>, <Self as PermissionCommandTrait<R>>::Error> {
        let mut root = Root::<Permission>::record_new(
            PermissionEvent::Created { id, name }.into(),
        )?;
        self.repository
            .save(&mut root)
            .await
            .map_err(|e| crate::Error::WriteRepositoryError(e.into()))?;
        Ok(root)
    }
}

#[cfg(test)]
mod tests {
    use crate::admin::permission::domain::interfaces::in_memory_repository::InMemoryPermissionRepository;

    use super::*;

    #[tokio::test]
    async fn create_returns_root_with_applied_state() {
        let id: PermissionId = "019d0ce8-facb-7c90-b9d7-287ae4f17c92"
            .parse()
            .expect("valid UUID");

        let result = PermissionCommand::new(InMemoryPermissionRepository::new())
            .create(id.clone(), "can_invite_users".to_string())
            .await;

        assert!(result.is_ok());
        let root = result.unwrap();
        assert_eq!(root.aggregate_id(), &id);
        assert_eq!(root.name(), "can_invite_users");
        assert_eq!(root.version(), 1);
    }
}
