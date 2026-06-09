//! This crate contains all shared UI for the workspace.

use dioxus::prelude::*;

pub mod components;
pub mod form_machine;
pub mod formatting;
pub mod hooks;
pub mod i18n;
pub mod layouts;
pub mod views;

pub const FAVICON: Asset = asset!("/assets/favicon.svg");

// Global base sheets — loaded first so CSS custom properties from @theme are
// available to every component stylesheet that follows.
const CSS_TAILWIND: Asset = asset!("/assets/tailwind.css");
const CSS_THEME: Asset = asset!("/assets/theme.css");

// Preload all component stylesheets as compile-time assets so `GlobalStyles`
// can inject them from the root `App`. This avoids a CSR race condition where
// `document::Link` inside a component inserts a `<link>` tag *after* the
// component has already rendered, causing unstyled flashes during navigation.
const CSS_ACCORDION: Asset = asset!("./components/atoms/accordion/style.css");
const CSS_BUTTON: Asset = asset!("./components/atoms/button/style.css");
const CSS_CARD: Asset = asset!("./components/atoms/card/style.css");
const CSS_DROPDOWN_MENU: Asset = asset!("./components/atoms/dropdown_menu/style.css");
const CSS_INPUT: Asset = asset!("./components/atoms/input/style.css");
const CSS_NAVBAR: Asset = asset!("./components/atoms/navbar/style.css");
const CSS_SEARCHABLE_SELECT: Asset = asset!("./components/atoms/searchable_select/style.css");
const CSS_SELECT: Asset = asset!("./components/atoms/select/style.css");
const CSS_SKELETON: Asset = asset!("./components/atoms/skeleton/style.css");
const CSS_TABLE: Asset = asset!("./components/atoms/table/style.css");
const CSS_TABS: Asset = asset!("./components/atoms/tabs/style.css");
const CSS_TOAST: Asset = asset!("./components/atoms/toast/style.css");
const CSS_TOOLTIP: Asset = asset!("./components/atoms/tooltip/style.css");
const CSS_HEADER: Asset = asset!("./components/organisms/header/style.css");
const CSS_SIDEBAR: Asset = asset!("./components/organisms/sidebar/style.css");
const CSS_SETTINGS_MENU: Asset = asset!("./components/molecules/settings_menu/style.css");
const CSS_THEME_SWITCHER: Asset = asset!("./components/molecules/theme_switcher/style.css");
const CSS_DEFAULT_LAYOUT: Asset = asset!("./layouts/default/style.css");
const CSS_DASHBOARD: Asset = asset!("./views/dashboard/style.css");
const CSS_SETTINGS: Asset = asset!("./views/settings/style.css");
const CSS_SELECT_WORKSPACE: Asset = asset!("./views/select_workspace/style.css");
const CSS_LOGIN: Asset = asset!("./views/login/style.css");
const CSS_LANDING: Asset = asset!("./views/landing/style.css");
const CSS_TIMELINE: Asset = asset!("./views/timeline/style.css");

