use crate::admin::workspace_role::WorkspaceRole;

pub mod commands;
pub mod queries;
pub mod views;

#[eventually_macros::aggregate_root(WorkspaceRole)]
#[derive(Debug, Clone, PartialEq)]
pub struct WorkspaceRoleRoot;
