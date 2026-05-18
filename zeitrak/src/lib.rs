#![allow(clippy::missing_errors_doc)]

pub mod authentication;
pub mod authorization;
pub mod auth {
    pub use super::authentication::{CurrentUser, validate_token};
}
pub mod email;
pub mod error;
pub mod smtp_config;
pub mod smtp_oauth2;
pub mod invitation;
pub mod registration;
pub mod setup;
pub mod tenant;
pub mod user_settings;
#[cfg(feature = "landing")]
pub mod waitlist;
pub mod workspace;

pub use tenant::user;
pub use zeitrak_core as core;
pub use zeitrak_infrastructure::database::Migrate;
pub use zeitrak_infrastructure_impl as infrastructure;
