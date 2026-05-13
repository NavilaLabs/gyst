use crate::components::atoms::card::{Card, CardContent, CardFooter, CardHeader, CardTitle};
use crate::components::atoms::{
    Button, Input, SearchableSelect, Select, SelectOption, ToastExt, Toasts,
};
use crate::layouts::DefaultLayout;
use api::invitation::InvitationDto;
use api::workspace_role::WorkspaceRoleDto;
use chrono::NaiveDate;
use dioxus::prelude::*;
use dioxus_free_icons::icons::hi_solid_icons::{
    HiOfficeBuilding, HiSave, HiTrash, HiUser, HiUsers,
};
use dioxus_free_icons::Icon;
use dioxus_i18n::{prelude::i18n, tid};
use unic_langid::langid;

// ── Helpers ───────────────────────────────────────────────────────────────────

/// All IANA timezone names from chrono-tz, sorted alphabetically.
pub fn timezone_options() -> Vec<SelectOption<String>> {
    let mut tzs: Vec<&str> = chrono_tz::TZ_VARIANTS.iter().map(|tz| tz.name()).collect();
    tzs.sort_unstable();
    tzs.into_iter()
        .map(|name| SelectOption::new(name.to_string(), name))
        .collect()
}

/// Common date formats expressed as chrono format strings.
/// The label shows what a sample date (2026-04-10) looks like in each format.
fn date_format_options() -> Vec<SelectOption<String>> {
    let sample = NaiveDate::from_ymd_opt(2026, 4, 10).expect("valid date");
    [
        ("%Y-%m-%d", "ISO 8601"),
        ("%d.%m.%Y", "European"),
        ("%m/%d/%Y", "US"),
        ("%d/%m/%Y", "UK"),
        ("%d %B %Y", "Long"),
    ]
    .into_iter()
    .map(|(fmt, style)| {
        let example = sample.format(fmt).to_string();
        SelectOption::new(fmt.to_string(), format!("{example} ({style})"))
    })
    .collect()
}

fn language_options() -> Vec<SelectOption<String>> {
    [("en", "English"), ("de", "Deutsch")]
        .into_iter()
        .map(|(val, label)| SelectOption::new(val.to_string(), label))
        .collect()
}

/// ISO 4217 currency codes with names.
pub fn currency_options() -> Vec<SelectOption<String>> {
    [
        ("AUD", "AUD — Australian Dollar"),
        ("CAD", "CAD — Canadian Dollar"),
        ("CHF", "CHF — Swiss Franc"),
        ("EUR", "EUR — Euro"),
        ("GBP", "GBP — British Pound"),
        ("JPY", "JPY — Japanese Yen"),
        ("NOK", "NOK — Norwegian Krone"),
        ("SEK", "SEK — Swedish Krona"),
        ("USD", "USD — US Dollar"),
    ]
    .into_iter()
    .map(|(val, label)| SelectOption::new(val.to_string(), label))
    .collect()
}

fn week_start_options() -> Vec<SelectOption<String>> {
    [
        ("monday", "Monday"),
        ("sunday", "Sunday"),
        ("saturday", "Saturday"),
    ]
    .into_iter()
    .map(|(val, label)| SelectOption::new(val.to_string(), label))
    .collect()
}

// ── Component ─────────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
enum Tab {
    User,
    Workspace,
    Members,
}

fn ttl_options() -> Vec<SelectOption<u32>> {
    [(7u32, "7 days"), (14, "14 days"), (30, "30 days")]
        .into_iter()
        .map(|(val, label)| SelectOption::new(val, label))
        .collect()
}

