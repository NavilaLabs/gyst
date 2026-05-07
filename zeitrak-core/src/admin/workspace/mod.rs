pub(crate) mod application;
pub(crate) mod domain;

pub use application::{
    WorkspaceRoot,
    commands::{WorkspaceCommand, WorkspaceCommandTrait},
    views::WorkspaceView,
};
pub use domain::{
    aggregates::{Error, Workspace, WorkspaceId},
    events::WorkspaceEvent,
    interfaces::WorkspaceRepository,
};
