use eventually::aggregate;

use crate::admin::permission::{
    self,
    domain::{
        aggregates::{Permission, PermissionId},
        events::PermissionEvent,
    },
};

#[eventually_macros::aggregate_root(Permission)]
pub struct PermissionCommand;

impl PermissionCommand {
    /// # Errors
    ///
    /// Returns an error if the domain event cannot be applied to the aggregate.
    pub fn create(id: PermissionId, name: String) -> Result<Self, permission::DomainError> {
        Ok(
            aggregate::Root::<Permission>::record_new(PermissionEvent::Created { id, name }.into())
                .map_err(permission::DomainError::AggregateError)?
                .into(),
        )
    }
}

#[cfg(test)]
mod tests {
    use eventually::aggregate::{Aggregate, Root};

    use super::*;

    fn make_command_shell(id: PermissionId) -> PermissionCommand {
        let permission = Permission::apply(
            None,
            PermissionEvent::Created {
                id,
                name: "seed".to_string(),
            },
        )
        .expect("seed permission");
        Root::<Permission>::rehydrate_from_state(1, permission).into()
    }

    fn test_id() -> PermissionId {
        "019d0ce8-facb-7c90-b9d7-287ae4f17c91"
            .parse()
            .expect("valid UUID")
    }

    #[test]
    fn create_returns_root_with_applied_state() {
        let _shell = make_command_shell(test_id());
        let id: PermissionId = "019d0ce8-facb-7c90-b9d7-287ae4f17c92"
            .parse()
            .expect("valid UUID");

        let result = PermissionCommand::create(id.clone(), "can_invite_users".to_string());

        assert!(result.is_ok());
        let cmd = result.unwrap();
        assert_eq!(cmd.aggregate_id(), &id);
        assert_eq!(cmd.name(), "can_invite_users");
        assert_eq!(cmd.version(), 1);
    }
}
