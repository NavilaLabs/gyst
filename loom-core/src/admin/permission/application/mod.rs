use crate::admin::permission::Permission;

pub mod commands;
pub mod queries;
pub mod rows;

#[eventually_macros::aggregate_root(Permission)]
#[derive(Debug, Clone, PartialEq)]
pub struct PermissionRoot;
