use crate::components::atoms::{ColumnDef, DataTable, ToastExt, Toasts};
use crate::layouts::DefaultLayout;
use crate::views::activities::activity_row::ActivityRow;
use crate::views::activities::create_form::ActivityCreateForm;
use crate::ActivitiesCache;
use api::activity::ActivityDto;
use dioxus::prelude::*;
use dioxus_i18n::tid;

const PAGE_SIZE: usize = 15;

#[component]
pub fn Activities() -> Element {
    let activities_cache: ActivitiesCache = use_context();
    let mut activities = use_signal(|| activities_cache.read().clone());
    let mut loading = use_signal(|| activities_cache.read().is_empty());
    let mut toasts: Toasts = use_context();
    let mut page = use_signal(|| 0_usize);
    let editing_id = use_signal(|| Option::<String>::None);

    use_resource(move || async move {
        match api::activity::list_activities().await {
            Ok(list) => activities.set(list),
            Err(e) => toasts.push_error(e.to_string()),
        }
        loading.set(false);
    });

    let total = activities.read().len();
    let current_page = *page.read();
    let page_items: Vec<ActivityDto> = activities
        .read()
        .iter()
        .skip(current_page * PAGE_SIZE)
        .take(PAGE_SIZE)
        .cloned()
        .collect();

    let columns = vec![ColumnDef::new(tid!("common-name")), ColumnDef::new("").width("80px")];
    let col_count = columns.len();

    rsx! {
        DefaultLayout {
            div { class: "space-y-6",
                ActivityCreateForm {
                    on_created: move |dto: ActivityDto| activities.write().push(dto),
                }

                div { class: "island",
                    div { class: "island-header",
                        span { class: "island-title", {tid!("activities-title")} }
                    }
                    DataTable {
                        columns,
                        total,
                        page: current_page,
                        page_size: PAGE_SIZE,
                        loading: *loading.read(),
                        on_page_change: move |p| page.set(p),

                        for activity in page_items {
                            ActivityRow {
                                key: "{activity.id}",
                                activity,
                                activities,
                                editing_id,
                                col_count,
                            }
                        }
                    }
                }
            }
        }
    }
}
