use crate::admin::user::User;

pub mod commands;
pub mod inputs;
pub mod queries;
pub mod rows;

#[eventually_macros::aggregate_root(User)]
#[derive(Debug, Clone, PartialEq)]
pub struct UserRoot;
