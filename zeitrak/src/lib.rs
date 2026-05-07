#![allow(clippy::missing_errors_doc)]

pub mod authentication;
pub mod authorization;
pub mod error;
pub mod setup;
pub mod tenant;
pub mod user_settings;
pub mod workspace;

pub use zeitrak_core as core;
pub use zeitrak_infrastructure::database::Migrate;
pub use zeitrak_infrastructure_impl as infrastructure;
pub use tenant::user;
