use crate::components::atoms::card::{Card, CardContent, CardFooter, CardHeader, CardTitle};
use crate::components::atoms::{Button, Input, Select, SelectOption, ToastExt, Toasts};
use crate::formatting;
use api::activity::ActivityDto;
use dioxus::prelude::*;
use dioxus_free_icons::icons::hi_solid_icons::{HiClock, HiPlay, HiPlus, HiRefresh, HiStop};
use dioxus_free_icons::Icon;

#[derive(Clone, PartialEq, Props)]
pub(super) struct TimerCardProps {
    pub activities: Signal<Vec<ActivityDto>>,
    pub on_timer_changed: EventHandler<()>,
}

#[component]
pub(super) fn TimerCard(props: TimerCardProps) -> Element {
    let mut running: crate::RunningTimer = use_context();
    let user_settings: crate::UserSettings = use_context();
    let mut toasts: Toasts = use_context();

    let activities = props.activities;

    let mut manual_mode = use_signal(|| false);

    // Start timer form
    let mut project_id = use_signal(|| Option::<String>::None);
    let mut activity_id = use_signal(|| Option::<String>::None);
    let mut description = use_signal(String::new);
    let mut billable = use_signal(|| true);

    // Manual entry form
    let mut manual_activity_id = use_signal(|| Option::<String>::None);
    let mut manual_start = use_signal(String::new);
    let mut manual_end = use_signal(String::new);
    let mut manual_description = use_signal(String::new);
    let mut manual_billable = use_signal(|| true);
    let mut manual_submitting = use_signal(|| false);

    // Edit state for running timer
    let mut run_project_id = use_signal(|| Option::<String>::None);
    let mut run_activity_id = use_signal(|| Option::<String>::None);
    let mut run_description = use_signal(String::new);
    let mut run_billable = use_signal(|| true);

    // Sync run_* fields when running changes
    use_effect(move || {
        if let Some(ref ts) = *running.read() {
            run_activity_id.set(ts.activity_id.clone());
            run_description.set(ts.description.clone().unwrap_or_default());
        }
    });

    let on_start = move |_| async move {
        let pid = project_id.peek().clone();
        let aid = activity_id.peek().clone();
        let desc = description.peek().clone();
        let bill = *billable.peek();
        let (Some(pid), Some(aid)) = (pid, aid) else {
            return;
        };
        let desc_opt = if desc.is_empty() { None } else { Some(desc) };
        match api::timesheet::start_timesheet(Some(aid.clone()), desc_opt.clone()).await {
            Ok(dto) => {
                run_project_id.set(Some(pid));
                run_activity_id.set(Some(aid));
                run_description.set(desc_opt.unwrap_or_default());
                run_billable.set(bill);
                running.set(Some(dto));
                project_id.set(None);
                activity_id.set(None);
                description.set(String::new());
                billable.set(true);
                props.on_timer_changed.call(());
            }
            Err(e) => toasts.push_error(e.to_string()),
        }
    };

    // Stop saves any in-progress edits to the running timer before stopping it,
    // so there is no need for a separate "Save" button while the timer is running.
    let on_stop = move |_| async move {
        let maybe_ts = running.peek().clone();
        let Some(ts) = maybe_ts else { return };
        let ts_id = ts.id.clone();

        // Persist description / billable changes.
        let desc = {
            let s = run_description.peek().clone();
            if s.is_empty() {
                None
            } else {
                Some(s)
            }
        };
        if let Err(e) = api::timesheet::update_timesheet(ts_id.clone(), desc).await {
            toasts.push_error(e.to_string());
            return;
        }

        // Persist activity reassignment if changed.
        let new_aid = run_activity_id.peek().clone();
        if let Some(aid) = new_aid.clone() {
            let needs_reassign = ts.activity_id.as_deref() != Some(aid.as_str());
            if needs_reassign {
                if let Err(e) = api::timesheet::reassign_timesheet(ts_id.clone(), aid).await {
                    toasts.push_error(e.to_string());
                    return;
                }
            }
        }

        match api::timesheet::stop_timesheet(ts_id).await {
            Ok(()) => {
                running.set(None);
                props.on_timer_changed.call(());
            }
            Err(e) => toasts.push_error(e.to_string()),
        }
    };

    let on_create_manual = move |_| async move {
        let start_local = manual_start.peek().clone();
        let end_local = manual_end.peek().clone();
        let aid = manual_activity_id.peek().clone();
        let desc_raw = manual_description.peek().clone();

        if start_local.is_empty() || end_local.is_empty() {
            toasts.push_error("Start and end time are required");
            return;
        }
        manual_submitting.set(true);
        let tz = user_settings.peek().timezone.clone();
        let start = formatting::from_input(&start_local, &tz);
        let end = formatting::from_input(&end_local, &tz);
        let desc = if desc_raw.is_empty() {
            None
        } else {
            Some(desc_raw)
        };
        match api::timesheet::create_timesheet_manual(aid, start, end, desc).await {
            Ok(_dto) => {
                manual_activity_id.set(None);
                manual_start.set(String::new());
                manual_end.set(String::new());
                manual_description.set(String::new());
                toasts.push_success("Timesheet created");
                props.on_timer_changed.call(());
            }
            Err(e) => toasts.push_error(e.to_string()),
        }
        manual_submitting.set(false);
    };

    rsx! {
        match running.read().clone() {
            Some(ts) => {
                rsx! {
                    Card { data_size: "md",
                        CardHeader {
                            CardTitle {
                                div { class: "flex items-center gap-2",
                                    Icon { icon: HiClock, width: 18, height: 18 }
                                    "Running Timer"
                                    span { class: "text-xs text-secondary font-normal ms-auto",
                                        {
                                            let s = user_settings.read();
                                            format!("Started: {}", formatting::format_datetime(&ts.start_time, &s.timezone, &s.date_format))
                                        }
                                    }
                                }
                            }
                        }
                        CardContent {
                            div { class: "grid grid-cols-1 gap-4 md:grid-cols-2",
                                div { class: "form-field",
                                    label { class: "form-label", "Activity" }
                                    Select::<String> {
                                        options: activities.read().iter()
                                            .map(|a| SelectOption::new(a.id.clone(), a.name.clone()))
                                            .collect(),
                                        value: run_activity_id.read().clone(),
                                        on_change: move |id: String| run_activity_id.set(Some(id)),
                                        placeholder: "Select activity…".to_string(),
                                    }
                                }
                                div { class: "form-field",
                                    label { class: "form-label", r#for: "run-desc", "Description" }
                                    Input {
                                        id: "run-desc",
                                        placeholder: "What are you working on?",
                                        value: run_description.read().clone(),
                                        oninput: move |e: FormEvent| run_description.set(e.value()),
                                    }
                                }
                                div { class: "form-field flex items-center gap-3",
                                    label { class: "form-label", "Billable" }
                                    input {
                                        r#type: "checkbox",
                                        class: "form-checkbox",
                                        checked: *run_billable.read(),
                                        oninput: move |_| { let v = *run_billable.peek(); run_billable.set(!v); },
                                    }
                                }
                            }
                        }
                        CardFooter {
                            Button { onclick: on_stop,
                                Icon { icon: HiStop, width: 16, height: 16 }
                                "Stop"
                            }
                        }
                    }
                }
            },
            None => rsx! {
                Card { data_size: "md",
                    CardHeader {
                        CardTitle {
                            div { class: "flex items-center justify-between",
                                div { class: "flex items-center gap-2",
                                    if *manual_mode.read() {
                                        Icon { icon: HiPlus, width: 18, height: 18 }
                                        "Manual Entry"
                                    } else {
                                        Icon { icon: HiPlay, width: 18, height: 18 }
                                        "Start Timer"
                                    }
                                }
                                div { class: "flex gap-1 text-xs",
                                    button {
                                        class: if !*manual_mode.read() { "tab-pill tab-pill--active" } else { "tab-pill" },
                                        onclick: move |_| manual_mode.set(false),
                                        "Timer"
                                    }
                                    button {
                                        class: if *manual_mode.read() { "tab-pill tab-pill--active" } else { "tab-pill" },
                                        onclick: move |_| manual_mode.set(true),
                                        "Manual"
                                    }
                                }
                            }
                        }
                    }

                    if !*manual_mode.read() {
                        CardContent {
                            div { class: "grid grid-cols-1 gap-4 md:grid-cols-2",
                                div { class: "form-field",
                                    label { class: "form-label", "Activity" }
                                    Select::<String> {
                                        options: activities.read().iter()
                                            .map(|a| SelectOption::new(a.id.clone(), a.name.clone()))
                                            .collect(),
                                        value: activity_id.read().clone(),
                                        on_change: move |id: String| activity_id.set(Some(id)),
                                        placeholder: "Select activity…".to_string(),
                                    }
                                }
                                div { class: "form-field",
                                    label { class: "form-label", r#for: "ts-desc", "Description" }
                                    Input {
                                        id: "ts-desc",
                                        placeholder: "Optional notes…",
                                        value: description.read().clone(),
                                        oninput: move |e: FormEvent| description.set(e.value()),
                                    }
                                }
                                div { class: "form-field flex items-center gap-3",
                                    label { class: "form-label", "Billable" }
                                    input {
                                        r#type: "checkbox",
                                        class: "form-checkbox",
                                        checked: *billable.read(),
                                        oninput: move |_| { let v = *billable.peek(); billable.set(!v); },
                                    }
                                }
                            }
                        }
                        CardFooter {
                            Button { onclick: on_start,
                                Icon { icon: HiPlay, width: 16, height: 16 }
                                "Start"
                            }
                        }
                    } else {
                        CardContent {
                            div { class: "grid grid-cols-1 gap-4 md:grid-cols-2",
                                div { class: "form-field",
                                    label { class: "form-label", r#for: "m-start", "Start time" }
                                    input {
                                        id: "m-start",
                                        r#type: "datetime-local",
                                        class: "input",
                                        value: manual_start.read().clone(),
                                        oninput: move |e: FormEvent| manual_start.set(e.value()),
                                    }
                                }
                                div { class: "form-field",
                                    label { class: "form-label", r#for: "m-end", "End time" }
                                    input {
                                        id: "m-end",
                                        r#type: "datetime-local",
                                        class: "input",
                                        value: manual_end.read().clone(),
                                        oninput: move |e: FormEvent| manual_end.set(e.value()),
                                    }
                                }
                                div { class: "form-field",
                                    label { class: "form-label", "Activity" }
                                    Select::<String> {
                                        options: activities.read().iter()
                                            .map(|a| SelectOption::new(a.id.clone(), a.name.clone()))
                                            .collect(),
                                        value: manual_activity_id.read().clone(),
                                        on_change: move |id: String| manual_activity_id.set(Some(id)),
                                        placeholder: "Select activity…".to_string(),
                                    }
                                }
                                div { class: "form-field md:col-span-2",
                                    label { class: "form-label", r#for: "m-desc", "Description" }
                                    Input {
                                        id: "m-desc",
                                        placeholder: "Optional notes…",
                                        value: manual_description.read().clone(),
                                        oninput: move |e: FormEvent| manual_description.set(e.value()),
                                    }
                                }
                                div { class: "form-field flex items-center gap-3",
                                    label { class: "form-label", "Billable" }
                                    input {
                                        r#type: "checkbox",
                                        class: "form-checkbox",
                                        checked: *manual_billable.read(),
                                        oninput: move |_| { let v = *manual_billable.peek(); manual_billable.set(!v); },
                                    }
                                }
                            }
                        }
                        CardFooter {
                            Button {
                                onclick: on_create_manual,
                                disabled: *manual_submitting.read(),
                                if *manual_submitting.read() {
                                    Icon { icon: HiRefresh, width: 16, height: 16 }
                                    "Creating…"
                                } else {
                                    Icon { icon: HiPlus, width: 16, height: 16 }
                                    "Create"
                                }
                            }
                        }
                    }
                }
            },
        }
    }
}
