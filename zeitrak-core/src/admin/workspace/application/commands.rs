use std::fmt::Debug;

use async_trait::async_trait;
use eventually::aggregate::Root;

use crate::admin::{
    permission::PermissionId,
    user::UserId,
    workspace::{
        application::WorkspaceRoot,
        domain::{
            aggregates::{Workspace, WorkspaceId},
            events::WorkspaceEvent,
            interfaces::WorkspaceRepository,
        },
    },
    workspace_role::WorkspaceRoleId,
};

#[async_trait]
pub trait WorkspaceCommandTrait<R> {
    type Error: Debug + Sync + Send;

    async fn create(
        &self,
        id: WorkspaceId,
        name: Option<String>,
    ) -> Result<Root<Workspace>, Self::Error>;

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
        &self,
        id: WorkspaceId,
        name: Option<String>,
        timezone: String,
        date_format: String,
        currency: String,
        week_start: String,
    ) -> Result<(), Self::Error>;
}

#[derive(Debug)]
pub struct WorkspaceCommand<Repo> {
    repository: Repo,
}

impl<Repo> WorkspaceCommand<Repo> {
    pub const fn new(repository: Repo) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<Repo, R> WorkspaceCommandTrait<R> for WorkspaceCommand<Repo>
where
    Repo: Debug + WorkspaceRepository<R>,
{
    type Error = crate::Error<Repo, Workspace, R>;

    /// # Errors
    ///
    /// Returns an error if the domain event cannot be applied or the root cannot be saved.
    async fn create(
        &self,
        id: WorkspaceId,
        name: Option<String>,
    ) -> Result<Root<Workspace>, <Self as WorkspaceCommandTrait<R>>::Error> {
        let mut root = Root::<Workspace>::record_new(WorkspaceEvent::Created { id, name }.into())?;
        self.repository
            .save(&mut root)
            .await
            .map_err(|e| crate::Error::WriteRepositoryError(e.into()))?;
        Ok(root)
    }

    async fn assign_user_role(
        &self,
        id: WorkspaceId,
        user_id: UserId,
        workspace_role_id: WorkspaceRoleId,
    ) -> Result<(), <Self as WorkspaceCommandTrait<R>>::Error> {
        let mut root: WorkspaceRoot = self
            .repository
            .get(&id)
            .await
            .map_err(|e| crate::Error::ReadRepositoryError(e.into()))?
            .into();
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
            .map_err(|e| crate::Error::WriteRepositoryError(e.into()))
    }

    async fn revoke_user_role(
        &self,
        id: WorkspaceId,
        user_id: UserId,
        workspace_role_id: WorkspaceRoleId,
    ) -> Result<(), <Self as WorkspaceCommandTrait<R>>::Error> {
        let mut root: WorkspaceRoot = self
            .repository
            .get(&id)
            .await
            .map_err(|e| crate::Error::ReadRepositoryError(e.into()))?
            .into();
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
            .map_err(|e| crate::Error::WriteRepositoryError(e.into()))
    }

    async fn grant_user_permission(
        &self,
        id: WorkspaceId,
        user_id: UserId,
        permission_id: PermissionId,
    ) -> Result<(), <Self as WorkspaceCommandTrait<R>>::Error> {
        let mut root: WorkspaceRoot = self
            .repository
            .get(&id)
            .await
            .map_err(|e| crate::Error::ReadRepositoryError(e.into()))?
            .into();
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
            .map_err(|e| crate::Error::WriteRepositoryError(e.into()))
    }

    async fn revoke_user_permission(
        &self,
        id: WorkspaceId,
        user_id: UserId,
        permission_id: PermissionId,
    ) -> Result<(), <Self as WorkspaceCommandTrait<R>>::Error> {
        let mut root: WorkspaceRoot = self
            .repository
            .get(&id)
            .await
            .map_err(|e| crate::Error::ReadRepositoryError(e.into()))?
            .into();
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
            .map_err(|e| crate::Error::WriteRepositoryError(e.into()))
    }

    async fn update_settings(
        &self,
        id: WorkspaceId,
        name: Option<String>,
        timezone: String,
        date_format: String,
        currency: String,
        week_start: String,
    ) -> Result<(), <Self as WorkspaceCommandTrait<R>>::Error> {
        let mut root: WorkspaceRoot = self
            .repository
            .get(&id)
            .await
            .map_err(|e| crate::Error::ReadRepositoryError(e.into()))?
            .into();
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
            .map_err(|e| crate::Error::WriteRepositoryError(e.into()))
    }
}

#[cfg(test)]
mod tests {
    use eventually::aggregate::{Aggregate, Root};