#[component]
pub fn Settings() -> Element {
    let mut toasts: Toasts = use_context();
    let mut active_tab = use_signal(|| Tab::User);
    let mut i18n = i18n();

    // Global context — read first so we can seed local signals.
    let mut global_user_settings: crate::UserSettings = use_context();
    let mut global_workspace_settings: crate::WorkspaceSettings = use_context();

    // ── User settings state (seeded from global context) ──────────────────────
    let mut user_timezone = {
        let v = global_user_settings.peek().timezone.clone();
        use_signal(move || v)
    };
    let mut user_date_format = {
        let v = global_user_settings.peek().date_format.clone();
        use_signal(move || v)
    };
    let mut user_language = {
        let v = global_user_settings.peek().language.clone();
        use_signal(move || v)
    };
    let mut user_saving = use_signal(|| false);
    let mut user_loaded = use_signal(|| false);

    // ── Workspace settings state (seeded from global context) ─────────────────
    let mut ws_name = {
        let v = global_workspace_settings
            .peek()
            .name
            .clone()
            .unwrap_or_default();
        use_signal(move || v)
    };
    let mut ws_timezone = {
        let v = global_workspace_settings.peek().timezone.clone();
        use_signal(move || v)
    };
    let mut ws_date_format = {
        let v = global_workspace_settings.peek().date_format.clone();
        use_signal(move || v)
    };
    let mut ws_currency = {
        let v = global_workspace_settings.peek().currency.clone();
        use_signal(move || v)
    };
    let mut ws_week_start = {
        let v = global_workspace_settings.peek().week_start.clone();
        use_signal(move || v)
    };
    let mut ws_saving = use_signal(|| false);
    let mut ws_loaded = use_signal(|| false);

    // ── Members state ─────────────────────────────────────────────────────
    let mut roles = use_signal(Vec::<WorkspaceRoleDto>::new);
    let mut workspace_invitations = use_signal(Vec::<InvitationDto>::new);
    let mut invite_email = use_signal(String::new);
    let mut invite_role_id = use_signal(String::new);
    let mut invite_ttl = use_signal(|| 7u32);
    let mut invite_sending = use_signal(|| false);

    // Load both on mount — overwrites the context-seeded values with fresh data.
    use_resource(move || async move {
        match api::settings::get_user_settings().await {
            Ok(dto) => {
                user_timezone.set(dto.timezone);
                user_date_format.set(dto.date_format);
                user_language.set(dto.language);
                user_loaded.set(true);
            }
            Err(e) => toasts.push_error(e.to_string()),
        }
    });

    use_resource(move || async move {
        match api::settings::get_workspace_settings().await {
            Ok(dto) => {
                ws_name.set(dto.name.unwrap_or_default());
                ws_timezone.set(dto.timezone);
                ws_date_format.set(dto.date_format);
                ws_currency.set(dto.currency);
                ws_week_start.set(dto.week_start);
                ws_loaded.set(true);
            }
            Err(e) => toasts.push_error(e.to_string()),
        }
    });

    use_resource(move || async move {
        if let Ok(list) = api::workspace_role::list_workspace_roles().await {
            if let Some(first) = list.first() {
                invite_role_id.set(first.id.clone());
            }
            roles.set(list);
        }
    });

    use_resource(move || async move {
        if let Ok(list) = api::invitation::list_invitations().await {
            workspace_invitations.set(list);
        }
    });

    let on_save_user = move |_| async move {
        let timezone = user_timezone.peek().clone();
        let date_format = user_date_format.peek().clone();
        let language = user_language.peek().clone();

        user_saving.set(true);
        match api::settings::update_user_settings(
            timezone.clone(),
            date_format.clone(),
            language.clone(),
        )
        .await
        {
            Ok(()) => {
                // Push to global context so other views update immediately.
                global_user_settings.write().timezone = timezone;
                global_user_settings.write().date_format = date_format;
                let lang_id = match language.as_str() {
                    "de" => langid!("de-DE"),
                    _ => langid!("en-US"),
                };
                i18n.set_language(lang_id);
                global_user_settings.write().language = language;
                toasts.push_success("User settings saved");
            }
            Err(e) => toasts.push_error(e.to_string()),
        }
        user_saving.set(false);
    };

    let on_save_workspace = move |_| async move {
        let name_raw = ws_name.peek().clone();
        let name = if name_raw.trim().is_empty() {
            None
        } else {
            Some(name_raw.trim().to_string())
        };
        let timezone = ws_timezone.peek().clone();
        let date_format = ws_date_format.peek().clone();
        let currency = ws_currency.peek().clone();
        let week_start = ws_week_start.peek().clone();

        ws_saving.set(true);
        match api::settings::update_workspace_settings(
            name.clone(),
            timezone.clone(),
            date_format.clone(),
            currency.clone(),
            week_start.clone(),
        )
        .await
        {
            Ok(()) => {
                // Push to global context so other views update immediately.
                {
                    let mut ws = global_workspace_settings.write();
                    ws.name = name;
                    ws.timezone = timezone;
                    ws.date_format = date_format;
                    ws.currency = currency;
                    ws.week_start = week_start;
                }
                toasts.push_success("Workspace settings saved");
            }
            Err(e) => toasts.push_error(e.to_string()),
        }
        ws_saving.set(false);
    };

    rsx! {
        DefaultLayout {
            div { class: "space-y-6",

                // ── Tab pills ─────────────────────────────────────────────────
                div { class: "flex gap-2",
                    button {
                        class: if *active_tab.read() == Tab::User { "tab-pill tab-pill--active" } else { "tab-pill" },
                        onclick: move |_| active_tab.set(Tab::User),
                        Icon { icon: HiUser, width: 14, height: 14 }
                        {tid!("settings-tab-my-settings")}
                    }
                    button {
                        class: if *active_tab.read() == Tab::Workspace { "tab-pill tab-pill--active" } else { "tab-pill" },
                        onclick: move |_| active_tab.set(Tab::Workspace),
                        Icon { icon: HiOfficeBuilding, width: 14, height: 14 }
                        {tid!("settings-tab-workspace-settings")}
                    }
                    button {
                        class: if *active_tab.read() == Tab::Members { "tab-pill tab-pill--active" } else { "tab-pill" },
                        onclick: move |_| active_tab.set(Tab::Members),
                        Icon { icon: HiUsers, width: 14, height: 14 }
                        {tid!("settings-tab-members")}
                    }
                }

                // ── User settings ─────────────────────────────────────────────
                if *active_tab.read() == Tab::User {
                    Card { data_size: "md",
                        CardHeader {
                            CardTitle {
                                div { class: "flex items-center gap-2",
                                    Icon { icon: HiUser, width: 18, height: 18 }
                                    {tid!("settings-my-settings-title")}
                                }
                            }
                        }
                        CardContent {
                            div { class: "space-y-4",
                                div { class: "form-field",
                                    label { class: "form-label", {tid!("common-timezone")} }
                                    SearchableSelect::<String> {
                                        options: timezone_options(),
                                        value: Some(user_timezone.read().clone()),
                                        on_change: move |v| user_timezone.set(v),
                                        placeholder: tid!("common-select-timezone"),
                                    }
                                }
                                div { class: "form-field",
                                    label { class: "form-label", {tid!("common-date-format")} }
                                    Select::<String> {
                                        options: date_format_options(),
                                        value: Some(user_date_format.read().clone()),
                                        on_change: move |v| user_date_format.set(v),
                                        placeholder: tid!("common-select-format"),
                                    }
                                }
                                div { class: "form-field",
                                    label { class: "form-label", {tid!("common-language")} }
                                    Select::<String> {
                                        options: language_options(),
                                        value: Some(user_language.read().clone()),
                                        on_change: move |v| user_language.set(v),
                                        placeholder: tid!("common-select-language"),
                                    }
                                }
                            }
                        }
                        CardFooter {
                            Button {
                                onclick: on_save_user,
                                disabled: *user_saving.read(),
                                Icon { icon: HiSave, width: 16, height: 16 }
                                if *user_saving.read() { {tid!("common-saving")} } else { {tid!("common-save-settings")} }
                            }
                        }
                    }
                }

                // ── Members ───────────────────────────────────────────────────
                if *active_tab.read() == Tab::Members {
                    // Invite member card
                    Card { data_size: "md",
                        CardHeader {
                            CardTitle {
                                div { class: "flex items-center gap-2",
                                    Icon { icon: HiUsers, width: 18, height: 18 }
                                    {tid!("settings-invite-member-title")}
                                }
                            }
                        }
                        CardContent {
                            div { class: "space-y-4",
                                div { class: "form-field",
                                    label { class: "form-label", {tid!("settings-invite-email-label")} }
                                    Input {
                                        placeholder: tid!("settings-invite-email-placeholder"),
                                        value: invite_email.read().clone(),
                                        oninput: move |e: FormEvent| invite_email.set(e.value()),
                                    }
                                }
                                div { class: "form-field",
                                    label { class: "form-label", {tid!("common-role")} }
                                    Select::<String> {
                                        options: roles.read().iter().map(|r| SelectOption::new(r.id.clone(), r.name.clone())).collect(),
                                        value: Some(invite_role_id.read().clone()),
                                        on_change: move |v| invite_role_id.set(v),
                                        placeholder: tid!("settings-invite-role-placeholder"),
                                    }
                                }
                                div { class: "form-field",
                                    label { class: "form-label", {tid!("settings-invite-expires-label")} }
                                    Select::<u32> {
                                        options: ttl_options(),
                                        value: Some(*invite_ttl.read()),
                                        on_change: move |v| invite_ttl.set(v),
                                        placeholder: tid!("settings-invite-duration-placeholder"),
                                    }
                                }
                            }
                        }
                        CardFooter {
                            Button {
                                disabled: *invite_sending.read() || invite_email.read().is_empty() || invite_role_id.read().is_empty(),
                                onclick: move |_| async move {
                                    let email = invite_email.peek().clone();
                                    let role_id = invite_role_id.peek().clone();
                                    let ttl = *invite_ttl.peek();
                                    invite_sending.set(true);
                                    match api::invitation::send_invitation(email, role_id, ttl).await {
                                        Ok(_) => {
                                            invite_email.set(String::new());
                                            toasts.push_success("Invitation sent");
                                            if let Ok(list) = api::invitation::list_invitations().await {
                                                workspace_invitations.set(list);
                                            }
                                        }
                                        Err(e) => toasts.push_error(e.to_string()),
                                    }
                                    invite_sending.set(false);
                                },
                                if *invite_sending.read() { {tid!("settings-invite-sending")} } else { {tid!("settings-invite-send")} }
                            }
                        }
                    }

                    // Open invitations card
                    {
                        let pending: Vec<InvitationDto> = workspace_invitations
                            .read()
                            .iter()
                            .filter(|i| i.status == "pending")
                            .cloned()
                            .collect();

                        rsx! {
                            Card { data_size: "md",
                                CardHeader {
                                    CardTitle {
                                        div { class: "flex items-center gap-2",
                                            Icon { icon: HiUsers, width: 18, height: 18 }
                                            {tid!("settings-open-invitations-title")}
                                        }
                                    }
                                }
                                CardContent {
                                    if pending.is_empty() {
                                        p { class: "text-sm text-secondary",
                                            {tid!("settings-no-pending-invitations")}
                                        }
                                    } else {
                                        div { class: "space-y-2",
                                            for inv in pending.iter() {
                                                {
                                                    let inv = inv.clone();
                                                    let token = inv.token.clone();
                                                    let token_retain = inv.token.clone();
                                                    rsx! {
                                                        div {
                                                            key: "{inv.id}",
                                                            class: "flex items-center justify-between gap-3 py-2 border-b border-border last:border-0",
                                                            div { class: "flex flex-col gap-0.5 min-w-0",
                                                                span { class: "text-sm font-medium truncate",
                                                                    "{inv.email}"
                                                                }
                                                                span { class: "text-xs text-secondary", {tid!("settings-invitation-pending")} }
                                                            }
                                                            button {
                                                                class: "flex items-center gap-1 px-2 py-1 text-xs rounded border border-border text-secondary hover:text-error hover:border-error transition-colors cursor-pointer bg-transparent",
                                                                title: "Revoke invitation", // tooltip only, not translated
                                                                onclick: move |_| {
                                                                    let t = token.clone();
                                                                    let tr = token_retain.clone();
                                                                    async move {
                                                                        match api::invitation::revoke_invitation(t).await {
                                                                            Ok(()) => {
                                                                                workspace_invitations.write().retain(|i| i.token != tr);
                                                                                toasts.push_success("Invitation revoked");
                                                                            }
                                                                            Err(e) => toasts.push_error(e.to_string()),
                                                                        }
                                                                    }
                                                                },
                                                                Icon { icon: HiTrash, width: 12, height: 12 }
                                                                {tid!("settings-revoke-invitation")}
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

                // ── Workspace settings ────────────────────────────────────────
                if *active_tab.read() == Tab::Workspace {
                    Card { data_size: "md",
                        CardHeader {
                            CardTitle {
                                div { class: "flex items-center gap-2",
                                    Icon { icon: HiOfficeBuilding, width: 18, height: 18 }
                                    {tid!("settings-workspace-title")}
                                }
                            }
                        }
                        CardContent {
                            div { class: "space-y-4",
                                div { class: "form-field",
                                    label { class: "form-label", {tid!("common-workspace-name")} }
                                    Input {
                                        placeholder: tid!("settings-workspace-name-placeholder"),
                                        value: ws_name.read().clone(),
                                        oninput: move |e: FormEvent| ws_name.set(e.value()),
                                    }
                                }
                                div { class: "form-field",
                                    label { class: "form-label", {tid!("common-timezone")} }
                                    SearchableSelect::<String> {
                                        options: timezone_options(),
                                        value: Some(ws_timezone.read().clone()),
                                        on_change: move |v| ws_timezone.set(v),
                                        placeholder: tid!("common-select-timezone"),
                                    }
                                }
                                div { class: "form-field",
                                    label { class: "form-label", {tid!("common-date-format")} }
                                    Select::<String> {
                                        options: date_format_options(),
                                        value: Some(ws_date_format.read().clone()),
                                        on_change: move |v| ws_date_format.set(v),
                                        placeholder: tid!("common-select-format"),
                                    }
                                }
                                div { class: "form-field",
                                    label { class: "form-label", {tid!("settings-currency-label")} }
                                    Select::<String> {
                                        options: currency_options(),
                                        value: Some(ws_currency.read().clone()),
                                        on_change: move |v| ws_currency.set(v),
                                        placeholder: tid!("settings-currency-placeholder"),
                                    }
                                }
                                div { class: "form-field",
                                    label { class: "form-label", {tid!("settings-week-starts-label")} }
                                    Select::<String> {
                                        options: week_start_options(),
                                        value: Some(ws_week_start.read().clone()),
                                        on_change: move |v| ws_week_start.set(v),
                                        placeholder: tid!("settings-week-starts-placeholder"),
                                    }
                                }
                            }
                        }
                        CardFooter {
                            Button {
                                onclick: on_save_workspace,
                                disabled: *ws_saving.read(),
                                Icon { icon: HiSave, width: 16, height: 16 }
                                if *ws_saving.read() { {tid!("common-saving")} } else { {tid!("common-save-settings")} }
                            }
                        }
                    }
                }
            }
        }
    }
}
