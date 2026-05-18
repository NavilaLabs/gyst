//! This crate contains all shared fullstack server functions.

pub mod activity;
pub mod auth;
pub mod developer;
pub mod invitation;
pub mod login;
pub mod member;
pub mod permissions;
pub mod registration;
pub mod session;
pub mod settings;
pub mod setup;
pub mod smtp;
pub mod timesheet;
pub mod timesheet_tag;
pub mod workspace;
#[cfg(feature = "landing")]
pub mod waitlist;
pub mod workspace_role;
