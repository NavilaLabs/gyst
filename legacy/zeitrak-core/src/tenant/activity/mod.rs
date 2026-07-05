pub(crate) mod application;
pub(crate) mod domain;

pub use application::{
    ActivityRoot,
    commands::{ActivityCommand, ActivityCommandTrait, ActivityHandler, ActivityHandlerTrait},
    inputs::{CreateActivityInput, UpdateActivityInput},
    queries::{ActivityQuery, ActivityQueryTrait},
    views::ActivityRow,
};
pub use domain::{
    aggregates::{Activity, ActivityId, Error},
    events::ActivityEvent,
    interfaces::ActivityRepository,
};
