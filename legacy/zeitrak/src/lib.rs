#![allow(clippy::missing_errors_doc)]

#[cfg(all(feature = "sqlite", feature = "postgres"))]
compile_error!("features `sqlite` and `postgres` are mutually exclusive — enable exactly one");

pub mod authentication;
pub mod authorization;
// pub mod plugin_aggregate;
// pub mod plugin_hooks;
pub mod auth {
    pub use super::authentication::{CurrentUser, validate_token};
}
pub mod email;
pub mod error;
pub mod invitation;
pub mod registration;
pub mod setup;
pub mod smtp_config;
pub mod smtp_oauth2;
pub mod tenant;
pub mod user_settings;
#[cfg(feature = "landing")]
pub mod waitlist;
pub mod workspace;

pub use tenant::user;
pub use zeitrak_core as core;
pub use zeitrak_infrastructure::database::Migrate;
pub use zeitrak_infrastructure_impl as infrastructure;
