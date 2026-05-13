use crate::components::atoms::{
    Button, Card, CardContent, CardFooter, Form, FormField, Input, Label,
};
use crate::layouts::DefaultLayout;
use dioxus::prelude::*;
use dioxus_free_icons::icons::hi_solid_icons::{HiLogin, HiRefresh};
use dioxus_free_icons::Icon;
use dioxus_i18n::tid;

type AuthState = Signal<Option<Option<api::auth::UserInfo>>>;

#[component]
pub fn Register() -> Element {
    let mut name = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut repeat_password = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);
    let mut submitting = use_signal(|| false);

    let navigator = use_navigator();
    let mut auth: AuthState = use_context();

    let invite_only = use_resource(|| api::registration::is_invite_only());

    let on_submit = move |_| {
        let name = name.read().clone();
        let email = email.read().clone();
        let password = password.read().clone();
        let repeat_password = repeat_password.read().clone();

        async move {
            submitting.set(true);
            error.set(None);

            if password != repeat_password {
                error.set(Some("Passwords do not match.".to_string()));
                submitting.set(false);
                return;
            }

            match api::registration::register(name, email, password).await {
                Ok(()) => {
                    if let Ok(user) = api::auth::get_current_user().await {
                        auth.set(Some(user));
                    }
                    navigator.push("/verify-email/pending");
                }
                Err(e) => {
                    error.set(Some(e.to_string()));
                    submitting.set(false);
                }
            }
        }
    };

    match invite_only.value().cloned() {
        None => rsx! {},
        Some(Ok(true)) => rsx! {
            DefaultLayout {
                Card {
                    class: "w-full",
                    data_size: "md",
                    CardContent {
                        p { class: "text-center text-sm",
                            {tid!("register-invite-only")}
                        }
                    }
                    CardFooter {
                        a {
                            href: "/login",
                            class: "text-sm underline mx-auto",
                            {tid!("register-sign-in-instead")}
                        }
                    }
                }
            }
        },
        Some(Ok(false)) | Some(Err(_)) => rsx! {
            DefaultLayout {
                Card {
                    class: "w-full",
                    data_size: "md",
                    CardContent {
                        Form {
                            FormField {
                                Label { html_for: "name", class: "w-full", {tid!("common-name")} }
                                Input {
                                    id: "name",
                                    r#type: "text",
                                    class: "w-full",
                                    oninput: move |e: FormEvent| name.set(e.value()),
                                }
                            }
                            FormField {
                                Label { html_for: "email", class: "w-full", {tid!("common-email")} }
                                Input {
                                    id: "email",
                                    r#type: "email",
                                    class: "w-full",
                                    oninput: move |e: FormEvent| email.set(e.value()),
                                }
                            }
                            FormField {
                                Label { html_for: "password", class: "w-full", {tid!("common-password")} }
                                Input {
                                    id: "password",
                                    r#type: "password",
                                    class: "w-full",
                                    oninput: move |e: FormEvent| password.set(e.value()),
                                }
                            }
                            FormField {
                                Label { html_for: "repeat-password", class: "w-full", {tid!("common-repeat-password")} }
                                Input {
                                    id: "repeat-password",
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
                            {tid!("common-already-have-account")}
                        }
                        Button {
                            class: "ms-auto",
                            r#type: "submit",
                            disabled: *submitting.read(),
                            onclick: on_submit,
                            if *submitting.read() {
                                Icon { icon: HiRefresh, width: 16, height: 16 }
                                {tid!("common-please-wait")}
                            } else {
                                Icon { icon: HiLogin, width: 16, height: 16 }
                                {tid!("register-register")}
                            }
                        }
                    }
                }
            }
        },
    }
}
