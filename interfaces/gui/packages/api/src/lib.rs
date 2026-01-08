//! This crate contains all shared fullstack server functions.
use dioxus::prelude::*;

/// Echo the user input on the server.
#[post("/api/echo")]
pub async fn echo(input: String) -> Result<String, ServerFnError> {
    Ok(input)
}

#[post("/api/database/migrate")]
pub async fn migrate_database() -> Result<(), ServerFnError> {
    Ok(())
}
