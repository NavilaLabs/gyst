use crate::components::atoms::{
    Button, Input, TableCell, TableExpandRow, TableRow, ToastExt, Toasts,
};
use crate::form_machine::{new_form, FormAction, State};
use api::activity::ActivityDto;
use dioxus::prelude::*;
use dioxus_free_icons::icons::hi_solid_icons::{HiPencil, HiRefresh, HiSave, HiTrash, HiX};
use dioxus_free_icons::Icon;
use loom_core::{
    tenant::activity::UpdateActivityInput,
    validation::{validation_summary, Validate},
};

#[derive(Clone, PartialEq, Props)]
pub(super) struct ActivityRowProps {
    pub activity: ActivityDto,
    pub activities: Signal<Vec<ActivityDto>>,
    pub editing_id: Signal<Option<String>>,
    pub col_count: usize,
}

#[component]
pub(super) fn ActivityRow(props: ActivityRowProps) -> Element {
    let mut toasts: Toasts = use_context();

    let a = props.activity.clone();
    let aid = a.id.clone();
    let aid_delete = a.id.clone();
    let mut activities = props.activities;
    let mut editing_id = props.editing_id;
    let is_editing = editing_id.read().as_deref() == Some(a.id.as_str());

    let mut edit_form = use_signal(new_form);
    let mut edit_name = use_signal(String::new);
    let mut edit_comment = use_signal(String::new);
    let mut confirm_delete = use_signal(|| false);

    let on_save = move |_| async move {
        let id = match editing_id.peek().clone() {
            Some(id) => id,
            None => return,
        };
        let name = edit_name.peek().clone();
        let comment = {
            let s = edit_comment.peek().clone();
            if s.is_empty() {
                None
            } else {
                Some(s)
            }
        };

        edit_form.write().handle(&FormAction::Submit);
        if let Err(e) = (UpdateActivityInput { name: name.clone() }).validate() {
            edit_form
                .write()
                .handle(&FormAction::Fail(validation_summary(&e)));
            return;
        }

        if let Err(e) = api::activity::update_activity(id.clone(), name, comment).await {
            edit_form.write().handle(&FormAction::Fail(e.to_string()));
            toasts.push_error(e.to_string());
            return;
        }

        match api::activity::list_activities().await {
            Ok(list) => activities.set(list),
            Err(e) => toasts.push_error(e.to_string()),
        }
        edit_form
            .write()
            .handle(&FormAction::Succeed("Activity saved".into()));
        editing_id.set(None);
        toasts.push_success("Activity saved");
    };

    let on_delete = move |_| {
        let id = aid_delete.clone();
        async move {
            if let Err(e) = api::activity::delete_activity(id.clone()).await {
                toasts.push_error(e.to_string());
                return;
            }
            activities.write().retain(|x| x.id != id);
            confirm_delete.set(false);
            toasts.push_success("Activity deleted");
        }
    };

    let edit_submitting = matches!(edit_form.read().state(), State::Submitting {});

    rsx! {
        TableRow { key: "{a.id}",
            TableCell { "{a.name}" }
            TableCell {
                if is_editing {
                    Button {
                        onclick: move |_| { editing_id.set(None); confirm_delete.set(false); },
                        Icon { icon: HiX, width: 14, height: 14 }
                    }
                } else if *confirm_delete.read() {
                    div { class: "flex gap-1",
                        Button {
                            onclick: on_delete,
                            "Yes, delete"
                        }
                        Button {
                            onclick: move |_| confirm_delete.set(false),
                            "No"
                        }
                    }
                } else {
                    div { class: "flex gap-1",
                        Button {
                            onclick: move |_| {
                                let act = activities.read()
                                    .iter()
                                    .find(|x| x.id == aid)
                                    .cloned();
                                if let Some(ac) = act {
                                    edit_name.set(ac.name.clone());
                                    edit_comment.set(ac.comment.clone().unwrap_or_default());
                                    edit_form.write().handle(&FormAction::Reset);
                                    editing_id.set(Some(ac.id));
                                    confirm_delete.set(false);
                                }
                            },
                            Icon { icon: HiPencil, width: 14, height: 14 }
                        }
                        Button {
                            onclick: move |_| {
                                confirm_delete.set(true);
                                editing_id.set(None);
                            },
                            Icon { icon: HiTrash, width: 14, height: 14 }
                        }
                    }
                }
            }
        }
        if is_editing {
            TableExpandRow { col_count: props.col_count,
                div { class: "grid grid-cols-1 gap-4 md:grid-cols-2",
                    div { class: "form-field",
                        label { class: "form-label", r#for: "ea-name", "Name" }
                        Input {
                            id: "ea-name",
                            value: edit_name.read().clone(),
                            oninput: move |e: FormEvent| edit_name.set(e.value()),
                        }
                    }
                    div { class: "form-field",
                        label { class: "form-label", r#for: "ea-comment", "Comment" }
                        Input {
                            id: "ea-comment",
                            placeholder: "Optional description…",
                            value: edit_comment.read().clone(),
                            oninput: move |e: FormEvent| edit_comment.set(e.value()),
                        }
                    }
                }
                if matches!(edit_form.read().state(), State::Error {}) {
                    p { class: "text-red-500 text-sm mt-2",
                        "{edit_form.read().message}"
                    }
                }
                div { class: "flex gap-2 mt-2",
                    Button {
                        onclick: on_save,
                        disabled: edit_submitting,
                        if edit_submitting {
                            Icon { icon: HiRefresh, width: 14, height: 14 }
                            "Saving…"
                        } else {
                            Icon { icon: HiSave, width: 14, height: 14 }
                            "Save"
                        }
                    }
                    Button {
                        onclick: move |_| editing_id.set(None),
                        Icon { icon: HiX, width: 14, height: 14 }
                        "Cancel"
                    }
                }
            }
        }
    }
}
