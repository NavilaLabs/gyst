use crate::components::atoms::{
    Button, Card, CardContent, CardFooter, Form, FormField, Input, Label,
};
use crate::layouts::DefaultLayout;
use dioxus::prelude::*;
use dioxus_free_icons::icons::hi_solid_icons::{HiLogin, HiRefresh};
use dioxus_free_icons::Icon;

type AuthState = Signal<Option<Option<api::auth::UserInfo>>>;

#[component]
pub fn Login() -> Element {
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);
    let mut submitting = use_signal(|| false);

    let navigator = use_navigator();
    let mut auth: AuthState = use_context();

    let on_submit = move |_| {
        let email = email.read().clone();
        let password = password.read().clone();

        async move {
            submitting.set(true);
            error.set(None);

            match api::login::login(email, password).await {
                Ok(()) => {
                    // Fetch fresh user info and push it into the global auth signal
                    // so the navbar updates immediately without waiting for a re-mount.
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
        DefaultLayout {
            Card {
                class: "w-full",
                data_size: "md",
                CardContent {
                    Form {
                        FormField {
                            Label { html_for: "email", class: "w-full", "Email" }
                            Input {
                                id: "email",
                                r#type: "email",
                                class: "w-full",
                                oninput: move |e: FormEvent| email.set(e.value()),
                            }
                        }
                        FormField {
                            Label { html_for: "password", class: "w-full", "Password" }
                            Input {
                                id: "password",
                                r#type: "password",
                                class: "w-full",
                                oninput: move |e: FormEvent| password.set(e.value()),
                            }
                        }
                        if let Some(msg) = error.read().as_deref() {
                            p { class: "text-red-500 text-sm mt-2", "{msg}" }
                        }
                    }
                }
                CardFooter {
                    Button {
                        class: "ms-auto",
                        r#type: "submit",
                        disabled: *submitting.read(),
                        onclick: on_submit,
                        if *submitting.read() {
                            Icon { icon: HiRefresh, width: 16, height: 16 }
                            "Signing in…"
                        } else {
                            Icon { icon: HiLogin, width: 16, height: 16 }
                            "Sign in"
                        }
                    }
                }
            }
        }
    }
}
