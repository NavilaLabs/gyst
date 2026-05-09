use crate::layouts::DefaultLayout;
use crate::views::timesheets::entry_table::EntryTable;
use crate::views::timesheets::timer_card::TimerCard;
use crate::{ActivitiesCache, TagsCache, TimesheetsCache};
use dioxus::prelude::*;

#[component]
pub fn Timesheets() -> Element {
    let timesheets_cache: TimesheetsCache = use_context();
    let activities_cache: ActivitiesCache = use_context();
    let tags_cache: TagsCache = use_context();
    let mut running: crate::RunningTimer = use_context();

    let mut timesheets = use_signal(|| timesheets_cache.read().clone());
    let mut loading = use_signal(|| timesheets_cache.read().is_empty());
    let page = use_signal(|| 0_usize);
    let mut activities = use_signal(|| activities_cache.read().clone());
    let mut all_tags = use_signal(|| tags_cache.read().clone());

    use_resource(move || async move {
        if let Ok(list) = api::timesheet::list_timesheets().await {
            timesheets.set(list);
        }
        if let Ok(r) = api::timesheet::running_timesheet().await {
            running.set(r);
        }
        if let Ok(list) = api::activity::list_activities().await {
            activities.set(list);
        }
        if let Ok(list) = api::timesheet_tag::list_tags().await {
            all_tags.set(list);
        }
        loading.set(false);
    });

    let on_timer_changed = move |_| async move {
        if let Ok(list) = api::timesheet::list_timesheets().await {
            timesheets.set(list);
        }
        // Do not re-fetch running_timesheet here: on_start and on_stop manage
        // the running signal optimistically. Re-fetching would race against
        // the projection daemon and could overwrite a just-started timer with
        // a stale None from the server.
    };

    rsx! {
        DefaultLayout {
            div { class: "space-y-6",
                TimerCard {
                    activities,
                    on_timer_changed,
                }
                EntryTable {
                    timesheets,
                    activities,
                    all_tags,
                    page,
                    loading,
                }
            }
        }
    }
}
