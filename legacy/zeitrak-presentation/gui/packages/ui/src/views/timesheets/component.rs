use crate::components::molecules::MemberFilter;
use crate::layouts::DefaultLayout;
use crate::views::timesheets::entry_table::EntryTable;
use crate::views::timesheets::timer_card::TimerCard;
use crate::{ActivitiesCache, TagsCache};
use dioxus::prelude::*;

#[component]
pub fn Timesheets() -> Element {
    let activities_cache: ActivitiesCache = use_context();
    let tags_cache: TagsCache = use_context();

    let mut page: Signal<u32> = use_signal(|| 0);
    let mut member_id: Signal<Option<String>> = use_signal(|| None);
    let mut total: Signal<u64> = use_signal(|| 0);
    let mut can_filter_members = use_signal(|| false);
    let mut members: Signal<Vec<api::member::MemberDto>> = use_signal(Vec::new);

    let mut timesheets = use_signal(Vec::<api::timesheet::TimesheetDto>::new);
    let mut loading = use_signal(|| true);

    use_resource(move || async move {
        loading.set(true);
        let current_page = *page.read();
        let current_member = member_id.read().clone();
        if let Ok(result) = api::timesheet::list_timesheets(current_page, current_member).await {
            timesheets.set(result.items);
            total.set(result.total);
            // peek() avoids a reactive subscription that would restart this resource
            // when members.set(list) fires below.
            if result.can_filter_members && members.peek().is_empty() {
                if let Ok(list) = api::member::list_members().await {
                    members.set(list);
                }
            }
            can_filter_members.set(result.can_filter_members);
        }
        loading.set(false);
    });

    let on_timer_changed = move |_| async move {
        let current_page = *page.read();
        let current_member = member_id.read().clone();
        if let Ok(result) = api::timesheet::list_timesheets(current_page, current_member).await {
            timesheets.set(result.items);
            total.set(result.total);
        }
    };

    let on_member_change = move |new_member: Option<String>| {
        page.set(0);
        member_id.set(new_member);
    };

    rsx! {
        DefaultLayout {
            div { class: "space-y-6",
                TimerCard {
                    activities: activities_cache,
                    on_timer_changed,
                }
                if *can_filter_members.read() {
                    MemberFilter {
                        members,
                        selected: member_id,
                        on_change: on_member_change,
                    }
                }
                EntryTable {
                    timesheets,
                    activities: activities_cache,
                    all_tags: tags_cache,
                    page,
                    total,
                    loading,
                    on_page_change: move |p: u32| page.set(p),
                }
            }
        }
    }
}
