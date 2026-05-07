pub(crate) mod application;
pub(crate) mod domain;

pub use application::{
    WorkspaceRoleRoot,
    commands::{WorkspaceRoleCommand, WorkspaceRoleCommandTrait},
    views::WorkspaceRoleView,
};
pub use domain::{
    aggregates::{Error, WorkspaceRole, WorkspaceRoleId},
    events::WorkspaceRoleEvent,
    interfaces::WorkspaceRoleRepository,
};
