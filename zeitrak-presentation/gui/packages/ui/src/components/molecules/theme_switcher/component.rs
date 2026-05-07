use std::str::FromStr;

use crate::components::atoms::dropdown_menu::{
    DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger,
};
use dioxus::document::eval;
use dioxus::prelude::*;
use dioxus_free_icons::icons::hi_solid_icons::{HiDesktopComputer, HiMoon, HiSun};
use dioxus_free_icons::Icon;

#[derive(Clone, PartialEq, Copy, Debug)]
pub enum Theme {
    Light,
    Dark,
    System,
}

impl Theme {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
            Self::System => "system",
        }
    }
}

impl FromStr for Theme {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "light" => Ok(Self::Light),
            "dark" => Ok(Self::Dark),
            _ => Ok(Self::System),
        }
    }
}

fn apply_to_dom(theme: Theme) {
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
pub fn ThemeSwitcher() -> Element {
    let mut current_theme = use_signal(|| Theme::System);

    // Read persisted theme from localStorage on mount and apply it
    use_effect(move || {
        spawn(async move {
            let mut js = eval("dioxus.send(localStorage.getItem('theme') ?? 'system')");
            if let Ok(val) = js.recv::<String>().await {
                let theme = Theme::from_str(&val).unwrap_or(Theme::System);
                current_theme.set(theme);
                apply_to_dom(theme);
            }
        });
    });

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }

        div { class: "theme-switcher",
            DropdownMenu {
                DropdownMenuTrigger {
                    Icon { icon: HiSun, width: 20, height: 20 }
                }
                DropdownMenuContent {
                    DropdownMenuItem {
                        value: Theme::Light,
                        index: 0_usize,
                        on_select: move |t: Theme| {
                            current_theme.set(t);
                            apply_to_dom(t);
                        },
                        div { class: "theme-menu-item-inner",
                            Icon { icon: HiSun, width: 16, height: 16 }
                            "Light"
                            if *current_theme.read() == Theme::Light {
                                span { class: "theme-menu-item-check", "✓" }
                            }
                        }
                    }
                    DropdownMenuItem {
                        value: Theme::Dark,
                        index: 1_usize,
                        on_select: move |t: Theme| {
                            current_theme.set(t);
                            apply_to_dom(t);
                        },
                        div { class: "theme-menu-item-inner",
                            Icon { icon: HiMoon, width: 16, height: 16 }
                            "Dark"
                            if *current_theme.read() == Theme::Dark {
                                span { class: "theme-menu-item-check", "✓" }
                            }
                        }
                    }
                    DropdownMenuItem {
                        value: Theme::System,
                        index: 2_usize,
                        on_select: move |t: Theme| {
                            current_theme.set(t);
                            apply_to_dom(t);
                        },
                        div { class: "theme-menu-item-inner",
                            Icon { icon: HiDesktopComputer, width: 16, height: 16 }
                            "System"
                            if *current_theme.read() == Theme::System {
                                span { class: "theme-menu-item-check", "✓" }
                            }
                        }
                    }
                }
            }
        }
    }
}
