use crate::components::atoms::card::{Card, CardContent, CardFooter, CardHeader, CardTitle};
use crate::components::atoms::{Button, Input, ToastExt, Toasts};
use crate::form_machine::{new_form, FormAction, State};
use api::activity::ActivityDto;
use dioxus::prelude::*;
use dioxus_free_icons::icons::hi_solid_icons::{HiPlus, HiRefresh, HiTag};
use dioxus_free_icons::Icon;
use dioxus_i18n::tid;
use zeitrak_core::{
    tenant::activity::CreateActivityInput,
    validation::{validation_summary, Validate},
};

const PALETTE: [&str; 8] = [
    "#22c55e", "#3b82f6", "#a855f7", "#f59e0b", "#06b6d4", "#ef4444", "#ec4899", "#84cc16",
];

#[derive(Clone, PartialEq, Props)]
pub(super) struct ActivityCreateFormProps {
    pub on_created: EventHandler<ActivityDto>,
}

#[component]
pub(super) fn ActivityCreateForm(props: ActivityCreateFormProps) -> Element {
    let mut toasts: Toasts = use_context();

    let mut create_form = use_signal(new_form);
    let mut new_name = use_signal(String::new);
    let mut new_color = use_signal(|| PALETTE[0].to_string());
    let mut new_comment = use_signal(String::new);

    let on_create = move |_| async move {
        let name = new_name.peek().clone();
        let color = new_color.peek().clone();

        create_form.write().handle(&FormAction::Submit);
        if let Err(e) = (CreateActivityInput { name: name.clone(), color: color.clone() }).validate() {
            create_form
                .write()
                .handle(&FormAction::Fail(validation_summary(&e)));
            return;
        }
        match api::activity::create_activity(name, color).await {
            Ok(dto) => {
                new_name.set(String::new());
                new_color.set(PALETTE[0].to_string());
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
                        {tid!("activities-new")}
                    }
                }
            }
            CardContent {
                div { class: "grid grid-cols-1 gap-4 md:grid-cols-2",
                    div { class: "form-field",
                        label { class: "form-label", r#for: "a-name", {tid!("common-name")} }
                        Input {
                            id: "a-name",
                            placeholder: tid!("activities-name-placeholder"),
                            value: new_name.read().clone(),
                            oninput: move |e: FormEvent| new_name.set(e.value()),
                        }
                    }
                    div { class: "form-field",
                        label { class: "form-label", {tid!("activities-color")} }
                        div { class: "flex flex-wrap gap-2 items-center",
                            for c in PALETTE {
                                button {
                                    r#type: "button",
                                    class: "zk-color-swatch",
                                    "data-selected": (*new_color.read() == c).to_string(),
                                    style: "background:{c}",
                                    onclick: {
                                        let c = c.to_string();
                                        move |_| new_color.set(c.clone())
                                    },
                                }
                            }
                            Input {
                                id: "a-color-hex",
                                placeholder: tid!("activities-color-hex-placeholder"),
                                value: new_color.read().clone(),
                                oninput: move |e: FormEvent| new_color.set(e.value()),
                                style: "width:110px; font-family:var(--font-mono); font-size:12px;",
                            }
                        }
                    }
                    div { class: "form-field md:col-span-2",
                        label { class: "form-label", r#for: "a-comment", {tid!("common-comment")} }
                        Input {
                            id: "a-comment",
                            placeholder: tid!("common-optional-description"),
                            value: new_comment.read().clone(),
                            oninput: move |e: FormEvent| new_comment.set(e.value()),
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
                        {tid!("common-creating")}
                    } else {
                        Icon { icon: HiPlus, width: 16, height: 16 }
                        {tid!("common-create")}
                    }
                }
            }
        }
    }
}
