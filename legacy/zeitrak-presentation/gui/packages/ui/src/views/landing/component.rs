use api::waitlist::join_waitlist;
use dioxus::prelude::*;

#[component]
pub fn LandingPage() -> Element {
    let mut email = use_signal(String::new);
    let mut submitted = use_signal(|| false);
    let mut error_msg = use_signal(String::new);

    let on_submit = move |_| async move {
        let addr = email.read().trim().to_string();
        if addr.is_empty() {
            return;
        }
        match join_waitlist(addr).await {
            Ok(()) => submitted.set(true),
            Err(e) => error_msg.set(e.to_string()),
        }
    };

    rsx! {
        div { class: "lp",

            // ── Navbar ────────────────────────────────────────────────────────
            nav { class: "lp-nav",
                div { class: "lp-nav-inner",
                    a { class: "lp-nav-brand", href: "/",
                        div { class: "lp-nav-brand-mark", "Z" }
                        span { class: "lp-nav-brand-name", "Zeitrak" }
                    }
                    div { class: "lp-nav-links",
                        a { class: "lp-nav-link", href: "#features", "Features" }
                        a { class: "lp-nav-link", href: "#trackmap", "Track Map" }
                        a { class: "lp-nav-link", href: "#plugins", "Plugins" }
                        a { class: "lp-nav-link", href: "#open-source", "Open Source" }
                    }
                    div { class: "lp-nav-actions",
                        a { class: "lp-nav-signin", href: "/login", "Sign in" }
                        a { class: "lp-nav-cta", href: "/register", "Get early access" }
                    }
                }
            }

            // ── Hero ──────────────────────────────────────────────────────────
            section { class: "lp-hero",
                div { class: "lp-hero-copy",
                    div { class: "lp-hero-eyebrow",
                        span { class: "lp-hero-eyebrow-dot" }
                        "Local-first time tracking"
                    }
                    h1 { class: "lp-hero-headline",
                        "Every hour,"
                        br {}
                        em { "a station." }
                    }
                    p { class: "lp-hero-sub",
                        "Zeitrak turns your workday into a clear, ordered timeline — every project,
                        every interruption, every break, logged like stops on a rail journey."
                    }
                    div { class: "lp-hero-actions",
                        a { class: "lp-btn-primary", href: "/register", "Get early access" }
                        a { class: "lp-btn-secondary", href: "/login", "Sign in" }
                    }
                    div { class: "lp-hero-meta",
                        div { class: "lp-hero-meta-item", "✦ Open source" }
                        div { class: "lp-hero-meta-item", "✦ Self-hostable" }
                        div { class: "lp-hero-meta-item", "✦ Local-first" }
                        div { class: "lp-hero-meta-item", "✦ Free forever" }
                    }
                }

                // ── Track demo card (hero right) ─────────────────────────────
                div { class: "lp-hero-art",
                    div { class: "lp-track-demo",
                        div { class: "lp-track-demo-header",
                            div {
                                div { class: "lp-track-demo-title", "Wednesday, May 14" }
                                div { class: "lp-track-demo-sub", "3 stations · 3h 17min" }
                            }
                            div { class: "lp-manifest",
                                div { class: "lp-manifest-item",
                                    div { class: "lp-manifest-line lp-manifest-line-main" }
                                    "Main Line"
                                }
                                div { class: "lp-manifest-item",
                                    div { class: "lp-manifest-line lp-manifest-line-side" }
                                    "Sidetrack"
                                }
                            }
                        }
                        div { class: "lp-station-wrap",
                            svg {
                                class: "lp-rail-svg",
                                view_box: "0 0 30 340",
                                xmlns: "http://www.w3.org/2000/svg",
                                line { x1: "15", y1: "0", x2: "15", y2: "340", stroke: "#0d631b", stroke_width: "2.5", opacity: "0.2" }
                                circle { cx: "15", cy: "24", r: "7", fill: "#0d631b" }
                                path { d: "M15 138 Q22 150 28 158", stroke: "#bfcaba", stroke_width: "2", stroke_dasharray: "4,3", fill: "none" }
                                circle { cx: "15", cy: "208", r: "7", fill: "white", stroke: "#0d631b", stroke_width: "3" }
                            }
                            div { class: "lp-station-entry", style: "top: 8px;",
                                div { class: "lp-entry-row",
                                    div {
                                        div { class: "lp-entry-tag", "Deep Work" }
                                        div { class: "lp-entry-name", "Design System Architecture" }
                                        div { class: "lp-entry-time", "09:00 → 11:20" }
                                    }
                                    div { class: "lp-entry-dur", "02:15" }
                                }
                                div { class: "lp-entry-chips",
                                    span { class: "lp-chip", "Focus" }
                                    span { class: "lp-chip", "Architecture" }
                                }
                            }
                            div { class: "lp-station-entry lp-station-entry-side", style: "top: 120px;",
                                div { class: "lp-entry-row",
                                    div {
                                        div { class: "lp-entry-tag", "Sidetrack" }
                                        div { class: "lp-entry-name", "Client Sync" }
                                    }
                                    div { class: "lp-entry-dur", "15m" }
                                }
                            }
                            div { class: "lp-station-entry lp-station-entry-active", style: "top: 192px;",
                                div { class: "lp-entry-row",
                                    div {
                                        div { class: "lp-entry-tag lp-entry-tag-active", "Current Station" }
                                        div { class: "lp-entry-name", "Track Map Rendering" }
                                    }
                                    div { class: "lp-active-timer", "00:47:23" }
                                }
                                div { class: "lp-entry-buttons",
                                    button { class: "lp-halt-btn", "Halt" }
                                    button { class: "lp-side-btn", "Sidetrack" }
                                }
                            }
                        }
                    }
                }
            }

            // ── Features section ──────────────────────────────────────────────
            div { id: "features", class: "lp-features-bg",
                div { class: "lp-features-inner",
                    span { class: "lp-section-tag", "Why Zeitrak" }
                    h2 { class: "lp-section-title", "Everything your workday needs." }
                    p { class: "lp-section-sub",
                        "Built for makers, consultants, and teams who need precise records
                        without the overhead of complex project management tools."
                    }
                    div { class: "lp-feature-grid",
                        div { class: "lp-feature-card",
                            div { class: "lp-feature-icon lp-feature-icon-green",
                                svg { xmlns: "http://www.w3.org/2000/svg", view_box: "0 0 24 24", fill: "none", stroke: "#0d631b", stroke_width: "2", stroke_linecap: "round", stroke_linejoin: "round",
                                    circle { cx: "12", cy: "12", r: "10" }
                                    polyline { points: "12 6 12 12 16 14" }
                                }
                            }
                            p { class: "lp-feature-title", "Activity-first tracking" }
                            p { class: "lp-feature-desc",
                                "Define activities once — a project, a client, a habit. Start a
                                timer in one click. No configuration per entry."
                            }
                        }
                        div { class: "lp-feature-card",
                            div { class: "lp-feature-icon lp-feature-icon-blue",
                                svg { xmlns: "http://www.w3.org/2000/svg", view_box: "0 0 24 24", fill: "none", stroke: "#00598f", stroke_width: "2", stroke_linecap: "round", stroke_linejoin: "round",
                                    path { d: "M9 18l6-6-6-6" }
                                    line { x1: "4", y1: "12", x2: "20", y2: "12" }
                                }
                            }
                            p { class: "lp-feature-title", "Sidetrack distractions" }
                            p { class: "lp-feature-desc",
                                "An interrupt arrives. Log it as a sidetrack without stopping your
                                main session. Resume right where you left off."
                            }
                        }
                        div { class: "lp-feature-card",
                            div { class: "lp-feature-icon lp-feature-icon-green",
                                svg { xmlns: "http://www.w3.org/2000/svg", view_box: "0 0 24 24", fill: "none", stroke: "#0d631b", stroke_width: "2", stroke_linecap: "round", stroke_linejoin: "round",
                                    rect { x: "2", y: "3", width: "20", height: "14", rx: "2" }
                                    line { x1: "8", y1: "21", x2: "16", y2: "21" }
                                    line { x1: "12", y1: "17", x2: "12", y2: "21" }
                                }
                            }
                            p { class: "lp-feature-title", "Local-first, always yours" }
                            p { class: "lp-feature-desc",
                                "Your data lives in a local SQLite file. No internet required. Sync
                                to a server when you want — but you never depend on one."
                            }
                        }
                        div { class: "lp-feature-card",
                            div { class: "lp-feature-icon lp-feature-icon-warm",
                                svg { xmlns: "http://www.w3.org/2000/svg", view_box: "0 0 24 24", fill: "none", stroke: "#92400e", stroke_width: "2", stroke_linecap: "round", stroke_linejoin: "round",
                                    path { d: "M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" }
                                }
                            }
                            p { class: "lp-feature-title", "Event-sourced history" }
                            p { class: "lp-feature-desc",
                                "Every action is an immutable event. Travel back in time, replay
                                your day, and audit exactly what happened when."
                            }
                        }
                        div { class: "lp-feature-card",
                            div { class: "lp-feature-icon lp-feature-icon-blue",
                                svg { xmlns: "http://www.w3.org/2000/svg", view_box: "0 0 24 24", fill: "none", stroke: "#00598f", stroke_width: "2", stroke_linecap: "round", stroke_linejoin: "round",
                                    path { d: "M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z" }
                                }
                            }
                            p { class: "lp-feature-title", "Plugin ecosystem" }
                            p { class: "lp-feature-desc",
                                "Export to DATEV, sync with Jira, or build your own. Plugins run
                                as WASM modules — sandboxed, fast, and distributable."
                            }
                        }
                        div { class: "lp-feature-card",
                            div { class: "lp-feature-icon lp-feature-icon-green",
                                svg { xmlns: "http://www.w3.org/2000/svg", view_box: "0 0 24 24", fill: "none", stroke: "#0d631b", stroke_width: "2", stroke_linecap: "round", stroke_linejoin: "round",
                                    line { x1: "18", y1: "20", x2: "18", y2: "10" }
                                    line { x1: "12", y1: "20", x2: "12", y2: "4" }
                                    line { x1: "6", y1: "20", x2: "6", y2: "14" }
                                }
                            }
                            p { class: "lp-feature-title", "Rich reports" }
                            p { class: "lp-feature-desc",
                                "Daily, weekly, and monthly breakdowns with charts. See how you
                                really spend your time — and where to improve."
                            }
                        }
                    }
                }
            }

            // ── Track Map section ─────────────────────────────────────────────
            section { id: "trackmap", class: "lp-section",
                div { class: "lp-trackmap-grid",
                    div { class: "lp-trackmap-visual",
                        div { class: "lp-tm-header",
                            div { class: "lp-tm-title", "Wednesday, May 14" }
                            div { class: "lp-tm-sub", "4 stations · 6h 02min tracked" }
                        }
                        div { class: "lp-tm-track",
                            div { class: "lp-tm-line" }
                            div { class: "lp-tm-node lp-tm-node-filled", style: "top: 14px;" }
                            div { class: "lp-tm-entry",
                                div { class: "lp-tm-entry-top",
                                    div {
                                        div { class: "lp-tm-entry-tag", "Deep Work" }
                                        div { class: "lp-tm-entry-name", "Design System Architecture" }
                                    }
                                    div { class: "lp-tm-entry-dur", "02:15" }
                                }
                                div { class: "lp-tm-entry-time", "09:00 → 11:15" }
                            }
                            div { class: "lp-tm-entry lp-tm-entry-side",
                                div { class: "lp-tm-entry-top",
                                    div {
                                        div { class: "lp-tm-entry-tag", "Sidetrack" }
                                        div { class: "lp-tm-entry-name", "Client Sync" }
                                    }
                                    div { class: "lp-tm-entry-dur", "15m" }
                                }
                                div { class: "lp-tm-entry-time", "11:15 → 11:30" }
                            }
                            div { class: "lp-tm-node lp-tm-node-filled", style: "top: 189px;" }
                            div { class: "lp-tm-entry",
                                div { class: "lp-tm-entry-top",
                                    div {
                                        div { class: "lp-tm-entry-tag", "Development" }
                                        div { class: "lp-tm-entry-name", "API Integration" }
                                    }
                                    div { class: "lp-tm-entry-dur", "01:47" }
                                }
                                div { class: "lp-tm-entry-time", "12:30 → 14:17" }
                            }
                            div { class: "lp-tm-node lp-tm-node-active", style: "top: 277px;" }
                            div { class: "lp-tm-entry lp-tm-entry-active",
                                div { class: "lp-tm-entry-top",
                                    div {
                                        div { class: "lp-tm-entry-tag lp-tm-entry-tag-active", "Current Station" }
                                        div { class: "lp-tm-entry-name", "Track Map Rendering" }
                                    }
                                    div { class: "lp-tm-active-time", "00:47:23" }
                                }
                                div { class: "lp-tm-actions",
                                    button { class: "lp-tm-btn", "Halt" }
                                    button { class: "lp-tm-btn lp-tm-btn-green", "Sidetrack" }
                                }
                            }
                        }
                    }
                    div { class: "lp-trackmap-explain",
                        span { class: "lp-section-tag", "Track Map" }
                        h2 { class: "lp-section-title",
                            "Your whole day,"
                            br {}
                            em { "at a glance." }
                        }
                        p { class: "lp-section-sub",
                            "The Track Map shows every session, sidetrack, and break as stops on a
                            timeline. See patterns emerge. Understand where your hours actually go."
                        }
                        div { class: "lp-explain-items",
                            div { class: "lp-explain-item",
                                div { class: "lp-explain-icon lp-explain-icon-green",
                                    svg { xmlns: "http://www.w3.org/2000/svg", view_box: "0 0 24 24", fill: "none", stroke: "#0d631b", stroke_width: "2.5", stroke_linecap: "round",
                                        circle { cx: "12", cy: "12", r: "4" }
                                        line { x1: "12", y1: "2", x2: "12", y2: "7" }
                                        line { x1: "12", y1: "17", x2: "12", y2: "22" }
                                    }
                                }
                                div {
                                    div { class: "lp-explain-title", "Stations on the main line" }
                                    div { class: "lp-explain-desc",
                                        "Each activity session is a station. Completed stations are
                                        filled; the current one pulses."
                                    }
                                }
                            }
                            div { class: "lp-explain-item",
                                div { class: "lp-explain-icon lp-explain-icon-gray",
                                    svg { xmlns: "http://www.w3.org/2000/svg", view_box: "0 0 24 24", fill: "none", stroke: "#5c6057", stroke_width: "2", stroke_linecap: "round",
                                        path { d: "M5 12h14" }
                                        path { d: "M12 5l7 7-7 7" }
                                    }
                                }
                                div {
                                    div { class: "lp-explain-title", "Sidetracks off the main line" }
                                    div { class: "lp-explain-desc",
                                        "Interruptions branch off the main track. They close
                                        automatically when you resume your primary activity."
                                    }
                                }
                            }
                            div { class: "lp-explain-item",
                                div { class: "lp-explain-icon lp-explain-icon-blue",
                                    svg { xmlns: "http://www.w3.org/2000/svg", view_box: "0 0 24 24", fill: "none", stroke: "#00598f", stroke_width: "2", stroke_linecap: "round",
                                        rect { x: "3", y: "4", width: "18", height: "18", rx: "2" }
                                        line { x1: "16", y1: "2", x2: "16", y2: "6" }
                                        line { x1: "8", y1: "2", x2: "8", y2: "6" }
                                        line { x1: "3", y1: "10", x2: "21", y2: "10" }
                                    }
                                }
                                div {
                                    div { class: "lp-explain-title", "Tap any entry to edit" }
                                    div { class: "lp-explain-desc",
                                        "Adjust start times, add notes, or reassign activities.
                                        Your map stays accurate without friction."
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // ── Event Log section ─────────────────────────────────────────────
            section { class: "lp-section",
                div { class: "lp-eventlog-grid",
                    div {
                        span { class: "lp-section-tag", "Event Log" }
                        h2 { class: "lp-section-title",
                            "Nothing is ever"
                            br {}
                            em { "lost." }
                        }
                        p { class: "lp-section-sub",
                            "Every action — start, pause, sidetrack, resume — is stored as an
                            immutable event. Edit a session and the history still shows what
                            really happened."
                        }
                        div { class: "lp-eventsourcing-box",
                            span { class: "lp-eventsourcing-label", "Why event sourcing?" }
                            div { class: "lp-eventsourcing-item",
                                span { class: "lp-eventsourcing-check", "✓" }
                                "Replay your exact day at any point in time"
                            }
                            div { class: "lp-eventsourcing-item",
                                span { class: "lp-eventsourcing-check", "✓" }
                                "Audit log for billing and compliance"
                            }
                            div { class: "lp-eventsourcing-item",
                                span { class: "lp-eventsourcing-check", "✓" }
                                "Conflict-free sync across devices"
                            }
                        }
                    }
                    div { class: "lp-eventlog-visual",
                        div { class: "lp-el-header",
                            span { class: "lp-el-title", "Event Stream" }
                            span { class: "lp-el-badge", "Live" }
                        }
                        div { class: "lp-el-entries",
                            div { class: "lp-el-entry",
                                span { class: "lp-el-time", "09:00:00" }
                                div { class: "lp-el-dot lp-el-dot-started" }
                                div {
                                    div { class: "lp-el-event-name", "SessionStarted" }
                                    div { class: "lp-el-event-detail", "activity: Design System Architecture" }
                                }
                            }
                            div { class: "lp-el-entry",
                                span { class: "lp-el-time", "11:15:02" }
                                div { class: "lp-el-dot lp-el-dot-paused" }
                                div {
                                    div { class: "lp-el-event-name", "SessionPaused" }
                                    div { class: "lp-el-event-detail", "duration: 02:15:02" }
                                }
                            }
                            div { class: "lp-el-entry",
                                span { class: "lp-el-time", "11:15:09" }
                                div { class: "lp-el-dot lp-el-dot-side" }
                                div {
                                    div { class: "lp-el-event-name", "SidetrackStarted" }
                                    div { class: "lp-el-event-detail", "reason: Client Sync" }
                                }
                            }
                            div { class: "lp-el-entry",
                                span { class: "lp-el-time", "11:30:44" }
                                div { class: "lp-el-dot lp-el-dot-resumed" }
                                div {
                                    div { class: "lp-el-event-name", "SidetrackEnded" }
                                    div { class: "lp-el-event-detail", "duration: 00:15:35" }
                                }
                            }
                            div { class: "lp-el-entry lp-el-entry-last",
                                span { class: "lp-el-time lp-el-time-active", "now" }
                                div { class: "lp-el-dot lp-el-dot-active" }
                                div {
                                    div { class: "lp-el-event-name lp-el-event-name-active", "SessionStarted" }
                                    div { class: "lp-el-event-detail", "activity: Track Map Rendering" }
                                }
                            }
                        }
                    }
                }
            }

            // ── Plugins section ───────────────────────────────────────────────
            div { id: "plugins", class: "lp-plugins-bg",
                div { class: "lp-plugins-inner",
                    span { class: "lp-section-tag", "Plugin Ecosystem" }
                    h2 { class: "lp-section-title", "Extend without limits." }
                    p { class: "lp-section-sub",
                        "Zeitrak plugins are WASM modules — sandboxed, fast, and buildable in
                        any language. Ship your plugin to anyone running Zeitrak."
                    }
                    div { class: "lp-plugins-grid",
                        div { class: "lp-plugin-card",
                            span { class: "lp-plugin-badge", "Official" }
                            div { class: "lp-plugin-icon", "⏱" }
                            div { class: "lp-plugin-title", "Worktime Compliance" }
                            div { class: "lp-plugin-desc",
                                "Automatically flags sessions that exceed local labour law limits.
                                Alerts before you breach break-time regulations."
                            }
                        }
                        div { class: "lp-plugin-card",
                            span { class: "lp-plugin-badge", "Official" }
                            div { class: "lp-plugin-icon", "📊" }
                            div { class: "lp-plugin-title", "DATEV Export" }
                            div { class: "lp-plugin-desc",
                                "Generate DATEV-compatible CSV files from your timesheets in one
                                click. Ready for your accountant."
                            }
                        }
                        div { class: "lp-plugin-card",
                            span { class: "lp-plugin-badge", "Community" }
                            div { class: "lp-plugin-icon", "🔗" }
                            div { class: "lp-plugin-title", "Project Management Sync" }
                            div { class: "lp-plugin-desc",
                                "Sync sessions to Linear, Jira, or GitHub Issues. Log time against
                                tickets without leaving Zeitrak."
                            }
                        }
                    }
                    div { class: "lp-plugin-build",
                        div {
                            div { class: "lp-plugin-build-title", "Build your own plugin" }
                            div { class: "lp-plugin-build-desc",
                                "Write in Rust, Go, or any WASM-compilable language. Access the
                                full Zeitrak event stream. Publish to the community registry."
                            }
                        }
                        a { class: "lp-plugin-build-link", href: "#", "Read the plugin docs →" }
                    }
                }
            }

            // ── Reports section ───────────────────────────────────────────────
            section { class: "lp-section",
                div { class: "lp-reports-grid",
                    div {
                        span { class: "lp-section-tag", "Reports" }
                        h2 { class: "lp-section-title",
                            "See where your"
                            br {}
                            em { "hours go." }
                        }
                        p { class: "lp-section-sub",
                            "Daily, weekly, and monthly breakdowns. Activity distribution charts.
                            Billable hours summaries. Everything you need to invoice with confidence."
                        }
                        div { class: "lp-check-list",
                            div { class: "lp-check-item",
                                span { class: "lp-check-icon", "✓" }
                                "Activity and project breakdowns"
                            }
                            div { class: "lp-check-item",
                                span { class: "lp-check-icon", "✓" }
                                "Billable hours and revenue tracking"
                            }
                            div { class: "lp-check-item",
                                span { class: "lp-check-icon", "✓" }
                                "Export to CSV, PDF, or DATEV"
                            }
                            div { class: "lp-check-item",
                                span { class: "lp-check-icon", "✓" }
                                "Custom date ranges"
                            }
                        }
                    }
                    div { class: "lp-reports-visual",
                        div { class: "lp-stat-grid",
                            div { class: "lp-stat-card",
                                div { class: "lp-stat-label", "This Month" }
                                div { class: "lp-stat-value",
                                    "164"
                                    span { ".5 hrs" }
                                }
                                div { class: "lp-stat-trend", "↑ 12% vs last month" }
                            }
                            div { class: "lp-stat-card",
                                div { class: "lp-stat-label", "Billable" }
                                div { class: "lp-stat-value",
                                    "€14"
                                    span { ",280" }
                                }
                                div { class: "lp-stat-trend", "↑ 8% vs last month" }
                            }
                            div { class: "lp-stat-card",
                                div { class: "lp-stat-label", "Daily Avg" }
                                div { class: "lp-stat-value",
                                    "7"
                                    span { ".8 hrs" }
                                }
                                div { class: "lp-stat-trend lp-stat-trend-neutral", "vs 8h target" }
                            }
                        }
                        div { class: "lp-charts-row",
                            // Donut chart — activity mix
                            div { class: "lp-chart-card",
                                div { class: "lp-chart-title", "Activity Mix" }
                                svg {
                                    view_box: "0 0 120 120",
                                    width: "120",
                                    height: "120",
                                    // circumference of r=45: 282.7
                                    // dashoffset 70.7 = start from top (25% shift)
                                    circle { cx: "60", cy: "60", r: "45", fill: "none", stroke: "#eeeee9", stroke_width: "18" }
                                    circle { cx: "60", cy: "60", r: "45", fill: "none", stroke: "#0d631b",
                                        stroke_width: "18", stroke_dasharray: "127.2 155.5", stroke_dashoffset: "70.7" }
                                    circle { cx: "60", cy: "60", r: "45", fill: "none", stroke: "#00598f",
                                        stroke_width: "18", stroke_dasharray: "84.8 197.9", stroke_dashoffset: "-56.5" }
                                    circle { cx: "60", cy: "60", r: "45", fill: "none", stroke: "#f59e0b",
                                        stroke_width: "18", stroke_dasharray: "42.4 240.3", stroke_dashoffset: "-141.3" }
                                    circle { cx: "60", cy: "60", r: "45", fill: "none", stroke: "#bfcaba",
                                        stroke_width: "18", stroke_dasharray: "28.3 254.4", stroke_dashoffset: "-183.7" }
                                }
                                div { class: "lp-chart-legend",
                                    div { class: "lp-chart-legend-item",
                                        span {
                                            span { class: "lp-chart-legend-dot", style: "background:#0d631b;" }
                                            "Deep Work"
                                        }
                                        span { class: "lp-chart-legend-pct", "45%" }
                                    }
                                    div { class: "lp-chart-legend-item",
                                        span {
                                            span { class: "lp-chart-legend-dot", style: "background:#00598f;" }
                                            "Client Work"
                                        }
                                        span { class: "lp-chart-legend-pct", "30%" }
                                    }
                                    div { class: "lp-chart-legend-item",
                                        span {
                                            span { class: "lp-chart-legend-dot", style: "background:#f59e0b;" }
                                            "Admin"
                                        }
                                        span { class: "lp-chart-legend-pct", "15%" }
                                    }
                                    div { class: "lp-chart-legend-item",
                                        span {
                                            span { class: "lp-chart-legend-dot", style: "background:#bfcaba;" }
                                            "Sidetracks"
                                        }
                                        span { class: "lp-chart-legend-pct", "10%" }
                                    }
                                }
                            }
                            // Bar chart + breakdown
                            div { class: "lp-chart-card",
                                div { class: "lp-chart-header",
                                    div { class: "lp-chart-title", "Weekly Hours" }
                                    div { class: "lp-chart-legend-row",
                                        div { class: "lp-chart-legend-badge",
                                            div { style: "width:8px;height:8px;background:#0d631b;border-radius:2px;" }
                                            "Tracked"
                                        }
                                    }
                                }
                                svg {
                                    view_box: "0 0 200 96",
                                    width: "100%",
                                    height: "96",
                                    // bars grow upward from y=82, labels at y=93
                                    rect { x: "2", y: "10", width: "20", height: "72", rx: "4", fill: "#0d631b", opacity: "0.8" }
                                    rect { x: "30", y: "3", width: "20", height: "79", rx: "4", fill: "#0d631b", opacity: "0.8" }
                                    rect { x: "58", y: "17", width: "20", height: "65", rx: "4", fill: "#0d631b", opacity: "0.8" }
                                    rect { x: "86", y: "5", width: "20", height: "77", rx: "4", fill: "#0d631b", opacity: "0.8" }
                                    rect { x: "114", y: "13", width: "20", height: "69", rx: "4", fill: "#0d631b", opacity: "0.8" }
                                    rect { x: "142", y: "63", width: "20", height: "19", rx: "4", fill: "#dadad5" }
                                    rect { x: "170", y: "82", width: "20", height: "0", rx: "4", fill: "#dadad5" }
                                    // day labels
                                    // (using foreignObject would need a different approach;
                                    //  SVG text is the correct primitive here)
                                }
                                div { class: "lp-breakdown-table",
                                    div { class: "lp-breakdown-header",
                                        span { "Activity" }
                                        span { "Hours" }
                                        span { "%" }
                                    }
                                    div { class: "lp-breakdown-row",
                                        span { "Design System" }
                                        span { "28.5h" }
                                        span { class: "lp-breakdown-pct", "31%" }
                                    }
                                    div { class: "lp-breakdown-row",
                                        span { "Client Work" }
                                        span { "24.0h" }
                                        span { class: "lp-breakdown-pct", "26%" }
                                    }
                                    div { class: "lp-breakdown-row lp-breakdown-row-last",
                                        span { "API Integration" }
                                        span { "18.2h" }
                                        span { class: "lp-breakdown-pct", "20%" }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // ── Open Source section ───────────────────────────────────────────
            div { id: "open-source", class: "lp-opensource-bg",
                span { class: "lp-section-tag", "Open Source" }
                h2 { class: "lp-section-title",
                    "Free forever."
                    br {}
                    em { "Forever yours." }
                }
                p { class: "lp-section-sub", style: "margin: 0 auto;",
                    "Zeitrak is MIT-licensed. Audit the code, self-host it, fork it,
                    contribute to it. No vendor lock-in, ever."
                }
                div { class: "lp-pill-group",
                    div { class: "lp-pill", div { class: "lp-pill-dot" } "MIT License" }
                    div { class: "lp-pill", div { class: "lp-pill-dot" } "Self-hostable" }
                    div { class: "lp-pill", div { class: "lp-pill-dot" } "Local-first SQLite" }
                    div { class: "lp-pill", div { class: "lp-pill-dot" } "REST + Plugin API" }
                    div { class: "lp-pill", div { class: "lp-pill-dot" } "No telemetry" }
                    div { class: "lp-pill", div { class: "lp-pill-dot" } "Docker deploy" }
                }
                div { class: "lp-hero-actions", style: "justify-content: center;",
                    a { class: "lp-btn-primary", href: "/register", "Get early access" }
                    a { class: "lp-btn-secondary", href: "https://github.com/NavilaLabs/zeitrak", "View on GitHub" }
                }
            }

            // ── CTA section ───────────────────────────────────────────────────
            div { class: "lp-cta-bg",
                div { class: "lp-cta-inner",
                    div { class: "lp-cta-eyebrow",
                        span { class: "lp-hero-eyebrow-dot" }
                        "Launching Summer 2026"
                    }
                    h2 { class: "lp-cta-title",
                        "Be the first on"
                        br {}
                        em { "the platform." }
                    }
                    p { class: "lp-cta-sub",
                        "Join the early access list. Get notified when Zeitrak opens.
                        No spam — just one email when we're ready."
                    }
                    if *submitted.read() {
                        p { class: "lp-cta-success", "You're on the list! We'll be in touch." }
                    } else {
                        div { class: "lp-cta-form",
                            input {
                                class: "lp-cta-input",
                                r#type: "email",
                                placeholder: "your@email.com",
                                value: "{email}",
                                oninput: move |e| email.set(e.value()),
                            }
                            button {
                                class: "lp-btn-primary",
                                onclick: on_submit,
                                "Notify me"
                            }
                        }
                        if !error_msg.read().is_empty() {
                            p { class: "lp-cta-error", "{error_msg}" }
                        }
                    }
                    p { class: "lp-cta-note", "Free forever · No credit card required" }
                }
            }

            // ── Footer ────────────────────────────────────────────────────────
            footer { class: "lp-footer",
                div {
                    div { class: "lp-footer-logo",
                        "Zeit" span { "rak" }
                    }
                    div { class: "lp-footer-tagline", "Every hour, a station." }
                }
                div { class: "lp-footer-links",
                    a { class: "lp-footer-link", href: "#features", "Features" }
                    a { class: "lp-footer-link", href: "#plugins", "Plugins" }
                    a { class: "lp-footer-link", href: "/register", "Early access" }
                    a { class: "lp-footer-link", href: "https://github.com/NavilaLabs/zeitrak", "GitHub" }
                    a { class: "lp-footer-link", href: "#", "Docs" }
                }
                div { class: "lp-footer-copy", "© 2026 NavilaLabs" }
            }
        }
    }
}
