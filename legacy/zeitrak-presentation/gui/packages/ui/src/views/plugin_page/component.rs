use dioxus::prelude::*;
use dioxus_extism_frontend::PluginPageOutlet;

use crate::layouts::DefaultLayout;

/// Renders a plugin-contributed full page (§12.4 / step 30).
///
/// The route `/plugin/:plugin_id/*rest` is caught here. `PluginPageOutlet`
/// calls the `get_plugin_page` server function with the relative path and
/// renders the plugin's `PluginView` tree.
#[component]
pub fn PluginPage(plugin_id: String, rest: Vec<String>) -> Element {
    let relative_path = format!("/{}", rest.join("/"));
    let bypass_layout = use_signal(|| false);

    if *bypass_layout.read() {
        rsx! {
            PluginPageOutlet {
                relative_path,
                bypass_layout_signal: bypass_layout,
            }
        }
    } else {
        rsx! {
            DefaultLayout {
                PluginPageOutlet {
                    relative_path,
                    bypass_layout_signal: bypass_layout,
                    not_found: rsx! {
                        div { class: "plugin-not-found",
                            h2 { "Plugin page not found" }
                            p { "The plugin "{plugin_id}" has not contributed a page at this path." }
                        }
                    },
                }
            }
        }
    }
}
