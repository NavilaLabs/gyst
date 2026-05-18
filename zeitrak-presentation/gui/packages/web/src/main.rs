use api::auth::UserInfo;
use api::settings::{UserSettingsDto, WorkspaceSettingsDto};
use dioxus::prelude::*;
use dioxus_i18n::{prelude::*, tid};
use dioxus_motion::prelude::*;
use easer::functions::Easing;
#[cfg(feature = "landing")]
use ui::views::LandingPage;
use ui::{
    components::{
        atoms::{ToastMessage, ToastStack},
        organisms::{Header, Sidebar},
    },
    views::{
        setup::Setup, Activities, Dashboard, Database, InvitationAccept, Login, Register,
        SelectWorkspace, Settings, Tags, Timesheets, VerifyEmailConfirm, VerifyEmailPending,
    },
    ActivitiesCache, GlobalStyles, RunningElapsed, RunningTimer, SidebarOpen, TagsCache,
    TimesheetsCache, UserSettings, WorkspaceSettings, FAVICON,
};
use unic_langid::langid;

/// Three-state auth signal shared across the whole app.
///
/// - `None`           → still checking (initial page load)
/// - `Some(None)`     → confirmed not authenticated
/// - `Some(Some(u))`  → confirmed authenticated as `u`
pub type AuthState = Signal<Option<Option<UserInfo>>>;

#[component]
fn Landing() -> Element {
    #[cfg(feature = "landing")]
    rsx! { LandingPage {} }
    #[cfg(not(feature = "landing"))]
    rsx! {}
}

#[component]
fn NotFound(route: Vec<String>) -> Element {
    rsx! {
        h1 { "404 - Page Not Found" }
    }
}

#[derive(Debug, Clone, Routable, PartialEq, MotionTransitions)]
#[rustfmt::skip]
enum Route {
    #[layout(Layout)]

        // Setup — only accessible when setup is NOT yet complete.
        #[layout(RequireSetupIncomplete)]
            #[route("/setup")]
            Setup {},
        #[end_layout]

        // All remaining routes — only accessible after setup IS complete.
        #[layout(RequireSetupComplete)]
            #[route("/login")]
            Login {},
            #[route("/register")]
            Register {},
            #[route("/invitations/accept/:token")]
            InvitationAccept { token: String },
            #[route("/verify-email/pending")]
            VerifyEmailPending {},
            #[route("/verify-email/:token")]
            VerifyEmailConfirm { token: String },

            #[layout(RequireAuth)]
                // Workspace selection — accessible to any authenticated user.
                #[route("/select-workspace")]
                SelectWorkspace {},

                // All routes below additionally require a workspace to be selected.
                #[layout(RequireWorkspace)]
                    #[route("/dashboard")]
                    Dashboard {},

                    #[route("/activities")]
                    Activities {},

                    #[route("/timesheets")]
                    Timesheets {},

                    #[route("/tags")]
                    Tags {},

                    #[route("/settings")]
                    Settings {},

                    #[layout(RequireAdmin)]
                        #[route("/developer/database")]
                        Database {},
                    #[end_layout]
                #[end_layout]
            #[end_layout]
        #[end_layout]

    #[end_layout]

    #[route("/")]
    Landing {},
    #[route("/:..route")]
    NotFound { route: Vec<String> },
}

#[cfg(not(feature = "server"))]
fn main() {
    dotenvy::from_filename_override(".env").ok();
    dioxus::launch(App);
}

/// Handles the Microsoft OAuth2 callback for SMTP configuration.
///
/// Microsoft redirects here after the user grants SMTP.Send permission.
/// The handler exchanges the authorization `code` for a refresh token and
/// stores it in the admin database, then redirects back to the settings page.
#[cfg(feature = "server")]
async fn oauth2_callback_handler(
    axum::extract::Query(params): axum::extract::Query<
        std::collections::HashMap<String, String>,
    >,
) -> axum::response::Response {
    use axum::response::IntoResponse as _;

    let code = params.get("code").cloned().unwrap_or_default();
    let state = params.get("state").cloned().unwrap_or_default();

    match zeitrak::smtp_oauth2::complete_microsoft_oauth2(code, state).await {
        Ok(()) => axum::response::Redirect::to("/settings?smtp=authorized").into_response(),
        Err(e) => {
            let msg = urlencoding::encode(&e.to_string()).into_owned();
            axum::response::Redirect::to(&format!("/settings?smtp=error&msg={msg}"))
                .into_response()
        }
    }
}

