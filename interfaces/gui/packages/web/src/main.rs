use dioxus::prelude::*;

use ui::{
    views::{Database, Login},
    FAVICON,
};

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(DefaultLayout)]
    #[route("/login")]
    Login {},
    #[layout(DefaultLayout)]
    #[route("/developer/database")]
    Database {},
}

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    // Build cool things ✌️

    rsx! {
        // Global app resources
        document::Title { "Loom" }
        document::Meta { name: "description", content: "Loom" }
        document::Meta { name: "viewport", content: "width=device-width, initial-scale=1" }
        document::Link { rel: "icon", href: FAVICON }

        Router::<Route> {}
    }
}

// A web-specific Router around the shared `Navbar` component
// which allows us to use the web-specific `Route` enum.
#[component]
fn DefaultLayout() -> Element {
    rsx! {
        Outlet::<Route> {}
    }
}
