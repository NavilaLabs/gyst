use crate::components::atoms::{
    Button, Card, CardContent, CardFooter, Form, FormField, Input, Label,
};
use crate::layouts::DefaultLayout;
use dioxus::prelude::*;
use dioxus_free_icons::icons::hi_solid_icons::{HiCheck, HiRefresh};
use dioxus_free_icons::Icon;

type AuthState = Signal<Option<Option<api::auth::UserInfo>>>;

#[component]
pub fn InvitationAccept(token: String) -> Element {
    let token_clone = token.clone();
    let invitation =
        use_resource(move || api::invitation::get_invitation_by_token(token_clone.clone()));

    let auth: AuthState = use_context();

    match invitation.value().cloned() {
        // Still loading
        None => rsx! {},

        // Error fetching or invalid token
        Some(Err(_)) | Some(Ok(None)) => rsx! {
            DefaultLayout {
                Card {
                    class: "w-full",
                    data_size: "md",
                    CardContent {
                        p { class: "text-center text-sm",
                            "This invitation is invalid or has expired."
                        }
                    }
                    CardFooter {
                        a {
                            href: "/login",
                            class: "text-sm underline mx-auto",
                            "Go to sign in"
                        }
                    }
                }
            }
        },

        Some(Ok(Some(inv))) => {
            let status = inv.status.clone();
            if status != "pending" {
                return rsx! {
                    DefaultLayout {
                        Card {
                            class: "w-full",
                            data_size: "md",
                            CardContent {
                                p { class: "text-center text-sm",
                                    "This invitation has already been used or revoked."
                                }
                            }
                            CardFooter {
                                a {
                                    href: "/login",
                                    class: "text-sm underline mx-auto",
                                    "Go to sign in"
                                }
                            }
                        }
                    }
                };
            }

            let workspace_name = inv
                .workspace_name
                .clone()
                .unwrap_or_else(|| "a workspace".to_string());
            let invited_email = inv.email.clone();

            match auth.cloned() {
                // Authenticated — show accept button
                Some(Some(_)) => rsx! {
                    AcceptPanel {
                        workspace_name,
                        token: token.clone(),
                    }
                },
                // Unauthenticated or loading — show registration form
                _ => rsx! {
                    RegisterPanel {
                        workspace_name,
                        email: invited_email,
                        token: token.clone(),
                    }
                },
            }
        }
    }
}

// ── Sub-components ────────────────────────────────────────────────────────────

#[component]
fn AcceptPanel(workspace_name: String, token: String) -> Element {
    let mut error = use_signal(|| None::<String>);
    let mut submitting = use_signal(|| false);
    let mut auth: AuthState = use_context();
    let navigator = use_navigator();

    let on_accept = move |_| {
        let token = token.clone();
        async move {
            submitting.set(true);
            error.set(None);
            match api::invitation::accept_invitation(token).await {
                Ok(_) => {
                    if let Ok(user) = api::auth::get_current_user().await {
                        auth.set(Some(user));
                    }
                    navigator.push("/dashboard");
                }
                Err(e) => {
                    error.set(Some(e.to_string()));
                    submitting.set(false);
                }
            }
        }
    };

    rsx! {
        DefaultLayout {
            Card {
                class: "w-full",
                data_size: "md",
                CardContent {
                    p { class: "text-center text-sm",
                        "You have been invited to join "
                        strong { "{workspace_name}" }
                        "."
                    }
                    if let Some(msg) = error.read().as_deref() {
                        p { class: "text-red-500 text-sm mt-2", "{msg}" }
                    }
                }
                CardFooter {
                    Button {
                        class: "mx-auto",
                        r#type: "button",
                        disabled: *submitting.read(),
                        onclick: on_accept,
                        if *submitting.read() {
                            Icon { icon: HiRefresh, width: 16, height: 16 }
                            "Accepting…"
                        } else {
                            Icon { icon: HiCheck, width: 16, height: 16 }
                            "Accept invitation"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn RegisterPanel(workspace_name: String, email: String, token: String) -> Element {
    let mut name = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut repeat_password = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);
    let mut submitting = use_signal(|| false);
    let mut auth: AuthState = use_context();
    let navigator = use_navigator();

    let on_submit = move |_| {
        let name = name.read().clone();
        let password = password.read().clone();
        let repeat_password = repeat_password.read().clone();
        let token = token.clone();

        async move {
            submitting.set(true);
            error.set(None);

            if password != repeat_password {
                error.set(Some("Passwords do not match.".to_string()));
                submitting.set(false);
                return;
            }

            match api::invitation::register_and_accept(name, password, token).await {
                Ok(()) => {
                    if let Ok(user) = api::auth::get_current_user().await {
                        auth.set(Some(user));
                    }
                    navigator.push("/dashboard");
                }
                Err(e) => {
                    error.set(Some(e.to_string()));
                    submitting.set(false);
                }
            }
        }
    };

    rsx! {
        DefaultLayout {
            Card {
                class: "w-full",
                data_size: "md",
                CardContent {
                    p { class: "text-sm mb-4",
                        "You have been invited to join "
                        strong { "{workspace_name}" }
                        ". Create an account to continue."
                    }
                    Form {
                        FormField {
                            Label { html_for: "inv-name", class: "w-full", "Name" }
                            Input {
                                id: "inv-name",
                                r#type: "text",
                                class: "w-full",
                                oninput: move |e: FormEvent| name.set(e.value()),
                            }
                        }
                        FormField {
                            Label { html_for: "inv-email", class: "w-full", "Email" }
                            Input {
                                id: "inv-email",
                                r#type: "email",
                                class: "w-full",
                                value: "{email}",
                                readonly: true,
                                disabled: true,
                            }
                        }
                        FormField {
                            Label { html_for: "inv-password", class: "w-full", "Password" }
                            Input {
                                id: "inv-password",
                                r#type: "password",
                                class: "w-full",
                                oninput: move |e: FormEvent| password.set(e.value()),
                            }
                        }
                        FormField {
                            Label { html_for: "inv-repeat-password", class: "w-full", "Repeat password" }
                            Input {
                                id: "inv-repeat-password",
                                r#type: "password",
                                class: "w-full",
                                oninput: move |e: FormEvent| repeat_password.set(e.value()),
                            }
                        }
                        if let Some(msg) = error.read().as_deref() {
                            p { class: "text-red-500 text-sm mt-2", "{msg}" }
                        }
                    }
                }
                CardFooter {
                    a {
                        href: "/login",
                        class: "text-sm underline self-center",
                        "Already have an account?"
                    }
                    Button {
                        class: "ms-auto",
                        r#type: "submit",
                        disabled: *submitting.read(),
                        onclick: on_submit,
                        if *submitting.read() {
                            Icon { icon: HiRefresh, width: 16, height: 16 }
                            "Please wait…"
                        } else {
                            Icon { icon: HiCheck, width: 16, height: 16 }
                            "Register and join"
                        }
                    }
                }
            }
        }
    }
}
