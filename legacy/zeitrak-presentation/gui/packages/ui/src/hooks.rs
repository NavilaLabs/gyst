//! Custom Dioxus hooks for common reactive patterns.

use dioxus::prelude::*;

/// A reactive form field that defaults to a workspace setting but can be
/// overridden by the user.
///
/// ## The problem it solves
///
/// `use_signal(|| workspace_settings.peek().field)` only runs once at mount.
/// When the page loads cold, the `Layout` context starts at hard-coded
/// defaults (`Europe/Berlin`, `EUR`, …) and the real values arrive asynchronously.
/// Any signal seeded with `peek()` at mount will stay on the wrong default.
///
/// ## How it works
///
/// Internally this stores an `Option<T>` override signal:
/// - `None`   → the memo reads **reactively** from the workspace context,
///   so it updates automatically when the context loads.
/// - `Some(v)` → the user has explicitly chosen a value; the context is
///   ignored so subsequent context changes don't clobber the selection.
///
/// ## Usage
///
/// ```rust
/// let (new_timezone, mut new_timezone_set) =
///     use_workspace_field(|ws| ws.timezone.clone());
///
/// SearchableSelect::<String> {
///     value: Some(new_timezone()),          // reactive read
///     on_change: move |v| new_timezone_set.set(Some(v)),
/// }
///
/// // After successful form submission, reset to workspace default:
/// new_timezone_set.set(None);
/// ```
pub fn use_workspace_field<T: Clone + PartialEq + 'static>(
    selector: impl Fn(&api::settings::WorkspaceSettingsDto) -> T + 'static,
) -> (Memo<T>, Signal<Option<T>>) {
    let ws: crate::WorkspaceSettings = use_context();
    let override_val: Signal<Option<T>> = use_signal(|| None);
    let memo = use_memo(move || {
        // When override is None we read `ws` reactively — memo re-runs
        // whenever the context updates (e.g. after Layout's use_resource
        // completes).  When override is Some the context is never read, so
        // the user's explicit selection is never clobbered.
        override_val
            .read()
            .clone()
            .unwrap_or_else(|| selector(&ws.read()))
    });
    (memo, override_val)
}

/// Identical to [`use_workspace_field`] but reads from the **user** settings
/// context instead of the workspace context.
pub fn use_user_field<T: Clone + PartialEq + 'static>(
    selector: impl Fn(&api::settings::UserSettingsDto) -> T + 'static,
) -> (Memo<T>, Signal<Option<T>>) {
    let us: crate::UserSettings = use_context();
    let override_val: Signal<Option<T>> = use_signal(|| None);
    let memo = use_memo(move || {
        override_val
            .read()
            .clone()
            .unwrap_or_else(|| selector(&us.read()))
    });
    (memo, override_val)
}
