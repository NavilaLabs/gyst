use crate::admin::workspace::Workspace;

pub mod commands;
pub mod inputs;
pub mod queries;
pub mod rows;

#[eventually_macros::aggregate_root(Workspace)]
#[derive(Debug, Clone, PartialEq)]
pub struct WorkspaceRoot;
