use std::fmt::Debug;

use eventually::aggregate;

use crate::admin::{
    permission::PermissionId,
    workspace::WorkspaceId,
    workspace_role::{
        self,
        domain::{
            aggregates::{WorkspaceRole, WorkspaceRoleId},
            events::WorkspaceRoleEvent,
        },
    },
};

pub trait WorkspaceRoleCommandTrait<T> {
    type Error: Debug + Sync + Send;

    fn create(
        &self,
        id: WorkspaceRoleId,
        workspace_id: WorkspaceId,
        name: Option<String>,
    ) -> Result<T, Self::Error>;
}

#[eventually_macros::aggregate_root(WorkspaceRole)]
pub struct WorkspaceRoleCommand;

impl WorkspaceRoleCommandTrait<WorkspaceRoleCommand> for WorkspaceRoleCommand {
    type Error = workspace_role::Error;

    fn create(
        &self,
        id: WorkspaceRoleId,
        workspace_id: WorkspaceId,
        name: Option<String>,
    ) -> Result<WorkspaceRoleCommand, Self::Error> {
        Ok(aggregate::Root::<WorkspaceRole>::record_new(
            WorkspaceRoleEvent::Created {
                id,
                workspace_id,
                name,
            }
            .into(),
        )?
        .into())
    }
}

impl WorkspaceRoleCommand {
    /// # Errors
    ///
    /// Returns an error if the domain event cannot be applied to the aggregate.
    pub fn create(
        id: WorkspaceRoleId,
        workspace_id: WorkspaceId,
        name: Option<String>,
    ) -> Result<Self, workspace_role::Error> {
        Ok(aggregate::Root::<WorkspaceRole>::record_new(
            WorkspaceRoleEvent::Created {
                id,
                workspace_id,
                name,
            }
            .into(),
        )?
        .into())
    }

    /// # Errors
    ///
    /// Returns an error if the domain event cannot be applied to the aggregate.
    pub fn grant_permission(&mut self, permission_id: PermissionId) -> Result<(), workspace_role::Error> {
        self.record_that(WorkspaceRoleEvent::PermissionGranted { permission_id }.into())
    }

    /// # Errors
    ///
    /// Returns an error if the domain event cannot be applied to the aggregate.
    pub fn revoke_permission(&mut self, permission_id: PermissionId) -> Result<(), workspace_role::Error> {
        self.record_that(WorkspaceRoleEvent::PermissionRevoked { permission_id }.into())
    }
}

#[cfg(test)]
mod tests {
    use eventually::aggregate::{Aggregate, Root};

    use super::*;

    fn make_command_shell(id: WorkspaceRoleId, workspace_id: WorkspaceId) -> WorkspaceRoleCommand {
        let role = WorkspaceRole::apply(
            None,
            WorkspaceRoleEvent::Created {
                id,
                workspace_id,
                name: Some("seed".to_string()),
            },
        )
        .expect("seed workspace role");
        Root::<WorkspaceRole>::rehydrate_from_state(1, role).into()
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
    fn create_returns_root_with_applied_state() {
        let (_, workspace_id) = test_ids();
        let id: WorkspaceRoleId = "019d0ce8-facb-7c90-b9d7-287ae4f17c93"
            .parse()
            .expect("valid UUID");

        let result = WorkspaceRoleCommand::create(id.clone(), workspace_id, Some("admin".to_string()));

        assert!(result.is_ok());
        let cmd = result.unwrap();
        assert_eq!(cmd.aggregate_id(), &id);
        assert_eq!(cmd.name(), Some("admin"));
        assert_eq!(cmd.version(), 1);
    }

    #[test]
    fn grant_permission_records_event() {
        let (role_id, workspace_id) = test_ids();
        let mut cmd = make_command_shell(role_id, workspace_id);
        let permission_id = "019d0ce8-facb-7c90-b9d7-287ae4f17c94"
            .parse()
            .expect("valid UUID");

        let result = cmd.grant_permission(permission_id);
        assert!(result.is_ok());
        assert_eq!(cmd.version(), 2);
    }

    #[test]
    fn revoke_permission_records_event() {
        let (role_id, workspace_id) = test_ids();
        let mut cmd = make_command_shell(role_id, workspace_id);
        let permission_id: PermissionId = "019d0ce8-facb-7c90-b9d7-287ae4f17c94".parse().unwrap();

        cmd.grant_permission(permission_id.clone()).unwrap();
        let result = cmd.revoke_permission(permission_id);

        assert!(result.is_ok());
        assert_eq!(cmd.version(), 3);
    }
}
