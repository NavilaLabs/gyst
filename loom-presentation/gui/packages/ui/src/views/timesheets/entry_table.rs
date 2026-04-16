use crate::components::atoms::{
    Button, ColumnDef, DataTable, Input, Select, SelectOption, TableCell, TableExpandRow, TableRow,
    ToastExt, Toasts,
};
use crate::formatting;
use api::activity::ActivityDto;
use api::timesheet::TimesheetDto;
use api::timesheet_tag::TimesheetsTagDto;
use dioxus::prelude::*;
use dioxus_free_icons::icons::hi_solid_icons::{HiPencil, HiSave, HiTag, HiX};
use dioxus_free_icons::Icon;

#[derive(Clone, PartialEq, Props)]
pub(super) struct EntryTableProps {
    pub timesheets: Signal<Vec<TimesheetDto>>,
    pub activities: Signal<Vec<ActivityDto>>,
    pub all_tags: Signal<Vec<TimesheetsTagDto>>,
    pub page: Signal<usize>,
    pub loading: Signal<bool>,
}

#[component]
pub(super) fn EntryTable(props: EntryTableProps) -> Element {
    let mut toasts: Toasts = use_context();
    let user_settings: crate::UserSettings = use_context();

    let mut timesheets = props.timesheets;
    let activities = props.activities;
    let all_tags = props.all_tags;
    let mut page = props.page;

    let mut editing_id = use_signal(|| Option::<String>::None);
    let mut edit_activity_id = use_signal(|| Option::<String>::None);
    let mut edit_description = use_signal(String::new);
    let mut edit_start_time = use_signal(String::new);
    let mut edit_end_time = use_signal(|| Option::<String>::None);

    let mut tagging_id = use_signal(|| Option::<String>::None);
    let mut ts_tags = use_signal(Vec::<TimesheetsTagDto>::new);

    let on_save_edit = move |_| async move {
        let id = match editing_id.peek().clone() {
            Some(id) => id,
            None => return,
        };

        let new_start_local = edit_start_time.peek().clone();
        if !new_start_local.is_empty() {
            let tz = user_settings.peek().timezone.clone();
            let new_start = formatting::from_input(&new_start_local, &tz);
            let new_end = edit_end_time
                .peek()
                .as_deref()
                .map(|s| formatting::from_input(s, &tz));
            if let Err(e) = api::timesheet::update_timesheet_time(
                id.clone(),
                new_start.clone(),
                new_end.clone(),
            )
            .await
            {
                toasts.push_error(e.to_string());
                return;
            }
            if let Some(item) = timesheets.write().iter_mut().find(|x| x.id == id) {
                item.start_time = new_start;
                item.end_time = new_end;
            }
        }

        let new_aid = edit_activity_id.peek().clone();

        if let Some(ref aid) = &new_aid {
            let needs_reassign = timesheets
                .read()
                .iter()
                .find(|x| x.id == id)
                .map(|ts| ts.activity_id.as_deref() != Some(aid.as_str()))
                .unwrap_or(false);
            if needs_reassign {
                if let Err(e) = api::timesheet::reassign_timesheet(id.clone(), aid.clone()).await {
                    toasts.push_error(e.to_string());
                    return;
                }
            }
        }

        let desc = {
            let s = edit_description.peek().clone();
            if s.is_empty() {
                None
            } else {
                Some(s)
            }
        };
        if let Err(e) = api::timesheet::update_timesheet(id.clone(), desc.clone()).await {
            toasts.push_error(e.to_string());
            return;
        }
        if let Some(item) = timesheets.write().iter_mut().find(|x| x.id == id) {
            item.description = desc;
            if let Some(aid) = new_aid {
                item.activity_id = Some(aid);
            }
        }
        editing_id.set(None);
        toasts.push_success("Timesheet updated");
    };

    let total = timesheets.read().len();
    let current_page = *page.read();
    let page_size = 20_usize;
    let page_items: Vec<TimesheetDto> = timesheets
        .read()
        .iter()
        .skip(current_page * page_size)
        .take(page_size)
        .cloned()
        .collect();

    let ts_columns = vec![
        ColumnDef::new("Activity"),
        ColumnDef::new("Start").width("160px"),
        ColumnDef::new("Duration").right().width("90px"),
        ColumnDef::new("").width("100px"),
    ];
    let col_count = ts_columns.len();

    rsx! {
        div { class: "island",
            div { class: "island-header",
                span { class: "island-title", "Recent" }
            }
            DataTable {
                columns: ts_columns,
                total,
                page: current_page,
                page_size,
                loading: *props.loading.read(),
                on_page_change: move |p| page.set(p),

                for ts in page_items {
                    {
                        let t = ts.clone();
                        let tsid = t.id.clone();
                        let tsid2 = t.id.clone();
                        let is_editing = editing_id.read().as_deref() == Some(t.id.as_str());
                        let is_tagging = tagging_id.read().as_deref() == Some(t.id.as_str());
                        let act_name = t.activity_id.as_ref()
                            .and_then(|aid| activities.read().iter().find(|a| &a.id == aid).map(|a| a.name.clone()))
                            .unwrap_or_else(|| "—".to_string());
                        let duration_str = t.duration.map(|dur| {
                            let h = dur / 3600;
                            let m = (dur % 3600) / 60;
                            if h > 0 { format!("{h}h {m}m") } else { format!("{m}m") }
                        });
                        let date_str = {
                            let s = user_settings.read();
                            formatting::format_datetime(&t.start_time, &s.timezone, &s.date_format)
                        };

                        rsx! {
                            TableRow { key: "{t.id}",
                                TableCell {
                                    div { class: "flex flex-col gap-0.5",
                                        span { class: "text-xs text-secondary", "{act_name}" }
                                        if let Some(ref desc) = t.description {
                                            span { class: "text-xs text-secondary italic", "{desc}" }
                                        }
                                    }
                                }
                                TableCell { mono: true, "{date_str}" }
                                TableCell { align: crate::components::atoms::ColAlign::Right, mono: true,
                                    if let Some(ref d) = duration_str {
                                        span { "{d}" }
                                    } else {
                                        span { class: "text-secondary", "—" }
                                    }
                                }
                                TableCell {
                                    div { class: "flex gap-1",
                                        if is_editing || is_tagging {
                                            Button {
                                                onclick: move |_| { editing_id.set(None); tagging_id.set(None); },
                                                Icon { icon: HiX, width: 14, height: 14 }
                                            }
                                        } else {
                                            Button {
                                                onclick: move |_| {
                                                    let edit_ts = timesheets.read()
                                                        .iter()
                                                        .find(|x| x.id == tsid)
                                                        .cloned();
                                                    if let Some(et) = edit_ts {
                                                        edit_activity_id.set(et.activity_id.clone());
                                                        edit_description.set(et.description.clone().unwrap_or_default());
                                                        let tz = user_settings.peek().timezone.clone();
                                                        edit_start_time.set(formatting::to_input(&et.start_time, &tz));
                                                        edit_end_time.set(et.end_time.as_deref().map(|s| formatting::to_input(s, &tz)));
                                                        editing_id.set(Some(et.id));
                                                        tagging_id.set(None);
                                                    }
                                                },
                                                Icon { icon: HiPencil, width: 14, height: 14 }
                                            }
                                            Button {
                                                onclick: move |_| {
                                                    let tsid2 = tsid2.clone();
                                                    async move {
                                                        if tagging_id.peek().as_deref() == Some(tsid2.as_str()) {
                                                            tagging_id.set(None);
                                                        } else {
                                                            match api::timesheet_tag::list_timesheet_tags(tsid2.clone()).await {
                                                                Ok(tags) => ts_tags.set(tags),
                                                                Err(e) => toasts.push_error(e.to_string()),
                                                            }
                                                            tagging_id.set(Some(tsid2.clone()));
                                                            editing_id.set(None);
                                                        }
                                                    }
                                                },
                                                Icon { icon: HiTag, width: 14, height: 14 }
                                            }
                                        }
                                    }
                                }
                            }

                            if is_editing {
                                TableExpandRow { col_count,
                                    div { class: "grid grid-cols-1 gap-4 md:grid-cols-2",
                                        div { class: "form-field",
                                            label { class: "form-label", r#for: "et-start", "Start time" }
                                            input {
                                                id: "et-start",
                                                r#type: "datetime-local",
                                                class: "input",
                                                value: edit_start_time.read().clone(),
                                                oninput: move |e: FormEvent| edit_start_time.set(e.value()),
                                            }
                                        }
                                        if edit_end_time.read().is_some() {
                                            div { class: "form-field",
                                                label { class: "form-label", r#for: "et-end", "End time" }
                                                input {
                                                    id: "et-end",
                                                    r#type: "datetime-local",
                                                    class: "input",
                                                    value: edit_end_time.read().clone().unwrap_or_default(),
                                                    oninput: move |e: FormEvent| {
                                                        let v = e.value();
                                                        edit_end_time.set(if v.is_empty() { None } else { Some(v) });
                                                    },
                                                }
                                            }
                                        }
                                        div { class: "form-field",
                                            label { class: "form-label", "Activity" }
                                            Select::<String> {
                                                options: activities.read().iter()
                                                    .map(|a| SelectOption::new(a.id.clone(), a.name.clone()))
                                                    .collect(),
                                                value: edit_activity_id.read().clone(),
                                                on_change: move |id: String| edit_activity_id.set(Some(id)),
                                                placeholder: "Select activity…".to_string(),
                                            }
                                        }
                                        div { class: "form-field md:col-span-2",
                                            label { class: "form-label", r#for: "et-desc", "Description" }
                                            Input {
                                                id: "et-desc",
                                                placeholder: "Optional notes…",
                                                value: edit_description.read().clone(),
                                                oninput: move |e: FormEvent| edit_description.set(e.value()),
                                            }
                                        }
                                    }
                                    div { class: "flex gap-2 mt-2",
                                        Button { onclick: on_save_edit,
                                            Icon { icon: HiSave, width: 14, height: 14 }
                                            "Save"
                                        }
                                        Button {
                                            onclick: move |_| editing_id.set(None),
                                            Icon { icon: HiX, width: 14, height: 14 }
                                            "Cancel"
                                        }
                                    }
                                }
                            }

                            if is_tagging {
                                {
                                    let tsid_tag = t.id.clone();
                                    rsx! {
                                        TableExpandRow { col_count,
                                            p { class: "text-xs font-medium text-secondary mb-2", "Tags" }
                                            div { class: "flex flex-wrap gap-2",
                                                for tag in all_tags.read().clone() {
                                                    {
                                                        let tag_id = tag.id.clone();
                                                        let tag_id2 = tag.id.clone();
                                                        let tsid_a = tsid_tag.clone();
                                                        let tsid_b = tsid_tag.clone();
                                                        let is_applied = ts_tags.read().iter().any(|t| t.id == tag.id);
                                                        rsx! {
                                                            button {
                                                                key: "{tag.id}",
                                                                class: if is_applied { "tag-pill tag-pill--active" } else { "tag-pill" },
                                                                onclick: move |_| {
                                                                    let tag_id = tag_id.clone();
                                                                    let tsid_a = tsid_a.clone();
                                                                    let tsid_b = tsid_b.clone();
                                                                    let _ = tag_id2.clone();
                                                                    async move {
                                                                        let result = if is_applied {
                                                                            api::timesheet_tag::untag_timesheet(tag_id.clone(), tsid_a.clone()).await
                                                                        } else {
                                                                            api::timesheet_tag::tag_timesheet(tag_id.clone(), tsid_a.clone()).await
                                                                        };
                                                                        if result.is_ok() {
                                                                            if let Ok(tags) = api::timesheet_tag::list_timesheet_tags(tsid_b.clone()).await {
                                                                                ts_tags.set(tags);
                                                                            }
                                                                        }
                                                                    }
                                                                },
                                                                "{tag.name}"
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
