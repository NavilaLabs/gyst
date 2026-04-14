use eventually::aggregate;

use crate::admin::{
    permission::PermissionId,
    user::UserId,
    workspace::{
        self,
        domain::{
            aggregates::{Workspace, WorkspaceId},
            events::WorkspaceEvent,
        },
    },
    workspace_role::WorkspaceRoleId,
};

#[eventually_macros::aggregate_root(Workspace)]
pub struct WorkspaceCommand;

impl WorkspaceCommand {
    /// # Errors
    ///
    /// Returns an error if the domain event cannot be applied to the aggregate.
    pub fn create(id: WorkspaceId, name: Option<String>) -> Result<Self, crate::Error> {
        Ok(
            aggregate::Root::<Workspace>::record_new(WorkspaceEvent::Created { id, name }.into())
                .map_err(workspace::DomainError::from)?
                .into(),
        )
    }

    /// # Errors
    ///
    /// Returns an error if the domain event cannot be applied to the aggregate.
    pub fn assign_user_role(
        &mut self,
        user_id: UserId,
        workspace_role_id: WorkspaceRoleId,
    ) -> Result<(), crate::Error> {
        self.record_that(
            WorkspaceEvent::UserRoleAssigned {
                user_id,
                workspace_role_id,
            }
            .into(),
        )
        .map_err(|e| workspace::DomainError::AggregateError(e).into())
    }

    /// # Errors
    ///
    /// Returns an error if the domain event cannot be applied to the aggregate.
    pub fn revoke_user_role(
        &mut self,
        user_id: UserId,
        workspace_role_id: WorkspaceRoleId,
    ) -> Result<(), crate::Error> {
        self.record_that(
            WorkspaceEvent::UserRoleRevoked {
                user_id,
                workspace_role_id,
            }
            .into(),
        )
        .map_err(|e| workspace::DomainError::AggregateError(e).into())
    }

    /// # Errors
    ///
    /// Returns an error if the domain event cannot be applied to the aggregate.
    pub fn grant_user_permission(
        &mut self,
        user_id: UserId,
        permission_id: PermissionId,
    ) -> Result<(), crate::Error> {
        self.record_that(
            WorkspaceEvent::UserPermissionGranted {
                user_id,
                permission_id,
            }
            .into(),
        )
        .map_err(|e| workspace::DomainError::AggregateError(e).into())
    }

    /// # Errors
    ///
    /// Returns an error if the domain event cannot be applied to the aggregate.
    pub fn revoke_user_permission(
        &mut self,
        user_id: UserId,
        permission_id: PermissionId,
    ) -> Result<(), crate::Error> {
        self.record_that(
            WorkspaceEvent::UserPermissionRevoked {
                user_id,
                permission_id,
            }
            .into(),
        )
        .map_err(|e| workspace::DomainError::AggregateError(e).into())
    }

    /// # Errors
    ///
    /// Returns an error if the domain event cannot be applied to the aggregate.
    #[allow(clippy::too_many_arguments)]
    pub fn update_settings(
        &mut self,
        name: Option<String>,
        timezone: String,
        date_format: String,
        currency: String,
        week_start: String,
    ) -> Result<(), crate::Error> {
        self.record_that(
            WorkspaceEvent::SettingsUpdated {
                name,
                timezone,
                date_format,
                currency,
                week_start,
            }
            .into(),
        )
        .map_err(|e| workspace::DomainError::AggregateError(e).into())
    }
}

#[cfg(test)]
mod tests {
    use eventually::aggregate::{Aggregate, Root};

    use super::*;

    fn make_command_shell(id: WorkspaceId) -> WorkspaceCommand {
        let workspace = Workspace::apply(
            None,
            WorkspaceEvent::Created {
                id,
                name: Some("seed".to_string()),
            },
        )
        .expect("seed workspace");
        Root::<Workspace>::rehydrate_from_state(1, workspace).into()
    }

    fn test_id() -> WorkspaceId {
        "019d0ce8-facb-7c90-b9d7-287ae4f17c91"
            .parse()
            .expect("valid UUID")
    }

    #[test]
    fn create_returns_root_with_applied_state() {
        let id: WorkspaceId = "019d0ce8-facb-7c90-b9d7-287ae4f17c92"
            .parse()
            .expect("valid UUID");

        let result = WorkspaceCommand::create(id.clone(), Some("Acme".to_string()));

        assert!(result.is_ok());
        let cmd = result.unwrap();
        assert_eq!(cmd.aggregate_id(), &id);
        assert_eq!(cmd.name(), Some("Acme"));
        assert_eq!(cmd.version(), 1);
    }

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