/// Inject all stylesheets into the document head in dependency order.
///
/// Render this once at the top of `App` so every stylesheet is present before
/// any route renders. This prevents the CSR flash-of-unstyled-content that
/// occurs when `document::Link` inside a component inserts a `<link>` tag
/// after the component has already painted.
///
/// Load order:
/// 1. `tailwind.css` — Tailwind base + `@theme` colour/radius/font tokens as `:root` CSS variables
/// 2. `theme.css` — spacing, shadows, transitions, and shared utility classes
/// 3. Component stylesheets — may use any `var(--color-*)` token from steps 1–2
#[component]
pub fn GlobalStyles() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: CSS_TAILWIND }
        document::Link { rel: "stylesheet", href: CSS_THEME }
        document::Link { rel: "stylesheet", href: CSS_ACCORDION }
        document::Link { rel: "stylesheet", href: CSS_BUTTON }
        document::Link { rel: "stylesheet", href: CSS_CARD }
        document::Link { rel: "stylesheet", href: CSS_DROPDOWN_MENU }
        document::Link { rel: "stylesheet", href: CSS_INPUT }
        document::Link { rel: "stylesheet", href: CSS_NAVBAR }
        document::Link { rel: "stylesheet", href: CSS_SEARCHABLE_SELECT }
        document::Link { rel: "stylesheet", href: CSS_SELECT }
        document::Link { rel: "stylesheet", href: CSS_SKELETON }
        document::Link { rel: "stylesheet", href: CSS_TABLE }
        document::Link { rel: "stylesheet", href: CSS_TABS }
        document::Link { rel: "stylesheet", href: CSS_TOAST }
        document::Link { rel: "stylesheet", href: CSS_TOOLTIP }
        document::Link { rel: "stylesheet", href: CSS_HEADER }
        document::Link { rel: "stylesheet", href: CSS_SIDEBAR }
        document::Link { rel: "stylesheet", href: CSS_SETTINGS_MENU }
        document::Link { rel: "stylesheet", href: CSS_THEME_SWITCHER }
        document::Link { rel: "stylesheet", href: CSS_DEFAULT_LAYOUT }
        document::Link { rel: "stylesheet", href: CSS_DASHBOARD }
        document::Link { rel: "stylesheet", href: CSS_SETTINGS }
        document::Link { rel: "stylesheet", href: CSS_SELECT_WORKSPACE }
        document::Link { rel: "stylesheet", href: CSS_LOGIN }
        document::Link { rel: "stylesheet", href: CSS_LANDING }
        document::Link { rel: "stylesheet", href: CSS_TIMELINE }
    }
}

/// Global shared state for the currently running timesheet.
/// Provided by the top-level `Layout` and consumed by Sidebar, Dashboard, and Timesheets.
pub type RunningTimer = Signal<Option<api::timesheet::TimesheetDto>>;

/// Global shared elapsed-seconds counter for the running timer.
/// Updated by a single coroutine in `Layout`; all components read from this.
pub type RunningElapsed = Signal<u64>;

/// User-level display settings (timezone, date format, language).
/// Loaded once in `Layout` and available to every component via context.
pub type UserSettings = Signal<api::settings::UserSettingsDto>;

/// Workspace-level settings (name, timezone, date format, currency, week start).
/// Loaded once in `Layout` and available to every component via context.
pub type WorkspaceSettings = Signal<api::settings::WorkspaceSettingsDto>;

/// Global cache of all activities. Pre-populated in `Layout` so views start with data.
pub type ActivitiesCache = Signal<Vec<api::activity::ActivityDto>>;

/// Global cache of all tags. Pre-populated in `Layout` so views start with data.
pub type TagsCache = Signal<Vec<api::timesheet_tag::TimesheetsTagDto>>;

/// Whether the sidebar is open/expanded (`true`) or collapsed/hidden (`false`).
/// On desktop: `true` = full sidebar, `false` = icon-only collapsed.
/// On mobile: `true` = drawer visible, `false` = drawer hidden.
pub type SidebarOpen = Signal<bool>;

/// Navigation direction used to select the page-enter animation in `DefaultLayout`.
///
/// - `1`  → forward  (entering from the right, e.g. Dashboard → Timesheets)
/// - `-1` → backward (entering from the left,  e.g. Timesheets → Dashboard)
/// - `0`  → initial load / no direction preference
///
/// Set by the top-level `Layout` component during every render that detects a route change.
pub type NavDirection = Signal<i8>;

/// Frontend plugin host context (§12.1).
///
/// Provided via `use_context_provider` in the root `Layout` so that
/// `PluginSlot<PluginHostCtx>`, `OverridableComponent<PluginHostCtx>`, and
/// `PluginAwareRouter<_, PluginHostCtx>` can access user identity for capability gating.
pub type PluginHostCtx = std::sync::Arc<api::plugin_ctx::ZeitrakPluginCtx>;
