use api::workspace::WorkspaceDto;
use dioxus::prelude::*;
use dioxus_free_icons::icons::hi_solid_icons::HiArrowRight;
use dioxus_free_icons::Icon;

type AuthState = Signal<Option<Option<api::auth::UserInfo>>>;

fn workspace_initial(ws: &WorkspaceDto) -> char {
    ws.name
        .as_deref()
        .and_then(|n| n.chars().next())
        .map(|c| c.to_ascii_uppercase())
        .unwrap_or('W')
}

#[component]
pub fn SelectWorkspace() -> Element {
    let mut workspaces = use_signal(Vec::<WorkspaceDto>::new);
    let mut error = use_signal(|| None::<String>);
    let mut auth: AuthState = use_context();
    let navigator = use_navigator();

    use_resource(move || async move {
        match api::workspace::list_workspaces().await {
            Ok(list) => workspaces.set(list),
            Err(e) => error.set(Some(e.to_string())),
        }
    });

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }

        div { class: "workspace-select-page",
            div { class: "workspace-select-container",

                // Headline
                div {
                    div { class: "workspace-select-eyebrow",
                        span { class: "workspace-select-eyebrow-dot" }
                        span { class: "workspace-select-eyebrow-text", "Zeitrak" }
                    }
                    h1 { class: "workspace-select-heading", "Choose your workspace" }
                    p { class: "workspace-select-subheading",
                        "Select the workspace you'd like to enter."
                    }
                }

                // Workspace list
                div { class: "workspace-list",
                    if workspaces.read().is_empty() && error.read().is_none() {
                        div { class: "workspace-select-empty",
                            "Loading workspaces…"
                        }
                    }

                    for ws in workspaces.read().iter() {
                        {
                            let ws = ws.clone();
                            let id = ws.id.clone();
                            let initial = workspace_initial(&ws);
                            let name = ws.name.clone().unwrap_or_else(|| "Unnamed workspace".to_string());

                            rsx! {
                                div {
                                    key: "{ws.id}",
                                    class: "workspace-item",
                                    onclick: move |_| {
                                        let id = id.clone();
                                        async move {
                                            match api::workspace::select_workspace(id).await {
                                                Ok(()) => {
                                                    if let Ok(user) = api::auth::get_current_user().await {
                                                        auth.set(Some(user));
                                                    }
                                                    navigator.push("/dashboard");
                                                }
                                                Err(e) => error.set(Some(e.to_string())),
                                            }
                                        }
                                    },
                                    div { class: "workspace-item-avatar", "{initial}" }
                                    div { class: "workspace-item-info",
                                        span { class: "workspace-item-name", "{name}" }
                                        span { class: "workspace-item-meta", "Click to enter" }
                                    }
                                    div { class: "workspace-item-arrow",
                                        Icon { icon: HiArrowRight, width: 18, height: 18 }
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some(msg) = error.read().as_ref() {
                    div { class: "workspace-select-error", "{msg}" }
                }
            }
        }
    }
}