#[cfg(feature = "server")]
#[tokio::main]
async fn main() {
    dotenvy::from_filename_override(".env").ok();

    // Ensure the admin database exists and is fully migrated before serving.
    zeitrak::setup::init_admin_db()
        .await
        .expect("failed to initialise admin database");

    let address = dioxus::cli_config::fullstack_address_or_localhost();

    let session_store = tower_sessions::MemoryStore::default();
    let session_layer = tower_sessions::SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_same_site(tower_sessions::cookie::SameSite::Lax);

    let router = axum::Router::new()
        .route(
            "/api/smtp/oauth2/callback",
            axum::routing::get(oauth2_callback_handler),
        )
        .serve_dioxus_application(dioxus_server::ServeConfig::new(), App)
        .layer(session_layer);

    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    axum::serve(listener, router.into_make_service())
        .await
        .unwrap();
}

#[component]
fn App() -> Element {
    // Global auth state — available to every component in the tree.
    use_context_provider(|| Signal::new(None::<Option<UserInfo>>));
    use_init_i18n(ui::i18n::i18n_config);

    let resolver: TransitionVariantResolver<Route> = std::rc::Rc::new(|from, to| {
        fn idx(route: &Route) -> i32 {
            match route {
                Route::Login { .. } => 0,
                Route::Setup { .. } => 1,
                Route::Register { .. } => 1,
                Route::InvitationAccept { .. } => 1,
                Route::VerifyEmailPending { .. } => 1,
                Route::VerifyEmailConfirm { .. } => 1,
                Route::SelectWorkspace { .. } => 2,
                Route::Dashboard { .. } => 3,
                Route::Activities { .. } => 4,
                Route::Timesheets { .. } => 5,
                Route::Tags { .. } => 6,
                Route::Settings { .. } => 7,
                Route::Database { .. } => 8,
                _ => -1,
            }
        }
        let from_idx = idx(from);
        let to_idx = idx(to);
        if from_idx != -1 && to_idx != -1 {
            if to_idx > from_idx {
                TransitionVariant::SlideLeft
            } else if to_idx < from_idx {
                TransitionVariant::SlideRight
            } else {
                TransitionVariant::Fade
            }
        } else {
            to.get_transition()
        }
    });
    use_context_provider(|| resolver);

    let tween = use_signal(|| Tween {
        duration: std::time::Duration::from_millis(500),
        easing: easer::functions::Cubic::ease_in_out,
    });
    use_context_provider(|| tween);

    rsx! {
        GlobalStyles {}
        document::Title { "Zeitrak" }
        document::Meta { name: "description", content: "Zeitrak" }
        document::Meta { name: "viewport", content: "width=device-width, initial-scale=1" }
        document::Link { rel: "icon", href: FAVICON }
        document::Link {
            rel: "preconnect",
            href: "https://fonts.googleapis.com"
        }
        document::Link {
            rel: "preconnect",
            href: "https://fonts.gstatic.com",
            crossorigin: "anonymous"
        }
        document::Link {
            rel: "stylesheet",
            href: "https://fonts.googleapis.com/css2?family=Instrument+Serif:ital@0;1&family=Manrope:wght@400;500;600;700&family=JetBrains+Mono:wght@400;500&display=swap"
        }

        Router::<Route> {}
    }
}

// ── Top-level layout ──────────────────────────────────────────────────────────

