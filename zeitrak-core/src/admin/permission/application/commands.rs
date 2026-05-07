use std::fmt::Debug;

use async_trait::async_trait;
use eventually::aggregate::Root;

use crate::admin::permission::{
    self, PermissionRepository,
    application::PermissionRoot,
    domain::{
        aggregates::{Permission, PermissionId},
        events::PermissionEvent,
    },
};

#[async_trait]
pub trait PermissionCommandTrait<T> {
    type Error: Debug + Sync + Send;

    async fn create(&self, id: PermissionId, name: String) -> Result<T, Self::Error>;
}

#[derive(Debug)]
pub struct PermissionCommand<R> {
    repository: R,
}

#[async_trait]
impl<R> PermissionCommandTrait<PermissionRoot> for PermissionCommand<R>
where
    R: Debug + PermissionRepository,
{
    type Error = crate::Error<R, Permission>;

    async fn create(&self, id: PermissionId, name: String) -> Result<PermissionRoot, Self::Error> {
        Ok(
            Root::<Permission>::record_new(PermissionEvent::Created { id, name }.into())
                .map_err(|_| permission::Error::AlreadyExists)?
                .into(),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::admin::permission::domain::interfaces::in_memory_repository::InMemoryPermissionRepository;

    use super::*;

    fn test_id() -> PermissionId {
        "019d0ce8-facb-7c90-b9d7-287ae4f17c92"
            .parse()
            .expect("valid UUID")
    }

    #[tokio::test]
    async fn create_returns_root_with_applied_state() {
        let id = test_id();

        let result = PermissionCommand { repository: InMemoryPermissionRepository::new() }
            .create(id.clone(), "can_invite_users".to_string())
            .await;

        assert!(result.is_ok());
        let root = result.unwrap();
        assert_eq!(root.aggregate_id(), &id);
        assert_eq!(root.name(), "can_invite_users");
        assert_eq!(root.version(), 1);
    }
}
