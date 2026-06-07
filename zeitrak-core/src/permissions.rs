//! Permission name constants following the `<aggregate>.<action>` convention.
//!
//! These are the canonical string keys stored in the `permissions` table.
//! Instance admin status is stored as `is_instance_admin` on the user — not
//! as a permission.  All users receive explicit per-aggregate permissions via
//! their roles.
//!
//! Naming rule: `<aggregate>.<action>`, e.g. `activity.create`.
//! Event names are internal implementation details and are intentionally
//! kept separate from permission names.

// Activity domain
pub const ACTIVITY_CREATE: &str = "activity.create";
pub const ACTIVITY_READ: &str = "activity.read";
pub const ACTIVITY_UPDATE: &str = "activity.update";
pub const ACTIVITY_DELETE: &str = "activity.delete";
pub const ACTIVITY_EXPORT: &str = "activity.export";

// Timesheet domain
pub const TIMESHEET_CREATE: &str = "timesheet.create";
pub const TIMESHEET_READ: &str = "timesheet.read";
pub const TIMESHEET_UPDATE: &str = "timesheet.update";
pub const TIMESHEET_DELETE: &str = "timesheet.delete";
pub const TIMESHEET_EXPORT: &str = "timesheet.export";
/// Grants access to all workspace members' timesheets, not just the user's own.
pub const TIMESHEET_READ_ALL: &str = "timesheet.read_all";

// Tag domain
pub const TAG_CREATE: &str = "tag.create";
pub const TAG_READ: &str = "tag.read";
pub const TAG_UPDATE: &str = "tag.update";
pub const TAG_DELETE: &str = "tag.delete";
pub const TAG_EXPORT: &str = "tag.export";

// Member management
pub const MEMBER_CREATE: &str = "member.create";
pub const MEMBER_READ: &str = "member.read";
pub const MEMBER_UPDATE: &str = "member.update";
pub const MEMBER_DELETE: &str = "member.delete";
pub const MEMBER_EXPORT: &str = "member.export";

// Role management
pub const ROLE_CREATE: &str = "role.create";
pub const ROLE_READ: &str = "role.read";
pub const ROLE_UPDATE: &str = "role.update";
pub const ROLE_DELETE: &str = "role.delete";
pub const ROLE_EXPORT: &str = "role.export";

/// Every built-in permission that must be seeded in the database.
/// Used by migrations and initial setup logic.
pub const ALL: &[&str] = &[
    ACTIVITY_CREATE,
    ACTIVITY_READ,
    ACTIVITY_UPDATE,
    ACTIVITY_DELETE,
    ACTIVITY_EXPORT,
    TIMESHEET_CREATE,
    TIMESHEET_READ,
    TIMESHEET_UPDATE,
    TIMESHEET_DELETE,
    TIMESHEET_EXPORT,
    TIMESHEET_READ_ALL,
    TAG_CREATE,
    TAG_READ,
    TAG_UPDATE,
    TAG_DELETE,
    TAG_EXPORT,
    MEMBER_CREATE,
    MEMBER_READ,
    MEMBER_UPDATE,
    MEMBER_DELETE,
    MEMBER_EXPORT,
    ROLE_CREATE,
    ROLE_READ,
    ROLE_UPDATE,
    ROLE_DELETE,
    ROLE_EXPORT,
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
