use crate::components::atoms::{Card, CardContent, CardFooter};
use crate::layouts::DefaultLayout;
use dioxus::prelude::*;
use dioxus_i18n::tid;

#[component]
pub fn VerifyEmailPending() -> Element {
    rsx! {
        DefaultLayout {
            Card {
                class: "w-full",
                data_size: "md",
                CardContent {
                    p { class: "text-center font-semibold mb-2", {tid!("verify-email-pending-heading")} }
                    p { class: "text-center text-sm",
                        {tid!("verify-email-pending-body")}
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
    }
}
