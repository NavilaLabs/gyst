use crate::components::atoms::{Card, CardContent, CardFooter};
use crate::layouts::DefaultLayout;
use dioxus::prelude::*;

#[component]
pub fn VerifyEmailConfirm(token: String) -> Element {
    let result = use_resource(move || api::registration::verify_email(token.clone()));

    match result.value().cloned() {
        None => rsx! {},
        Some(Ok(())) => rsx! {
            DefaultLayout {
                Card {
                    class: "w-full",
                    data_size: "md",
                    CardContent {
                        p { class: "text-center font-semibold mb-2", "Email verified" }
                        p { class: "text-center text-sm",
                            "Your email address has been verified. You can now sign in."
                        }
                    }
                    CardFooter {
                        a {
                            href: "/login",
                            class: "text-sm underline mx-auto",
                            "Sign in"
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
                        p { class: "text-center font-semibold mb-2", "Verification failed" }
                        p { class: "text-center text-sm",
                            "This verification link is invalid or has already been used."
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
    }
}
