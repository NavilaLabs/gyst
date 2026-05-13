use crate::components::atoms::{Card, CardContent, CardFooter};
use crate::layouts::DefaultLayout;
use dioxus::prelude::*;
use dioxus_i18n::tid;

#[component]
pub fn VerifyEmailConfirm(token: String) -> Element {
    let result = use_server_future(move || api::registration::verify_email(token.clone()))?;

    match result() {
        None => rsx! {},
        Some(Ok(())) => rsx! {
            DefaultLayout {
                Card {
                    class: "w-full",
                    data_size: "md",
                    CardContent {
                        p { class: "text-center font-semibold mb-2", {tid!("verify-email-verified-heading")} }
                        p { class: "text-center text-sm",
                            {tid!("verify-email-verified-body")}
                        }
                    }
                    CardFooter {
                        a {
                            href: "/login",
                            class: "text-sm underline mx-auto",
                            {tid!("common-sign-in")}
                        }
                    }
                }
            }
        },
        Some(Err(_)) => rsx! {
            DefaultLayout {
                Card {
                    class: "w-full",
                    data_size: "md",
                    CardContent {
                        p { class: "text-center font-semibold mb-2", {tid!("verify-email-failed-heading")} }
                        p { class: "text-center text-sm",
                            {tid!("verify-email-failed-body")}
                        }
                    }
                    CardFooter {
                        a {
                            href: "/login",
                            class: "text-sm underline mx-auto",
                            {tid!("common-go-to-sign-in")}
                        }
                    }
                }
            }
        },
    }
}
