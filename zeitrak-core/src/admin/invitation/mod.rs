pub(crate) mod application;
pub(crate) mod domain;

pub use application::{
    InvitationRoot,
    commands::{InvitationCommand, InvitationCommandTrait},
    inputs::CreateInvitationInput,
    queries::{InvitationQuery, InvitationQueryTrait},
    rows::InvitationRow,
};
pub use domain::{
    aggregates::{Invitation, InvitationId, InvitationStatus},
    events::InvitationEvent,
    interfaces::InvitationRepository,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invitation already exists")]
    AlreadyExists,
    #[error("invitation not found")]
    NotFound,
    #[error("invitation is not in pending status")]
    NotPending,
    #[error("invitation has expired")]
    Expired,
}
