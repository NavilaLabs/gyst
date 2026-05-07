use crate::components::atoms::{
    Button, ButtonVariant, Card, CardContent, CardFooter, Form, FormField, Input, Label,
    TabContent, TabList, TabTrigger, Tabs,
};
use crate::layouts::DefaultLayout;
use dioxus::prelude::*;
use dioxus_free_icons::icons::hi_solid_icons::{HiCheck, HiChevronLeft, HiChevronRight, HiRefresh};
use dioxus_free_icons::Icon;

#[component]
pub fn Setup() -> Element {
    let mut active_tab = use_signal(|| Some("admin".to_string()));
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut confirm_password = use_signal(String::new);
    let mut workspace_name = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);
    let mut submitting = use_signal(|| false);

    let navigator = use_navigator();

    let on_submit = move |_| {
        let email = email.read().clone();
        let password = password.read().clone();
        let workspace_name = workspace_name.read().clone();

        async move {
            submitting.set(true);
            error.set(None);

            match api::setup::setup("admin".to_string(), email, password, workspace_name).await {
                Ok(()) => {
                    navigator.push("/login");
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
                Tabs {
                    value: active_tab,
                    default_value: "admin",
                    on_value_change: move |v: String| active_tab.set(Some(v)),
                    CardContent {
                        TabList {
                            class: "w-full mb-4",
                            TabTrigger { value: "admin", index: 0usize, "Admin User" },
                            TabTrigger { value: "workspace", index: 1usize, "Workspace" }
                        }
                        TabContent { value: "admin", index: 0usize,
                            Form {
                                FormField {
                                    Label { html_for: "username", class: "w-full", "Username" }
                                    Input { id: "username", class: "w-full", disabled: true, placeholder: "admin" }
                                }
                                FormField {
                                    Label { html_for: "email", class: "w-full", "Email" }
                                    Input { id: "email", r#type: "email", class: "w-full", oninput: move |e: FormEvent| email.set(e.value()) }
                                }
                                FormField {
                                    Label { html_for: "password", class: "w-full", "Password" }
                                    Input { id: "password", r#type: "password", class: "w-full", oninput: move |e: FormEvent| password.set(e.value()) }
                                }
                                FormField {
                                    Label { html_for: "confirm_password", class: "w-full", "Confirm Password" }
                                    Input { id: "confirm_password", r#type: "password", class: "w-full", oninput: move |e: FormEvent| confirm_password.set(e.value()) }
                                }
                            }
                        }
                        TabContent { value: "workspace", index: 1usize,
                            Form {
                                FormField {
                                    Label { html_for: "workspace_name", class: "w-full", "Workspace Name" }
                                    Input { id: "workspace_name", class: "w-full", oninput: move |e: FormEvent| workspace_name.set(e.value()) }
                                }
                            }
                        }
                        if let Some(msg) = error.read().as_deref() {
                            p { class: "text-red-500 text-sm mt-2", "{msg}" }
                        }
                    }
                }
                CardFooter {
                    class: "flex",
                    if active_tab.read().as_deref() == Some("workspace") {
                        Button {
                            variant: ButtonVariant::Secondary,
                            onclick: move |_| active_tab.set(Some("admin".to_string())),
                            Icon { icon: HiChevronLeft, width: 16, height: 16 }
                            "Back"
                        }
                    }
                    if active_tab.read().as_deref() == Some("admin") {
                        Button {
                            class: "ms-auto",
                            onclick: move |_| active_tab.set(Some("workspace".to_string())),
                            "Next"
                            Icon { icon: HiChevronRight, width: 16, height: 16 }
                        }
                    } else {
                        Button {
                            class: "ms-auto",
                            r#type: "submit",
                            disabled: *submitting.read(),
                            onclick: on_submit,
                            if *submitting.read() {
                                Icon { icon: HiRefresh, width: 16, height: 16 }
                                "Submitting…"
                            } else {
                                Icon { icon: HiCheck, width: 16, height: 16 }
                                "Submit"
                            }
                        }
                    }
                }
            }
        }
    }
}