    use super::*;

    fn make_command_shell(id: WorkspaceId) -> Root<Workspace> {
        let workspace = Workspace::apply(
            None,
            WorkspaceEvent::Created {
                id,
                name: Some("seed".to_string()),
            },
        )
        .expect("seed workspace");
        Root::<Workspace>::rehydrate_from_state(1, workspace)
    }

    fn test_id() -> WorkspaceId {
        "019d0ce8-facb-7c90-b9d7-287ae4f17c91"
            .parse()
            .expect("valid UUID")
    }

    #[test]
    fn assign_user_role_records_event() {
        let id = test_id();
        let mut cmd = make_command_shell(id);
        let user_id: UserId = "019d0ce8-facb-7c90-b9d7-287ae4f17c92"
            .parse()
            .expect("valid UUID");
        let role_id: WorkspaceRoleId = "019d0ce8-facb-7c90-b9d7-287ae4f17c93"
            .parse()
            .expect("valid UUID");

        let result = cmd.record_that(
            WorkspaceEvent::UserRoleAssigned {
                user_id,
                workspace_role_id: role_id,
            }
            .into(),
        );
        assert!(result.is_ok());
        assert_eq!(cmd.version(), 2);
    }

    #[test]
    fn revoke_user_role_records_event() {
        let id = test_id();
        let mut cmd = make_command_shell(id);
        let user_id: UserId = "019d0ce8-facb-7c90-b9d7-287ae4f17c92".parse().unwrap();
        let role_id: WorkspaceRoleId = "019d0ce8-facb-7c90-b9d7-287ae4f17c93".parse().unwrap();

        cmd.record_that(
            WorkspaceEvent::UserRoleAssigned {
                user_id: user_id.clone(),
                workspace_role_id: role_id.clone(),
            }
            .into(),
        )
        .unwrap();
        let result = cmd.record_that(
            WorkspaceEvent::UserRoleRevoked {
                user_id,
                workspace_role_id: role_id,
            }
            .into(),
        );

        assert!(result.is_ok());
        assert_eq!(cmd.version(), 3);
    }

    #[test]
    fn grant_user_permission_records_event() {
        let id = test_id();
        let mut cmd = make_command_shell(id);
        let user_id: UserId = "019d0ce8-facb-7c90-b9d7-287ae4f17c92".parse().unwrap();
        let perm_id: PermissionId = "019d0ce8-facb-7c90-b9d7-287ae4f17c94".parse().unwrap();

        let result = cmd.record_that(
            WorkspaceEvent::UserPermissionGranted {
                user_id,
                permission_id: perm_id,
            }
            .into(),
        );

        assert!(result.is_ok());
        assert_eq!(cmd.version(), 2);
    }

    #[test]
    fn revoke_user_permission_records_event() {
        let id = test_id();
        let mut cmd = make_command_shell(id);
        let user_id: UserId = "019d0ce8-facb-7c90-b9d7-287ae4f17c92".parse().unwrap();
        let perm_id: PermissionId = "019d0ce8-facb-7c90-b9d7-287ae4f17c94".parse().unwrap();

        cmd.record_that(
            WorkspaceEvent::UserPermissionGranted {
                user_id: user_id.clone(),
                permission_id: perm_id.clone(),
            }
            .into(),
        )
        .unwrap();
        let result = cmd.record_that(
            WorkspaceEvent::UserPermissionRevoked {
                user_id,
                permission_id: perm_id,
            }
            .into(),
        );

        assert!(result.is_ok());
        assert_eq!(cmd.version(), 3);
    }
}
