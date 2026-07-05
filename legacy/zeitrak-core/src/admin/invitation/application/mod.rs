use crate::admin::invitation::domain::aggregates::Invitation;

pub mod commands;
pub mod inputs;
pub mod queries;
pub mod rows;

#[eventually_macros::aggregate_root(Invitation)]
#[derive(Debug, Clone, PartialEq)]
pub struct InvitationRoot;
