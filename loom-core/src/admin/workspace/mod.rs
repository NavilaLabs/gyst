pub(crate) mod application;
pub(crate) mod domain;

pub use application::{
    commands::{WorkspaceCommand, WorkspaceCommandTrait},
    views::WorkspaceView,
    WorkspaceRoot,
};
pub use domain::{
    aggregates::{Error, Workspace, WorkspaceId},
    events::WorkspaceEvent,
    interfaces::WorkspaceRepository,
};
