use crate::components::atoms::{Button, Form, FormField, Input, Label};
use crate::views::timeline::TimelinePreview;
use dioxus::prelude::*;
use dioxus_free_icons::icons::hi_solid_icons::{HiLogin, HiRefresh};
use dioxus_free_icons::Icon;
use dioxus_i18n::tid;

type AuthState = Signal<Option<Option<api::auth::UserInfo>>>;

#[component]
pub fn Login() -> Element {
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);
    let mut submitting = use_signal(|| false);

    let navigator = use_navigator();
    let mut auth: AuthState = use_context();
    let invite_only = use_resource(api::registration::is_invite_only);

    let on_submit = move |_| {
        let email = email.read().clone();
        let password = password.read().clone();

        async move {
            submitting.set(true);
            error.set(None);

            match api::login::login(email, password).await {
                Ok(()) => {
                    if let Ok(user) = api::auth::get_current_user().await {
                        auth.set(Some(user));
                    }
                    navigator.push("/select-workspace");
                }
                Err(e) => {
                    error.set(Some(e.to_string()));
                    submitting.set(false);
                }
            }
        }
    };

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }

        div { class: "auth-screen",
            div { class: "auth-form-wrap",
                div { class: "auth-form",
                    // Brand
                    div { class: "auth-brand",
                        div { class: "auth-brand-mark", "Z" }
                        div { class: "auth-brand-text",
                            span { class: "auth-brand-name", "Zeitrak" }
                            span { class: "auth-brand-sub", {tid!("sidebar-brand-sub")} }
                        }
                    }

                    // Heading
                    h1 { class: "auth-heading", {tid!("login-heading")} }
                    p { class: "auth-subheading", {tid!("login-subheading")} }

                    // Form
                    Form {
                        FormField {
                            Label { html_for: "email", {tid!("common-email")} }
                            Input {
                                id: "email",
                                r#type: "email",
                                placeholder: tid!("login-email-placeholder"),
                                oninput: move |e: FormEvent| email.set(e.value()),
                            }
                        }
                        FormField {
                            Label { html_for: "password", {tid!("common-password")} }
                            Input {
                                id: "password",
                                r#type: "password",
                                placeholder: "••••••••",
                                oninput: move |e: FormEvent| password.set(e.value()),
                            }
                        }
                        if let Some(msg) = error.read().as_deref() {
                            p { class: "auth-error", "{msg}" }
                        }
                        Button {
                            class: "auth-submit-btn",
                            r#type: "submit",
                            disabled: *submitting.read(),
                            onclick: on_submit,
                            if *submitting.read() {
                                Icon { icon: HiRefresh, width: 15, height: 15 }
                                {tid!("login-signing-in")}
                            } else {
                                Icon { icon: HiLogin, width: 15, height: 15 }
                                {tid!("common-sign-in")}
                            }
                        }
                    }

                    // Footer
                    if matches!(invite_only.value().cloned(), Some(Ok(false))) {
                        div { class: "auth-footer",
                            span { class: "auth-footer-text", {tid!("login-no-account")} }
                            a {
                                href: "/register",
                                class: "auth-footer-link",
                                {tid!("login-create-account")}
                            }
                        }
                    }
                }
            }

            // Right art panel — timeline preview
            div { class: "auth-art",
                TimelinePreview {}
            }
        }
    }
}
