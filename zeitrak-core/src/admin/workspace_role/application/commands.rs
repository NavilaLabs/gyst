use std::fmt::Debug;

use async_trait::async_trait;
use eventually::aggregate;

use crate::admin::{
    permission::PermissionId,
    workspace::WorkspaceId,
    workspace_role::{
        WorkspaceRoleRepository, domain::{
            aggregates::{WorkspaceRole, WorkspaceRoleId},
            events::WorkspaceRoleEvent,
        }
    },
};

#[async_trait]
pub trait WorkspaceRoleCommandTrait<T> {
    type Error: Debug + Sync + Send;

    async fn create(
        &self,
        id: WorkspaceRoleId,
        workspace_id: WorkspaceId,
        name: Option<String>,
    ) -> Result<T, Self::Error>;

    async fn grant_permission(
        &self,
        role_id: WorkspaceRoleId,
        permission_id: PermissionId,
    ) -> Result<(), Self::Error>;

    async fn revoke_permission(
        &self,
        role_id: WorkspaceRoleId,
        permission_id: PermissionId,
    ) -> Result<(), Self::Error>;
}

#[derive(Debug)]
pub struct WorkspaceRoleCommand<R> {
    repository: R,
}

impl<R> WorkspaceRoleCommand<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R> WorkspaceRoleCommandTrait<WorkspaceRole> for WorkspaceRoleCommand<R>
where
    R: Debug + WorkspaceRoleRepository,
{
    type Error = crate::Error<R, WorkspaceRole>;

    async fn create(
        &self,
        id: WorkspaceRoleId,
        workspace_id: WorkspaceId,
        name: Option<String>,
    ) -> Result<WorkspaceRole, Self::Error> {
        Ok(
            aggregate::Root::<WorkspaceRole>::record_new(WorkspaceRoleEvent::Created { id, workspace_id, name }.into())?
                .to_aggregate_type(),
        )
    }

    async fn grant_permission(
           &self,
           id: WorkspaceRoleId,
           permission_id: PermissionId,
       ) -> Result<(), Self::Error> {
       let mut root = self.repository.get(&id).await.map_err(|e| crate::Error::ReadRepositoryError(e.into()))?;
       Ok(root.record_that(WorkspaceRoleEvent::PermissionGranted { permission_id }.into())?)
   }

   async fn revoke_permission(
           &self,
           id: WorkspaceRoleId,
           permission_id: PermissionId,
       ) -> Result<(), Self::Error> {
       let mut root = self.repository.get(&id).await.map_err(|e| crate::Error::ReadRepositoryError(e.into()))?;
       Ok(root.record_that(WorkspaceRoleEvent::PermissionRevoked { permission_id }.into())?)
   }
}

#[cfg(test)]
mod tests {
    use eventually::aggregate::{Aggregate, Root};

    use super::*;

    fn make_command_shell(id: WorkspaceRoleId, workspace_id: WorkspaceId) -> Root<WorkspaceRole> {
        let role = WorkspaceRole::apply(
            None,
            WorkspaceRoleEvent::Created {
                id,
                workspace_id,
                name: Some("seed".to_string()),
            },
        )
        .expect("seed workspace role");
        Root::<WorkspaceRole>::rehydrate_from_state(1, role)
    }

    fn test_ids() -> (WorkspaceRoleId, WorkspaceId) {
        (
            "019d0ce8-facb-7c90-b9d7-287ae4f17c91"
                .parse()
                .expect("valid UUID"),
            "019d0ce8-facb-7c90-b9d7-287ae4f17c92"
                .parse()
                .expect("valid UUID"),
        )
    }

    #[test]
    fn grant_permission_records_event() {
        let (role_id, workspace_id) = test_ids();
        let mut cmd = make_command_shell(role_id, workspace_id);
        let permission_id: PermissionId = "019d0ce8-facb-7c90-b9d7-287ae4f17c94"
            .parse()
            .expect("valid UUID");

        let result = cmd.record_that(WorkspaceRoleEvent::PermissionGranted { permission_id }.into());
        assert!(result.is_ok());
        assert_eq!(cmd.version(), 2);
    }

    #[test]
    fn revoke_permission_records_event() {
        let (role_id, workspace_id) = test_ids();
        let mut cmd = make_command_shell(role_id, workspace_id);
        let permission_id: PermissionId = "019d0ce8-facb-7c90-b9d7-287ae4f17c94".parse().unwrap();

        cmd.record_that(WorkspaceRoleEvent::PermissionGranted { permission_id: permission_id.clone() }.into()).unwrap();
        let result = cmd.record_that(WorkspaceRoleEvent::PermissionRevoked { permission_id }.into());

        assert!(result.is_ok());
        assert_eq!(cmd.version(), 3);
    }
}
