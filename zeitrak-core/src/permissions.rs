//! Permission name constants following the `<aggregate>.<action>` convention.
//!
//! These are the canonical string keys stored in the `permissions` table.
//! A user holding the [`ADMIN_BYPASS`] permission bypasses all other permission
//! checks — only regular users with an explicit role or direct grant need the
//! remaining constants.
//!
//! Naming rule: `<aggregate>.<action>`, e.g. `customer.create`.
//! Event names are internal implementation details and are intentionally
//! kept separate from permission names.

// Admin / cross-cutting
/// Grants unconditional access to every operation across all workspaces.
///
/// Assign this permission to an admin workspace role instead of hardcoding
/// role names in the authorization service.
pub const ADMIN_BYPASS: &str = "admin.bypass";

// Activity domain
pub const ACTIVITY_CREATE: &str = "activity.create";
pub const ACTIVITY_UPDATE: &str = "activity.update";
pub const ACTIVITY_DELETE: &str = "activity.delete";

// Timesheet domain
pub const TIMESHEET_CREATE: &str = "timesheet.create";
pub const TIMESHEET_UPDATE: &str = "timesheet.update";
pub const TIMESHEET_EXPORT: &str = "timesheet.export";
pub const TIMESHEET_CANCEL: &str = "timesheet.cancel";

// Cross-cutting
pub const TAG_MANAGE: &str = "tag.manage";

// Member management
pub const MEMBER_INVITE: &str = "member.invite";
pub const MEMBER_MANAGE: &str = "member.manage";

// Role management
pub const ROLE_MANAGE: &str = "role.manage";

/// Every built-in permission that must be seeded in the database.
/// Used by migrations and initial setup logic.
pub const ALL: &[&str] = &[
    ADMIN_BYPASS,
    ACTIVITY_CREATE,
    ACTIVITY_UPDATE,
    ACTIVITY_DELETE,
    TIMESHEET_CREATE,
    TIMESHEET_UPDATE,
    TIMESHEET_EXPORT,
    TIMESHEET_CANCEL,
    TAG_MANAGE,
    MEMBER_INVITE,
    MEMBER_MANAGE,
    ROLE_MANAGE,
];

// ── Plugin-extensible registry ────────────────────────────────────────────────

/// A source of additional permission names contributed by a plugin.
///
/// The core permissions are always present (see [`ALL`]); implement this trait
/// to register extra permissions that your plugin introduces.
pub trait PermissionsProvider: Send + Sync {
    fn permissions(&self) -> &[&'static str];
}

struct CorePermissionsProvider;

impl PermissionsProvider for CorePermissionsProvider {
    fn permissions(&self) -> &[&'static str] {
        ALL
    }
}

/// Aggregates permissions from core and all registered plugins.
///
/// Build one instance at startup by calling [`PermissionsRegistry::register`]
/// for each plugin, then use [`PermissionsRegistry::all`] to iterate the
/// complete set for database seeding.
pub struct PermissionsRegistry {
    providers: Vec<Box<dyn PermissionsProvider>>,
}

impl Default for PermissionsRegistry {
    fn default() -> Self {
        Self {
            providers: vec![Box::new(CorePermissionsProvider)],
        }
    }
}

impl PermissionsRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, provider: impl PermissionsProvider + 'static) {
        self.providers.push(Box::new(provider));
    }

    /// Returns an iterator over every permission name from all providers.
    pub fn all(&self) -> impl Iterator<Item = &str> {
        self.providers
            .iter()
            .flat_map(|p| p.permissions().iter().copied())
    }
}