#[component]
fn Layout() -> Element {
    let mut auth: AuthState = use_context();
    let mut i18n = i18n();

    // Provide toast context for all descendant views.
    use_context_provider(|| Signal::new(Vec::<ToastMessage>::new()));

    // Provide sidebar open/collapsed state.
    // On WASM: default false (hidden), then open on first render if viewport is wide.
    // On server: default true (always expanded for SSR).
    #[cfg(target_arch = "wasm32")]
    {
        let mut sidebar_open: SidebarOpen = use_context_provider(|| Signal::new(false));
        use_effect(move || {
            let is_desktop = js_sys::eval("window.innerWidth >= 768")
                .ok()
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            sidebar_open.set(is_desktop);
        });
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _: SidebarOpen = use_context_provider(|| Signal::new(true));
    }

    // Provide global running-timer context for Sidebar, Dashboard, and Timesheets.
    let mut running: RunningTimer =
        use_context_provider(|| Signal::new(None::<api::timesheet::TimesheetDto>));

    // Provide entity caches — views read from these to avoid the empty-then-loaded flash.
    let mut activities_cache: ActivitiesCache = use_context_provider(|| Signal::new(Vec::new()));
    let mut tags_cache: TagsCache = use_context_provider(|| Signal::new(Vec::new()));
    let mut timesheets_cache: TimesheetsCache = use_context_provider(|| Signal::new(Vec::new()));

    // Provide user and workspace settings with sane defaults; refreshed after login.
    let mut user_settings: UserSettings = use_context_provider(|| {
        Signal::new(UserSettingsDto {
            timezone: "Europe/Berlin".to_string(),
            date_format: "%Y-%m-%d".to_string(),
            language: "en".to_string(),
        })
    });
    let mut workspace_settings: WorkspaceSettings = use_context_provider(|| {
        Signal::new(WorkspaceSettingsDto {
            name: None,
            timezone: "Europe/Berlin".to_string(),
            date_format: "%Y-%m-%d".to_string(),
            currency: "EUR".to_string(),
            week_start: "monday".to_string(),
        })
    });

    // Provide shared elapsed-seconds counter — updated by one coroutine, read everywhere.
    #[cfg(target_arch = "wasm32")]
    let mut elapsed: RunningElapsed = use_context_provider(|| Signal::new(0u64));
    #[cfg(not(target_arch = "wasm32"))]
    let _elapsed: RunningElapsed = use_context_provider(|| Signal::new(0u64));

    // Single coroutine that owns the tick; computes immediately then every second.
    #[cfg(target_arch = "wasm32")]
    {
        let _timer = use_coroutine(move |_: UnboundedReceiver<()>| async move {
            loop {
                if let Some(ref ts) = *running.read() {
                    let start_ms = js_sys::Date::parse(&ts.start_time);
                    if !start_ms.is_nan() {
                        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                        let secs = ((js_sys::Date::now() - start_ms) / 1000.0).max(0.0) as u64;
                        elapsed.set(secs);
                    }
                } else {
                    elapsed.set(0);
                }
                gloo_timers::future::TimeoutFuture::new(1_000).await;
            }
        });
    }

    // Fetch session auth state once when the layout mounts.
    use_resource(move || async move {
        let user = api::auth::get_current_user().await.ok().flatten();
        auth.set(Some(user));
    });

    // Re-fetch running timer, settings, and entity caches whenever auth/workspace changes.
    use_resource(move || async move {
        let _ = auth.read(); // subscribe — re-runs when auth changes
        if let Ok(r) = api::timesheet::running_timesheet().await {
            running.set(r);
        }
        if let Ok(s) = api::settings::get_user_settings().await {
            let lang_id = match s.language.as_str() {
                "de" => langid!("de-DE"),
                _ => langid!("en-US"),
            };
            i18n.set_language(lang_id);
            user_settings.set(s);
        }
        if let Ok(s) = api::settings::get_workspace_settings().await {
            workspace_settings.set(s);
        }
        // Pre-populate entity caches so views render with data immediately on mount.
        if let Ok(list) = api::activity::list_activities().await {
            activities_cache.set(list);
        }
        if let Ok(list) = api::timesheet_tag::list_tags().await {
            tags_cache.set(list);
        }
        if let Ok(list) = api::timesheet::list_timesheets().await {
            timesheets_cache.set(list);
        }
    });

    let route: Route = use_route();
    let view_title = match &route {
        Route::Dashboard {} => tid!("layout-title-dashboard"),
        Route::Activities {} => tid!("layout-title-activities"),
        Route::Timesheets {} => tid!("layout-title-timesheets"),
        Route::Tags {} => tid!("layout-title-tags"),
        Route::Settings {} => tid!("layout-title-settings"),
        Route::Database {} => tid!("layout-title-developer"),
        Route::SelectWorkspace {} => tid!("layout-title-workspaces"),
        Route::Login {} => tid!("layout-title-login"),
        Route::Setup {} => tid!("layout-title-setup"),
        Route::Register {} => tid!("layout-title-register"),
        Route::InvitationAccept { .. } => tid!("layout-title-accept-invitation"),
        Route::VerifyEmailPending {} => tid!("layout-title-verify-email"),
        Route::VerifyEmailConfirm { .. } => tid!("layout-title-verify-email"),
        Route::Landing {} => String::new(),
        Route::NotFound { .. } => tid!("layout-title-not-found"),
    };

    // Build "Workspace / View" title; show only the view name while on non-workspace routes.
    let page_title = {
        let ws_name = workspace_settings.read().name.clone();
        let skip_prefix = matches!(
            &route,
            Route::Landing {}
                | Route::Login {}
                | Route::Setup {}
                | Route::Register {}
                | Route::InvitationAccept { .. }
                | Route::VerifyEmailPending {}
                | Route::VerifyEmailConfirm { .. }
                | Route::SelectWorkspace {}
                | Route::NotFound { .. }
        );
        match ws_name.filter(|_| !skip_prefix) {
            Some(ws) if !view_title.is_empty() => format!("{ws} / {view_title}"),
            _ => view_title,
        }
    };

    rsx! {
        div { class: "app-shell",
            Sidebar {}
            div { class: "app-right",
                Header { title: page_title }
                main { class: "app-main",
                    AnimatedOutlet::<Route> {}
                }
            }
        }
        ToastStack {}
    }
}

