use dioxus::prelude::*;

/// Adds an email address to the early-access waitlist.
///
/// Returns `Ok(())` whether the address is new or already present — callers
/// cannot distinguish the two cases to prevent email enumeration.
#[post("/api/waitlist/join")]
pub async fn join_waitlist(email: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        use crate::session::internal;

        let email_sender = zeitrak::email::email_sender_from_config()
            .await
            .map_err(internal)?;
        let owner_email = zeitrak::email::owner_email();

        zeitrak::waitlist::join_waitlist(email, &*email_sender, owner_email)
            .await
            .map_err(internal)
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = email;
        Ok(())
    }
}
