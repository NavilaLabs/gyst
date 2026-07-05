use api::invitation::InvitationDto;
use api::workspace::WorkspaceDto;
use dioxus::prelude::*;
// use dioxus_extism_frontend::PluginSlot;
use dioxus_free_icons::icons::hi_solid_icons::{HiArrowRight, HiCheck, HiX};
use dioxus_free_icons::Icon;
use dioxus_i18n::tid;

// use crate::PluginHostCtx;

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
    let mut invitations = use_signal(Vec::<InvitationDto>::new);
    let mut error = use_signal(|| None::<String>);
    let mut auth: AuthState = use_context();
    let navigator = use_navigator();

    use_resource(move || async move {
        match api::workspace::list_workspaces().await {
            Ok(list) => workspaces.set(list),
            Err(e) => error.set(Some(e.to_string())),
        }
    });

    use_resource(move || async move {
        if let Ok(list) = api::invitation::list_my_invitations().await {
            invitations.set(list);
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
                        span { class: "workspace-select-eyebrow-text", {tid!("workspace-select-brand")} }
                    }
                    h1 { class: "workspace-select-heading", {tid!("workspace-select-heading")} }
                    p { class: "workspace-select-subheading",
                        {tid!("workspace-select-subheading")}
                    }
                }

                // Workspace list
                div { class: "workspace-list",
                    if workspaces.read().is_empty() && error.read().is_none() {
                        div { class: "workspace-select-empty",
                            {tid!("workspace-select-loading")}
                        }
                    }

                    for ws in workspaces.read().iter() {
                        {
                            let ws = ws.clone();
                            let id = ws.id.clone();
                            let initial = workspace_initial(&ws);
                            let name = ws.name.clone().unwrap_or_else(|| tid!("workspace-select-unnamed"));

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
                                        span { class: "workspace-item-meta", {tid!("workspace-select-click-to-enter")} }
                                    }
                                    div { class: "workspace-item-arrow",
                                        Icon { icon: HiArrowRight, width: 18, height: 18 }
                                    }
                                }
                            }
                        }
                    }
                }

                // Pending invitations
                if !invitations.read().is_empty() {
                    div { class: "workspace-invitations-section",
                        p { class: "workspace-invitations-label", {tid!("workspace-select-pending-invitations")} }
                        div { class: "workspace-list",
                            for inv in invitations.read().iter() {
                                {
                                    let inv = inv.clone();
                                    let workspace_name = inv.workspace_name.clone()
                                        .unwrap_or_else(|| tid!("workspace-select-unnamed"));
                                    let accept_token = inv.token.clone();
                                    let decline_token = inv.token.clone();

                                    rsx! {
                                        div {
                                            key: "{inv.id}",
                                            class: "workspace-item workspace-item--invitation",
                                            div { class: "workspace-item-avatar workspace-item-avatar--invite", "?" }
                                            div { class: "workspace-item-info",
                                                span { class: "workspace-item-name", "{workspace_name}" }
                                                span { class: "workspace-item-meta", {tid!("workspace-select-invited-meta")} }
                                            }
                                            div { class: "workspace-item-invite-actions",
                                                button {
                                                    class: "invite-action-btn invite-action-btn--accept",
                                                    title: tid!("workspace-select-accept-title"),
                                                    onclick: move |e| {
                                                        e.stop_propagation();
                                                        let token = accept_token.clone();
                                                        async move {
                                                            match api::invitation::accept_invitation(token.clone()).await {
                                                                Ok(_) => {
                                                                    if let Ok(user) = api::auth::get_current_user().await {
                                                                        auth.set(Some(user));
                                                                    }
                                                                    if let Ok(list) = api::workspace::list_workspaces().await {
                                                                        workspaces.set(list);
                                                                    }
                                                                    invitations.write().retain(|i| i.token != token);
                                                                }
                                                                Err(e) => error.set(Some(e.to_string())),
                                                            }
                                                        }
                                                    },
                                                    Icon { icon: HiCheck, width: 14, height: 14 }
                                                    {tid!("workspace-select-accept")}
                                                }
                                                button {
                                                    class: "invite-action-btn invite-action-btn--decline",
                                                    title: tid!("workspace-select-decline-title"),
                                                    onclick: move |e| {
                                                        e.stop_propagation();
                                                        let token = decline_token.clone();
                                                        async move {
                                                            match api::invitation::decline_invitation(token.clone()).await {
                                                                Ok(()) => invitations.write().retain(|i| i.token != token),
                                                                Err(e) => error.set(Some(e.to_string())),
                                                            }
                                                        }
                                                    },
                                                    Icon { icon: HiX, width: 14, height: 14 }
                                                    {tid!("workspace-select-decline")}
                                                }
                                            }
                                        }
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
