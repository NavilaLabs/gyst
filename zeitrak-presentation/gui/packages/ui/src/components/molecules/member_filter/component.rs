use crate::components::atoms::{Select, SelectOption};
use dioxus::prelude::*;

/// A member selector dropdown. Shows "All members" plus one entry per workspace member.
///
/// Emits `None` when "All members" is selected, and `Some(user_id)` for a specific member.
/// Only render this component when the caller has `timesheet.read_all` access.
#[derive(Clone, PartialEq, Props)]
pub struct MemberFilterProps {
    pub members: Signal<Vec<api::member::MemberDto>>,
    pub selected: Signal<Option<String>>,
    pub on_change: EventHandler<Option<String>>,
}

/// Empty string is the sentinel for "All members".
const ALL_MEMBERS_SENTINEL: &str = "";

#[component]
pub fn MemberFilter(props: MemberFilterProps) -> Element {
    let members = props.members;
    let selected = props.selected;

    let mut options = vec![SelectOption::new(
        ALL_MEMBERS_SENTINEL.to_string(),
        "All members",
    )];
    options.extend(
        members
            .read()
            .iter()
            .map(|m| SelectOption::new(m.user_id.clone(), m.name.clone())),
    );

    let current_value = selected
        .read()
        .clone()
        .unwrap_or_else(|| ALL_MEMBERS_SENTINEL.to_string());

    rsx! {
        Select::<String> {
            options,
            value: Some(current_value),
            on_change: move |v: String| {
                let new_val = if v.is_empty() { None } else { Some(v) };
                props.on_change.call(new_val);
            },
            placeholder: "All members",
        }
    }
}