// ── Route guards ──────────────────────────────────────────────────────────────

/// Redirects to /setup when setup is NOT complete; renders children otherwise.
#[component]
fn RequireSetupComplete() -> Element {
    let nav = use_navigator();
    let complete = use_resource(|| async { api::setup::is_setup_complete().await });

    match complete.value().cloned() {
        None => rsx! {},
        Some(Ok(true)) => rsx! { Outlet::<Route> {} },
        Some(Ok(false)) | Some(Err(_)) => {
            nav.replace(Route::Setup {});
            rsx! {}
        }
    }
}

/// Redirects to /login when setup IS complete; renders children otherwise.
#[component]
fn RequireSetupIncomplete() -> Element {
    let nav = use_navigator();
    let complete = use_resource(|| async { api::setup::is_setup_complete().await });

    match complete.value().cloned() {
        None => rsx! {},
        Some(Err(_)) | Some(Ok(false)) => rsx! { Outlet::<Route> {} },
        Some(Ok(true)) => {
            nav.replace(Route::Login {});
            rsx! {}
        }
    }
}

/// Reads global auth state. Shows nothing while loading, redirects to /login
/// when unauthenticated, and renders the outlet when authenticated.
#[component]
fn RequireAuth() -> Element {
    let nav = use_navigator();
    let auth: AuthState = use_context();

    match auth.cloned() {
        None => rsx! {},
        Some(None) => {
            nav.replace(Route::Login {});
            rsx! {}
        }
        Some(Some(_)) => rsx! { AnimatedOutlet::<Route> {} },
    }
}

/// Redirects to /select-workspace when the authenticated user has not yet
/// chosen a workspace for this session.
#[component]
fn RequireWorkspace() -> Element {
    let nav = use_navigator();
    let auth: AuthState = use_context();

    match auth.cloned() {
        Some(Some(user)) if user.workspace_id.is_some() => {
            rsx! { AnimatedOutlet::<Route> {} }
        }
        Some(Some(_)) => {
            // Authenticated but no workspace selected yet.
            nav.replace(Route::SelectWorkspace {});
            rsx! {}
        }
        // Loading or unauthenticated — RequireAuth above handles these.
        _ => rsx! {},
    }
}

/// Redirects to /dashboard when the authenticated user is not an admin.
#[component]
fn RequireAdmin() -> Element {
    let nav = use_navigator();
    let auth: AuthState = use_context();

    match auth.cloned() {
        Some(Some(user)) if user.is_admin => rsx! { Outlet::<Route> {} },
        Some(Some(_)) => {
            nav.replace(Route::Dashboard {});
            rsx! {}
        }
        // Loading or unauthenticated — RequireAuth handles these cases above us.
        _ => rsx! {},
    }
}
