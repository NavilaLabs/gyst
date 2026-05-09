use crate::admin::workspace_role::WorkspaceRole;

pub mod commands;
pub mod inputs;
pub mod queries;
pub mod rows;

#[eventually_macros::aggregate_root(WorkspaceRole)]
#[derive(Debug, Clone, PartialEq)]
pub struct WorkspaceRoleRoot;
