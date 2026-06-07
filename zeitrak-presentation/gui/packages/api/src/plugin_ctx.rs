/// Frontend plugin context — the client-visible portion of the current user identity.
///
/// Provided in the Dioxus context tree by the root `Layout` component so that
/// [`dioxus_extism_frontend::PluginSlot`] and [`PluginAwareRouter`] can access
/// the current user's identity for future capability-gating (§12.1).
///
/// This is the frontend counterpart to `zeitrak-plugin-host::ZeitrakHostCtx`,
/// which lives server-side and carries the full authorization repository.
/// The frontend type contains only the fields that survive serialization and
/// make sense in a WASM context.
use dioxus_extism_protocol::SessionContextProvider;
use serde::{Deserialize, Serialize};

/// Frontend-safe plugin host context.
///
/// Placed in the Dioxus context tree so that `PluginSlot<ZeitrakPluginCtx>`,
/// `OverridableComponent<ZeitrakPluginCtx>`, and `PluginAwareRouter<_, ZeitrakPluginCtx>`
/// can access the calling user's identity for capability gating.
///
/// The value is `Default` (all `None`/`false`) until auth resolves.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ZeitrakPluginCtx {
    /// Authenticated user ID, or `None` while loading / unauthenticated.
    pub user_id: Option<String>,
    /// Authenticated user email, or `None` while loading / unauthenticated.
    pub email: Option<String>,
    /// Active workspace ID, or `None` while no workspace is selected.
    pub workspace_id: Option<String>,
    /// Whether the current user has the workspace-admin role.
    pub is_admin: bool,
}

impl SessionContextProvider for ZeitrakPluginCtx {
    fn session_user_id(&self) -> Option<&str> {
        self.user_id.as_deref()
    }

    fn session_email(&self) -> Option<&str> {
        self.email.as_deref()
    }
}
