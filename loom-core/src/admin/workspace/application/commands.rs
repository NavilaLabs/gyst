use std::fmt::Debug;

use async_trait::async_trait;
use eventually::aggregate;

use crate::admin::{
    permission::PermissionId,
    user::UserId,
    workspace::{
        self, WorkspaceRepository,
        domain::{
            aggregates::{Workspace, WorkspaceId},
            events::WorkspaceEvent,
        },
    },
    workspace_role::WorkspaceRoleId,
};

#[async_trait]
pub trait WorkspaceCommandTrait<T> {
    type Error: Debug + Sync + Send;

    async fn create(&self, id: WorkspaceId, name: Option<String>) -> Result<T, Self::Error>;

    async fn assign_user_role(
        &self,
        id: WorkspaceId,
        user_id: UserId,
        workspace_role_id: WorkspaceRoleId,
    ) -> Result<(), Self::Error>;

    async fn revoke_user_role(
        &self,
        id: WorkspaceId,
        user_id: UserId,
        workspace_role_id: WorkspaceRoleId,
    ) -> Result<(), Self::Error>;

    async fn grant_user_permission(
        &self,
        id: WorkspaceId,
        user_id: UserId,
        permission_id: PermissionId,
    ) -> Result<(), Self::Error>;

    async fn revoke_user_permission(
        &self,
        id: WorkspaceId,
        user_id: UserId,
        permission_id: PermissionId,
    ) -> Result<(), Self::Error>;

    async fn update_settings(
        &mut self,
        id: WorkspaceId,
        name: Option<String>,
        timezone: String,
        date_format: String,
        currency: String,
        week_start: String,
    ) -> Result<(), Self::Error>;
}

#[derive(Debug)]
pub struct WorkspaceCommand<R> {
    repository: R,
}

