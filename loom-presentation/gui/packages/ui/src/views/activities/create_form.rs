use crate::components::atoms::card::{Card, CardContent, CardFooter, CardHeader, CardTitle};
use crate::components::atoms::{Button, Input, ToastExt, Toasts};
use crate::form_machine::{new_form, FormAction, State};
use api::activity::ActivityDto;
use dioxus::prelude::*;
use dioxus_free_icons::icons::hi_solid_icons::{HiPlus, HiRefresh, HiTag};
use dioxus_free_icons::Icon;
use loom_core::{
    tenant::activity::CreateActivityInput,
    validation::{validation_summary, Validate},
};

#[derive(Clone, PartialEq, Props)]
pub(super) struct ActivityCreateFormProps {
    pub on_created: EventHandler<ActivityDto>,
}

#[component]
pub(super) fn ActivityCreateForm(props: ActivityCreateFormProps) -> Element {
    let mut toasts: Toasts = use_context();

    let mut create_form = use_signal(new_form);
    let mut new_name = use_signal(String::new);
    let mut new_comment = use_signal(String::new);
    let mut new_billable = use_signal(|| true);

    let on_create = move |_| async move {
        let name = new_name.peek().clone();

        create_form.write().handle(&FormAction::Submit);
        if let Err(e) = (CreateActivityInput { name: name.clone() }).validate() {
            create_form
                .write()
                .handle(&FormAction::Fail(validation_summary(&e)));
            return;
        }
        match api::activity::create_activity(name).await {
            Ok(dto) => {
                new_name.set(String::new());
                new_comment.set(String::new());
                create_form
                    .write()
                    .handle(&FormAction::Succeed("Activity created".into()));
                toasts.push_success("Activity created");
                props.on_created.call(dto);
            }
            Err(e) => {
                create_form.write().handle(&FormAction::Fail(e.to_string()));
                toasts.push_error(e.to_string());
            }
        }
    };

    let create_submitting = matches!(create_form.read().state(), State::Submitting {});

    rsx! {
        Card { data_size: "md",
            CardHeader {
                CardTitle {
                    div { class: "flex items-center gap-2",
                        Icon { icon: HiTag, width: 18, height: 18 }
                        "New Activity"
                    }
                }
            }
            CardContent {
                div { class: "grid grid-cols-1 gap-4 md:grid-cols-2",
                    div { class: "form-field",
                        label { class: "form-label", r#for: "a-name", "Name" }
                        Input {
                            id: "a-name",
                            placeholder: "Development",
                            value: new_name.read().clone(),
                            oninput: move |e: FormEvent| new_name.set(e.value()),
                        }
                    }
                    div { class: "form-field md:col-span-2",
                        label { class: "form-label", r#for: "a-comment", "Comment" }
                        Input {
                            id: "a-comment",
                            placeholder: "Optional description…",
                            value: new_comment.read().clone(),
                            oninput: move |e: FormEvent| new_comment.set(e.value()),
                        }
                    }
                    div { class: "form-field flex items-center gap-3",
                        label { class: "form-label", "Billable by default" }
                        input {
                            r#type: "checkbox",
                            class: "form-checkbox",
                            checked: *new_billable.read(),
                            oninput: move |_| { let v = *new_billable.peek(); new_billable.set(!v); },
                        }
                    }
                }
                if matches!(create_form.read().state(), State::Error {}) {
                    p { class: "text-red-500 text-sm mt-2",
                        "{create_form.read().message}"
                    }
                }
            }
            CardFooter {
                Button {
                    onclick: on_create,
                    disabled: create_submitting,
                    if create_submitting {
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
}
