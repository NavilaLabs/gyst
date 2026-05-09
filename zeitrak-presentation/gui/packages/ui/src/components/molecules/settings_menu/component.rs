use std::str::FromStr;

use crate::components::atoms::dropdown_menu::{
    DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger,
};
use crate::components::molecules::theme_switcher::Theme;
use dioxus::document::eval;
use dioxus::prelude::*;
use dioxus_free_icons::icons::hi_solid_icons::{
    HiCog, HiDatabase, HiDesktopComputer, HiMoon, HiSun,
};
use dioxus_free_icons::Icon;

type AuthState = Signal<Option<Option<api::auth::UserInfo>>>;

fn apply_theme(theme: Theme) {
    match theme {
        Theme::System => {
            eval("document.documentElement.removeAttribute('data-theme'); localStorage.removeItem('theme')");
        }
        _ => {
            let s = theme.as_str();
            eval(&format!(
                "document.documentElement.setAttribute('data-theme', '{s}'); localStorage.setItem('theme', '{s}')"
            ));
        }
    }
}

#[component]
pub fn SettingsMenu() -> Element {
    let mut current_theme = use_signal(|| Theme::System);
    let auth: AuthState = use_context();
    let is_admin = auth.cloned().flatten().map(|u| u.is_admin).unwrap_or(false);

    use_effect(move || {
        spawn(async move {
            let mut js = eval("dioxus.send(localStorage.getItem('theme') ?? 'system')");
            if let Ok(val) = js.recv::<String>().await {
                let t = Theme::from_str(&val).unwrap_or(Theme::System);
                current_theme.set(t);
                apply_theme(t);
            }
        });
    });

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }

        div { class: "settings-menu",
            DropdownMenu {
                DropdownMenuTrigger {
                    Icon { icon: HiCog, width: 18, height: 18 }
                }
                DropdownMenuContent {
                    div { class: "settings-section-label", "Theme" }
                    DropdownMenuItem {
                        value: Theme::Light,
                        index: 0_usize,
                        on_select: move |t: Theme| { current_theme.set(t); apply_theme(t); },
                        Icon { icon: HiSun, width: 14, height: 14 }
                        span { class: "settings-item-label", "Light" }
                        if *current_theme.read() == Theme::Light {
                            span { class: "settings-item-check", "✓" }
                        }
                    }
                    DropdownMenuItem {
                        value: Theme::Dark,
                        index: 1_usize,
                        on_select: move |t: Theme| { current_theme.set(t); apply_theme(t); },
                        Icon { icon: HiMoon, width: 14, height: 14 }
                        span { class: "settings-item-label", "Dark" }
                        if *current_theme.read() == Theme::Dark {
                            span { class: "settings-item-check", "✓" }
                        }
                    }
                    DropdownMenuItem {
                        value: Theme::System,
                        index: 2_usize,
                        on_select: move |t: Theme| { current_theme.set(t); apply_theme(t); },
                        Icon { icon: HiDesktopComputer, width: 14, height: 14 }
                        span { class: "settings-item-label", "System" }
                        if *current_theme.read() == Theme::System {
                            span { class: "settings-item-check", "✓" }
                        }
                    }
                    if is_admin {
                        div { class: "settings-separator" }
                        DropdownMenuItem {
                            value: "database".to_string(),
                            index: 3_usize,
                            on_select: move |_: String| {
                                navigator().push("/developer/database");
                            },
                            Icon { icon: HiDatabase, width: 14, height: 14 }
                            span { class: "settings-item-label", "Database" }
                        }
                    }
                }
            }
        }
    }
}