impl<R> WorkspaceCommand<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R> WorkspaceCommandTrait<Workspace> for WorkspaceCommand<R>
where
    R: Debug + WorkspaceRepository,
{
    type Error = crate::Error<R, Workspace>;

    async fn create(
        &self,
        id: WorkspaceId,
        name: Option<String>,
    ) -> Result<Workspace, Self::Error> {
        Ok(
            aggregate::Root::<Workspace>::record_new(WorkspaceEvent::Created { id, name }.into())?
                .to_aggregate_type(),
        )
    }

    async fn assign_user_role(
        &self,
        id: WorkspaceId,
        user_id: UserId,
        workspace_role_id: WorkspaceRoleId,
    ) -> Result<(), Self::Error> {
        let mut root = self
            .repository
            .get(&id)
            .await
            .map_err(|e| crate::Error::ReadRepositoryError(e.into()))?;
        root.record_that(
            WorkspaceEvent::UserRoleAssigned {
                user_id,
                workspace_role_id,
            }
            .into(),
        )?;

        self.repository
            .save(&mut root)
            .await
            .map_err(|e| crate::Error::WriteRepositoryError(e.into()))?;
        Ok(())
    }

    async fn revoke_user_role(
        &self,
        id: WorkspaceId,
        user_id: UserId,
        workspace_role_id: WorkspaceRoleId,
    ) -> Result<(), Self::Error> {
        let mut root = self
            .repository
            .get(&id)
            .await
            .map_err(|e| crate::Error::ReadRepositoryError(e.into()))?;
        root.record_that(
            WorkspaceEvent::UserRoleRevoked {
                user_id,
                workspace_role_id,
            }
            .into(),
        )?;
        self.repository
            .save(&mut root)
            .await
            .map_err(|e| crate::Error::WriteRepositoryError(e.into()))?;
        Ok(())
    }

    async fn grant_user_permission(
        &self,
        id: WorkspaceId,
        user_id: UserId,
        permission_id: PermissionId,
    ) -> Result<(), Self::Error> {
        let mut root = self
            .repository
            .get(&id)
            .await
            .map_err(|e| crate::Error::ReadRepositoryError(e.into()))?;
        root.record_that(
            WorkspaceEvent::UserPermissionGranted {
                user_id,
                permission_id,
            }
            .into(),
        )?;
        self.repository
            .save(&mut root)
            .await
            .map_err(|e| crate::Error::WriteRepositoryError(e.into()))?;
        Ok(())
    }

    async fn revoke_user_permission(
        &self,
        id: WorkspaceId,
        user_id: UserId,
        permission_id: PermissionId,
    ) -> Result<(), Self::Error> {
        let mut root = self
            .repository
            .get(&id)
            .await
            .map_err(|e| crate::Error::ReadRepositoryError(e.into()))?;
        root.record_that(
            WorkspaceEvent::UserPermissionRevoked {
                user_id,
                permission_id,
            }
            .into(),
        )?;
        self.repository
            .save(&mut root)
            .await
            .map_err(|e| crate::Error::WriteRepositoryError(e.into()))?;
        Ok(())
    }

    async fn update_settings(
        &mut self,
        id: WorkspaceId,
        name: Option<String>,
        timezone: String,
        date_format: String,
        currency: String,
        week_start: String,
    ) -> Result<(), Self::Error> {
        let mut root = self
            .repository
            .get(&id)
            .await
            .map_err(|e| crate::Error::ReadRepositoryError(e.into()))?;
        root.record_that(
            WorkspaceEvent::SettingsUpdated {
                name,
                timezone,
                date_format,
                currency,
                week_start,
            }
            .into(),
        )?;
        self.repository
            .save(&mut root)
            .await
            .map_err(|e| crate::Error::WriteRepositoryError(e.into()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use eventually::aggregate::{Aggregate, Root};

    use super::*;

    // fn make_command_shell(id: WorkspaceId) -> WorkspaceCommand {
    //     let workspace = Workspace::apply(
    //         None,
    //         WorkspaceEvent::Created {
    //             id,
    //             name: Some("seed".to_string()),
    //         },
    //     )
    //     .expect("seed workspace");
    //     Root::<Workspace>::rehydrate_from_state(1, workspace).into()
    // }

    // fn test_id() -> WorkspaceId {
    //     "019d0ce8-facb-7c90-b9d7-287ae4f17c91"
    //         .parse()
    //         .expect("valid UUID")
    // }

    // #[test]
    // fn create_returns_root_with_applied_state() {
    //     let id: WorkspaceId = "019d0ce8-facb-7c90-b9d7-287ae4f17c92"
    //         .parse()
    //         .expect("valid UUID");

    //     let result = WorkspaceCommand::new().create(id.clone(), Some("Acme".to_string()));

    //     assert!(result.is_ok());
    //     let cmd = result.unwrap();
    //     assert_eq!(cmd.aggregate_id(), &id);
    //     assert_eq!(cmd.name(), Some("Acme"));
    //     assert_eq!(cmd.version(), 1);
    // }

    #[test]
    fn assign_user_role_records_event() {
        let id = test_id();
        let mut cmd = make_command_shell(id);
        let user_id = "019d0ce8-facb-7c90-b9d7-287ae4f17c92"
            .parse()
            .expect("valid UUID");
        let role_id = "019d0ce8-facb-7c90-b9d7-287ae4f17c93"
            .parse()
            .expect("valid UUID");

        let result = cmd.assign_user_role(user_id, role_id);
        assert!(result.is_ok());
        assert_eq!(cmd.version(), 2);
    }

    #[test]
    fn revoke_user_role_records_event() {
        let id = test_id();
        let mut cmd = make_command_shell(id);
        let user_id: UserId = "019d0ce8-facb-7c90-b9d7-287ae4f17c92".parse().unwrap();
        let role_id: WorkspaceRoleId = "019d0ce8-facb-7c90-b9d7-287ae4f17c93".parse().unwrap();

        cmd.assign_user_role(user_id.clone(), role_id.clone())
            .unwrap();
        let result = cmd.revoke_user_role(user_id, role_id);

        assert!(result.is_ok());
        assert_eq!(cmd.version(), 3);
    }

    #[test]
    fn grant_user_permission_records_event() {
        let id = test_id();
        let mut cmd = make_command_shell(id);
        let user_id: UserId = "019d0ce8-facb-7c90-b9d7-287ae4f17c92".parse().unwrap();
        let perm_id: PermissionId = "019d0ce8-facb-7c90-b9d7-287ae4f17c94".parse().unwrap();

        let result = cmd.grant_user_permission(user_id, perm_id);

        assert!(result.is_ok());
        assert_eq!(cmd.version(), 2);
    }

    #[test]
    fn revoke_user_permission_records_event() {
        let id = test_id();
        let mut cmd = make_command_shell(id);
        let user_id: UserId = "019d0ce8-facb-7c90-b9d7-287ae4f17c92".parse().unwrap();
        let perm_id: PermissionId = "019d0ce8-facb-7c90-b9d7-287ae4f17c94".parse().unwrap();

        cmd.grant_user_permission(user_id.clone(), perm_id.clone())
            .unwrap();
        let result = cmd.revoke_user_permission(user_id, perm_id);

        assert!(result.is_ok());
        assert_eq!(cmd.version(), 3);
    }
}
