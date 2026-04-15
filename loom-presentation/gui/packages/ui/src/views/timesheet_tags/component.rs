use crate::components::atoms::card::{Card, CardContent, CardFooter, CardHeader, CardTitle};
use crate::components::atoms::{
    Button, ColumnDef, DataTable, Input, TableCell, TableExpandRow, TableRow, ToastExt, Toasts,
};
use crate::layouts::DefaultLayout;
use crate::TagsCache;
use api::timesheet_tag::TimesheetsTagDto;
use dioxus::prelude::*;
use dioxus_free_icons::icons::hi_solid_icons::{HiPencil, HiPlus, HiSave, HiTag, HiX};
use dioxus_free_icons::Icon;

const PAGE_SIZE: usize = 20;

#[component]
pub fn Tags() -> Element {
    let cache: TagsCache = use_context();
    let mut tags = use_signal(|| cache.read().clone());
    let mut loading = use_signal(|| cache.read().is_empty());
    let mut toasts: Toasts = use_context();
    let mut page = use_signal(|| 0_usize);

    let mut new_name = use_signal(String::new);

    let mut editing_id = use_signal(|| Option::<String>::None);
    let mut edit_name = use_signal(String::new);

    use_resource(move || async move {
        match api::timesheet_tag::list_tags().await {
            Ok(list) => tags.set(list),
            Err(e) => toasts.push_error(e.to_string()),
        }
        loading.set(false);
    });

    let on_create = move |_| async move {
        let name = new_name.peek().clone();
        if name.is_empty() {
            return;
        }
        match api::timesheet_tag::create_tag(name).await {
            Ok(dto) => {
                tags.write().push(dto);
                new_name.set(String::new());
                toasts.push_success("Tag created");
            }
            Err(e) => toasts.push_error(e.to_string()),
        }
    };

    let on_save = move |_| async move {
        let id = match editing_id.peek().clone() {
            Some(id) => id,
            None => return,
        };
        let name = edit_name.peek().clone();
        if name.is_empty() {
            return;
        }
        if let Err(e) = api::timesheet_tag::rename_tag(id.clone(), name.clone()).await {
            toasts.push_error(e.to_string());
            return;
        }
        if let Some(tag) = tags.write().iter_mut().find(|t| t.id == id) {
            tag.name = name;
        }
        editing_id.set(None);
        toasts.push_success("Tag renamed");
    };

    let total = tags.read().len();
    let current_page = *page.read();
    let page_items: Vec<TimesheetsTagDto> = tags
        .read()
        .iter()
        .skip(current_page * PAGE_SIZE)
        .take(PAGE_SIZE)
        .cloned()
        .collect();

    let columns = vec![ColumnDef::new("Name"), ColumnDef::new("").width("80px")];
    let col_count = columns.len();

    rsx! {
        DefaultLayout {
            div { class: "space-y-6",

                // ── Create form ──────────────────────────────────────────────
                Card { data_size: "md",
                    CardHeader {
                        CardTitle {
                            div { class: "flex items-center gap-2",
                                Icon { icon: HiTag, width: 18, height: 18 }
                                "New Tag"
                            }
                        }
                    }
                    CardContent {
                        div { class: "form-field",
                            label { class: "form-label", r#for: "tag-name", "Name" }
                            Input {
                                id: "tag-name",
                                placeholder: "e.g. urgent",
                                value: new_name.read().clone(),
                                oninput: move |e: FormEvent| new_name.set(e.value()),
                            }
                        }
                    }
                    CardFooter {
                        Button { onclick: on_create,
                            Icon { icon: HiPlus, width: 16, height: 16 }
                            "Create"
                        }
                    }
                }

                // ── Tag list ─────────────────────────────────────────────────
                div { class: "island",
                    div { class: "island-header",
                        span { class: "island-title", "Tags" }
                    }
                    DataTable {
                        columns,
                        total,
                        page: current_page,
                        page_size: PAGE_SIZE,
                        loading: *loading.read(),
                        on_page_change: move |p| page.set(p),

                        for tag in page_items {
                            {
                                let t = tag.clone();
                                let tid = t.id.clone();
                                let is_editing = editing_id.read().as_deref() == Some(t.id.as_str());

                                rsx! {
                                    TableRow { key: "{t.id}",
                                        TableCell { span { class: "font-medium", "{t.name}" } }
                                        TableCell {
                                            if is_editing {
                                                Button {
                                                    onclick: on_save,
                                                    Icon { icon: HiSave, width: 14, height: 14 }
                                                }
                                                Button {
                                                    onclick: move |_| editing_id.set(None),
                                                    Icon { icon: HiX, width: 14, height: 14 }
                                                }
                                            } else {
                                                Button {
                                                    onclick: move |_| {
                                                        let tag_name = tags.read()
                                                            .iter()
                                                            .find(|x| x.id == tid)
                                                            .map(|x| x.name.clone())
                                                            .unwrap_or_default();
                                                        edit_name.set(tag_name);
                                                        editing_id.set(Some(tid.clone()));
                                                    },
                                                    Icon { icon: HiPencil, width: 14, height: 14 }
                                                }
                                            }
                                        }
                                    }
                                    if is_editing {
                                        TableExpandRow { col_count,
                                            div { class: "flex items-end gap-3",
                                                div { class: "form-field flex-1",
                                                    label { class: "form-label", r#for: "et-name", "Name" }
                                                    Input {
                                                        id: "et-name",
                                                        value: edit_name.read().clone(),
                                                        oninput: move |e: FormEvent| edit_name.set(e.value()),
                                                    }
                                                }
                                                div { class: "flex gap-2",
                                                    Button { onclick: on_save,
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
