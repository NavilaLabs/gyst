use crate::components::atoms::{Card, CardContent, CardFooter};
use crate::layouts::DefaultLayout;
use dioxus::prelude::*;

#[component]
pub fn VerifyEmailPending() -> Element {
    rsx! {
        DefaultLayout {
            Card {
                class: "w-full",
                data_size: "md",
                CardContent {
                    p { class: "text-center font-semibold mb-2", "Check your inbox" }
                    p { class: "text-center text-sm",
                        "We sent a verification link to your email address. \
                         Please click the link to activate your account."
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
    }
}
