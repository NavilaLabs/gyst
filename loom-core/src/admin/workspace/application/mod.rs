use crate::admin::workspace::Workspace;

pub mod commands;
pub mod queries;
pub mod views;

#[eventually_macros::aggregate_root(Workspace)]
#[derive(Debug, Clone, PartialEq)]
pub struct WorkspaceRoot;
