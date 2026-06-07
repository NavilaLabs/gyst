use crate::components::atoms::card::{Card, CardContent, CardFooter, CardHeader, CardTitle};
use crate::components::atoms::{
    Button, ButtonVariant, Input, SearchableSelect, Select, SelectOption, ToastExt, Toasts,
};
use crate::layouts::DefaultLayout;
use crate::PluginHostCtx;
use api::invitation::InvitationDto;
use api::member::MemberDto;
use api::permissions::PermissionDto;
use api::workspace_role::WorkspaceRoleDto;
use chrono::NaiveDate;
use dioxus::prelude::*;
use dioxus_extism_frontend::PluginSlot;
use dioxus_free_icons::icons::hi_solid_icons::{
    HiBadgeCheck, HiBell, HiCheck, HiDownload, HiMail, HiOfficeBuilding, HiPencil, HiPlus,
    HiRefresh, HiSave, HiShieldCheck, HiTag, HiTrash, HiUser, HiUsers, HiX,
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

fn ttl_options() -> Vec<SelectOption<u32>> {
    [(7u32, "7 days"), (14, "14 days"), (30, "30 days")]
        .into_iter()
        .map(|(val, label)| SelectOption::new(val, label))
        .collect()
}

/// Extract up-to-two uppercase initials from a display name or email address.
fn name_initials(name: &str) -> String {
    let source = if name.contains('@') {
        name.split('@').next().unwrap_or(name)
    } else {
        name
    };
    let parts: Vec<&str> = source.split(['.', '_', '-', ' ']).collect();
    match parts.as_slice() {
        [a, b, ..] => {
            let a = a.chars().next().unwrap_or('?').to_uppercase().to_string();
            let b = b.chars().next().unwrap_or('?').to_uppercase().to_string();
            format!("{a}{b}")
        }
        [a] => a
            .chars()
            .next()
            .map(|c| c.to_uppercase().to_string())
            .unwrap_or_else(|| "?".to_string()),
        [] => "?".to_string(),
    }
}

// ── Tab ───────────────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
enum Tab {
    User,
    Workspace,
    Members,
    Smtp,
}

// ── Component ─────────────────────────────────────────────────────────────────

type AuthState = Signal<Option<Option<api::auth::UserInfo>>>;

#[component]
pub fn Settings() -> Element {
    let mut toasts: Toasts = use_context();
    let mut active_tab = use_signal(|| Tab::User);
    let mut i18n = i18n();

    let auth: AuthState = use_context();
    let user_email = auth.cloned().flatten().map(|u| u.email).unwrap_or_default();
    let initials = name_initials(&user_email);

    // Global context — read first so we can seed local signals.
    let mut global_user_settings: crate::UserSettings = use_context();
    let mut global_workspace_settings: crate::WorkspaceSettings = use_context();

    // ── User settings state ───────────────────────────────────────────────────
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

    // Notification toggles (local only, no backend yet)
    let mut notif_daily = use_signal(|| true);
    let mut notif_idle = use_signal(|| true);
    let mut notif_weekly = use_signal(|| false);

    // ── Workspace settings state ──────────────────────────────────────────────
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

    // ── Roles management state ────────────────────────────────────────────────
    let mut roles_with_perms = use_signal(Vec::<WorkspaceRoleDto>::new);
    let mut all_permissions = use_signal(Vec::<PermissionDto>::new);
    let mut new_role_name = use_signal(String::new);
    let mut role_expanded = use_signal(|| Option::<String>::None);
    let mut role_editing = use_signal(|| Option::<String>::None);
    let mut role_edit_name = use_signal(String::new);

    // ── Members state ─────────────────────────────────────────────────────────
    let mut members = use_signal(Vec::<MemberDto>::new);
    let mut member_role_dropdown = use_signal(|| Option::<String>::None);
    let mut workspace_invitations = use_signal(Vec::<InvitationDto>::new);
    let mut invite_email = use_signal(String::new);
    let mut invite_role_id = use_signal(String::new);
    let mut invite_ttl = use_signal(|| 7u32);
    let mut invite_sending = use_signal(|| false);

    // ── SMTP settings state ───────────────────────────────────────────────────
    let is_admin = auth.cloned().flatten().map(|u| u.is_admin).unwrap_or(false);
    let can_manage_workspace = auth
        .cloned()
        .flatten()
        .map(|u| u.can_manage_workspace)
        .unwrap_or(false);
    let mut smtp_auth_method = use_signal(|| "password".to_string());
    let mut smtp_host = use_signal(String::new);
    let mut smtp_port = use_signal(|| 587_u32);
    let mut smtp_username = use_signal(String::new);
    let mut smtp_from_address = use_signal(String::new);
    let mut smtp_use_tls = use_signal(|| true);
    let mut smtp_password = use_signal(String::new);
    let mut smtp_password_is_set = use_signal(|| false);
    let mut smtp_client_id = use_signal(String::new);
    let mut smtp_tenant_id = use_signal(String::new);
    let mut smtp_client_secret = use_signal(String::new);
    let mut smtp_client_secret_is_set = use_signal(|| false);
    let mut smtp_oauth2_email = use_signal(String::new);
    let mut smtp_oauth2_authorized = use_signal(|| false);
    let mut smtp_oauth2_url = use_signal(|| Option::<String>::None);
    let mut smtp_saving = use_signal(|| false);
    let mut smtp_test_to = use_signal(String::new);
    let mut smtp_testing = use_signal(|| false);

    // ── Data loading ──────────────────────────────────────────────────────────
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
        if let Ok(list) = api::workspace_role::list_roles_with_permissions().await {
            let default = list
                .iter()
                .find(|r| r.name == "standard")
                .or_else(|| list.first());
            if let Some(role) = default {
                invite_role_id.set(role.id.clone());
            }
            roles_with_perms.set(list);
        }
    });

    use_resource(move || async move {
        if let Ok(list) = api::permissions::list_permissions().await {
            all_permissions.set(list);
        }
    });

    use_resource(move || async move {
        if let Ok(list) = api::member::list_members().await {
            members.set(list);
        }
    });

    use_resource(move || async move {
        if let Ok(list) = api::invitation::list_invitations().await {
            workspace_invitations.set(list);
        }
    });

    use_resource(move || async move {
        if !is_admin {
            return;
        }
        if let Ok(dto) = api::smtp::get_smtp_config().await {
            smtp_auth_method.set(dto.auth_method);
            smtp_host.set(dto.host);
            smtp_port.set(u32::from(dto.port));
            smtp_username.set(dto.username);
            smtp_from_address.set(dto.from_address);
            smtp_use_tls.set(dto.use_tls);
            smtp_password_is_set.set(dto.password_is_set);
            smtp_client_id.set(dto.client_id.unwrap_or_default());
            smtp_tenant_id.set(dto.tenant_id.unwrap_or_default());
            smtp_client_secret_is_set.set(dto.client_secret_is_set);
            smtp_oauth2_email.set(dto.oauth2_smtp_email.unwrap_or_default());
            smtp_oauth2_authorized.set(dto.oauth2_authorized);
        }
    });

    let on_save_smtp = move |_| {
        let auth_method = smtp_auth_method.read().clone();
        let host = smtp_host.read().clone();
        let port = *smtp_port.read();
        let username = smtp_username.read().clone();
        let from_address = smtp_from_address.read().clone();
        let use_tls = *smtp_use_tls.read();
        let pw = smtp_password.read().clone();
        let client_id = smtp_client_id.read().clone();
        let tenant_id = smtp_tenant_id.read().clone();
        let secret = smtp_client_secret.read().clone();
        let oauth2_email = smtp_oauth2_email.read().clone();

        async move {
            smtp_saving.set(true);
            let pw_opt = if pw.is_empty() { None } else { Some(pw) };
            let secret_opt = if secret.is_empty() {
                None
            } else {
                Some(secret)
            };
            match api::smtp::save_smtp_config(
                auth_method,
                host,
                u16::try_from(port).unwrap_or(587),
                username,
                from_address,
                use_tls,
                pw_opt,
                if client_id.is_empty() {
                    None
                } else {
                    Some(client_id)
                },
                secret_opt,
                if tenant_id.is_empty() {
                    None
                } else {
                    Some(tenant_id)
                },
                if oauth2_email.is_empty() {
                    None
                } else {
                    Some(oauth2_email)
                },
            )
            .await
            {
                Ok(()) => toasts.push_success("SMTP settings saved"),
                Err(e) => toasts.push_error(e.to_string()),
            }
            smtp_saving.set(false);
        }
    };

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
                let mut ws = global_workspace_settings.write();
                ws.name = name;
                ws.timezone = timezone;
                ws.date_format = date_format;
                ws.currency = currency;
                ws.week_start = week_start;
                toasts.push_success("Workspace settings saved");
            }
            Err(e) => toasts.push_error(e.to_string()),
        }
        ws_saving.set(false);
    };

    // Snapshots for RSX rendering (avoids holding read guards across closures)
    let roles_snapshot: Vec<WorkspaceRoleDto> = roles_with_perms.read().clone();
    let perms_snapshot: Vec<PermissionDto> = all_permissions.read().clone();
    let members_snapshot: Vec<MemberDto> = members.read().clone();
    let current_expanded = role_expanded.read().clone();
    let current_editing = role_editing.read().clone();
    let current_member_dropdown = member_role_dropdown.read().clone();

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }

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
                    if can_manage_workspace || is_admin {
                        button {
                            class: if *active_tab.read() == Tab::Workspace { "tab-pill tab-pill--active" } else { "tab-pill" },
                            onclick: move |_| active_tab.set(Tab::Workspace),
                            Icon { icon: HiOfficeBuilding, width: 14, height: 14 }
                            {tid!("settings-tab-workspace-settings")}
                        }
                    }
                    if can_manage_workspace || is_admin {
                        button {
                            class: if *active_tab.read() == Tab::Members { "tab-pill tab-pill--active" } else { "tab-pill" },
                            onclick: move |_| active_tab.set(Tab::Members),
                            Icon { icon: HiUsers, width: 14, height: 14 }
                            {tid!("settings-tab-members")}
                        }
                    }
                    if is_admin {
                        button {
                            class: if *active_tab.read() == Tab::Smtp { "tab-pill tab-pill--active" } else { "tab-pill" },
                            onclick: move |_| active_tab.set(Tab::Smtp),
                            Icon { icon: HiMail, width: 14, height: 14 }
                            {tid!("settings-tab-smtp")}
                        }
                    }
                }

                // ── My Settings tab ───────────────────────────────────────────
                if *active_tab.read() == Tab::User {
                    div { class: "settings-grid",

                        // Profile card
                        Card { data_size: "md",
                            CardHeader {
                                CardTitle {
                                    div { class: "flex items-center gap-2",
                                        Icon { icon: HiUser, width: 16, height: 16 }
                                        {tid!("settings-profile-title")}
                                    }
                                }
                            }
                            CardContent {
                                div { class: "settings-profile-row",
                                    div { class: "settings-avatar", "{initials}" }
                                    div { class: "flex flex-col gap-1",
                                        span { class: "text-sm font-medium", "{user_email}" }
                                        span { class: "text-xs text-secondary", "Member" }
                                    }
                                }
                                div { class: "space-y-4",
                                    div { class: "form-field",
                                        label { class: "form-label", {tid!("common-email")} }
                                        Input {
                                            value: user_email.clone(),
                                            disabled: true,
                                        }
                                    }
                                }
                            }
                        }

                        // Localization card
                        Card { data_size: "md",
                            CardHeader {
                                CardTitle {
                                    div { class: "flex items-center gap-2",
                                        Icon { icon: HiTag, width: 16, height: 16 }
                                        {tid!("settings-localization-title")}
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

                        // Notifications card — not yet implemented
                        div { class: "settings-card-disabled",
                        Card { data_size: "md",
                            CardHeader {
                                CardTitle {
                                    div { class: "flex items-center gap-2",
                                        Icon { icon: HiBell, width: 16, height: 16 }
                                        {tid!("settings-notifications-title")}
                                        span { class: "settings-coming-soon-badge", "Coming soon" }
                                    }
                                }
                            }
                            CardContent {
                                div { class: "settings-notif-rows",
                                    div { class: "settings-row-spaced",
                                        div { class: "settings-row-label",
                                            span { class: "settings-row-label-title", {tid!("settings-notifications-daily-digest")} }
                                            span { class: "settings-row-label-desc", {tid!("settings-notifications-daily-digest-desc")} }
                                        }
                                        button {
                                            class: if *notif_daily.read() { "settings-status-pill settings-status-pill--on" } else { "settings-status-pill" },
                                            onclick: move |_| { let v = *notif_daily.read(); notif_daily.set(!v); },
                                            span { class: "settings-status-pill-dot" }
                                            if *notif_daily.read() { "On" } else { "Off" }
                                        }
                                    }
                                    div { class: "settings-row-spaced",
                                        div { class: "settings-row-label",
                                            span { class: "settings-row-label-title", {tid!("settings-notifications-idle-reminder")} }
                                            span { class: "settings-row-label-desc", {tid!("settings-notifications-idle-reminder-desc")} }
                                        }
                                        button {
                                            class: if *notif_idle.read() { "settings-status-pill settings-status-pill--on" } else { "settings-status-pill" },
                                            onclick: move |_| { let v = *notif_idle.read(); notif_idle.set(!v); },
                                            span { class: "settings-status-pill-dot" }
                                            if *notif_idle.read() { "On" } else { "Off" }
                                        }
                                    }
                                    div { class: "settings-row-spaced",
                                        div { class: "settings-row-label",
                                            span { class: "settings-row-label-title", {tid!("settings-notifications-weekly-review")} }
                                            span { class: "settings-row-label-desc", {tid!("settings-notifications-weekly-review-desc")} }
                                        }
                                        button {
                                            class: if *notif_weekly.read() { "settings-status-pill settings-status-pill--on" } else { "settings-status-pill" },
                                            onclick: move |_| { let v = *notif_weekly.read(); notif_weekly.set(!v); },
                                            span { class: "settings-status-pill-dot" }
                                            if *notif_weekly.read() { "On" } else { "Off" }
                                        }
                                    }
                                }
                            }
                        }
                        } // end settings-card-disabled (Notifications)

                        // Security card — not yet implemented
                        div { class: "settings-card-disabled",
                        Card { data_size: "md",
                            CardHeader {
                                CardTitle {
                                    div { class: "flex items-center gap-2",
                                        Icon { icon: HiShieldCheck, width: 16, height: 16 }
                                        {tid!("settings-security-title")}
                                        span { class: "settings-coming-soon-badge", "Coming soon" }
                                    }
                                }
                            }
                            CardContent {
                                div { class: "settings-notif-rows",
                                    div { class: "settings-row-spaced",
                                        div { class: "settings-row-label",
                                            span { class: "settings-row-label-title", {tid!("settings-security-password")} }
                                            span { class: "settings-row-label-desc", {tid!("settings-security-password-desc")} }
                                        }
                                        Button {
                                            variant: ButtonVariant::Outline,
                                            {tid!("settings-security-password-change")}
                                        }
                                    }
                                    div { class: "settings-row-spaced",
                                        div { class: "settings-row-label",
                                            span { class: "settings-row-label-title", {tid!("settings-security-2fa")} }
                                            span { class: "settings-row-label-desc", {tid!("settings-security-2fa-desc")} }
                                        }
                                        span { class: "settings-status-pill",
                                            span { class: "settings-status-pill-dot" }
                                            "Off"
                                        }
                                    }
                                    div { class: "settings-row-spaced",
                                        div { class: "settings-row-label",
                                            span { class: "settings-row-label-title", {tid!("settings-security-sessions")} }
                                            span { class: "settings-row-label-desc", {tid!("settings-security-sessions-desc")} }
                                        }
                                        Button {
                                            variant: ButtonVariant::Outline,
                                            {tid!("settings-security-sessions-manage")}
                                        }
                                    }
                                }
                            }
                        }
                        } // end settings-card-disabled (Security)

                        // Plugin-contributed user settings sections (§12.2 — settings.sections).
                        PluginSlot::<PluginHostCtx> { name: "settings.sections".to_string() }
                    }
                }

                // ── Workspace tab ─────────────────────────────────────────────
                if *active_tab.read() == Tab::Workspace {
                    div { class: "settings-grid",

                        // Workspace settings card
                        Card { data_size: "md",
                            CardHeader {
                                CardTitle {
                                    div { class: "flex items-center gap-2",
                                        Icon { icon: HiOfficeBuilding, width: 16, height: 16 }
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
                        }

                        // Roles card — full width
                        div { class: "settings-grid-full",
                            Card { data_size: "md",
                                CardHeader {
                                    CardTitle {
                                        div { class: "flex items-center gap-2",
                                            Icon { icon: HiShieldCheck, width: 16, height: 16 }
                                            {tid!("settings-roles-title")}
                                        }
                                    }
                                }
                                CardContent {
                                    div { class: "space-y-2",
                                        for role in roles_snapshot.iter() {
                                            {
                                                let role = role.clone();
                                                let rid = role.id.clone();
                                                let rid_expand = rid.clone();
                                                let rid_edit_btn = rid.clone();
                                                let rid_edit_save = rid.clone();
                                                let rid_delete = rid.clone();
                                                let rname_display = role.name.clone();
                                                let rname_for_edit = role.name.clone();
                                                let role_perm_names = role.permissions.clone();
                                                let is_expanded = current_expanded.as_deref() == Some(rid.as_str());
                                                let is_editing = current_editing.as_deref() == Some(rid.as_str());

                                                rsx! {
                                                    div {
                                                        key: "{rid}",
                                                        class: "border border-border rounded-md overflow-hidden",

                                                        // Role header row
                                                        div { class: "flex items-center gap-2 px-3 py-2",
                                                            if is_editing {
                                                                div { class: "flex items-center gap-2 flex-1 min-w-0",
                                                                    Input {
                                                                        value: role_edit_name.read().clone(),
                                                                        oninput: move |e: FormEvent| role_edit_name.set(e.value()),
                                                                    }
                                                                    button {
                                                                        class: "shrink-0 p-1 text-success hover:opacity-80 cursor-pointer bg-transparent border-0",
                                                                        title: "Save rename",
                                                                        onclick: move |_| {
                                                                            let rid = rid_edit_save.clone();
                                                                            let name = role_edit_name.peek().trim().to_string();
                                                                            async move {
                                                                                if name.is_empty() { return; }
                                                                                match api::workspace_role::rename_role(rid.clone(), name.clone()).await {
                                                                                    Ok(()) => {
                                                                                        if let Some(r) = roles_with_perms.write().iter_mut().find(|r| r.id == rid) {
                                                                                            r.name = name;
                                                                                        }
                                                                                        role_editing.set(None);
                                                                                    }
                                                                                    Err(e) => toasts.push_error(e.to_string()),
                                                                                }
                                                                            }
                                                                        },
                                                                        Icon { icon: HiCheck, width: 14, height: 14 }
                                                                    }
                                                                    button {
                                                                        class: "shrink-0 p-1 text-secondary hover:text-primary cursor-pointer bg-transparent border-0",
                                                                        title: "Cancel",
                                                                        onclick: move |_| role_editing.set(None),
                                                                        Icon { icon: HiX, width: 14, height: 14 }
                                                                    }
                                                                }
                                                            } else {
                                                                span { class: "text-sm font-medium flex-1 min-w-0 truncate", "{rname_display}" }
                                                            }

                                                            div { class: "flex items-center gap-1 shrink-0",
                                                                // Toggle permissions panel
                                                                button {
                                                                    class: if is_expanded {
                                                                        "flex items-center gap-1 px-2 py-1 text-xs rounded border border-primary text-primary cursor-pointer bg-transparent"
                                                                    } else {
                                                                        "flex items-center gap-1 px-2 py-1 text-xs rounded border border-border text-secondary hover:border-primary hover:text-primary cursor-pointer bg-transparent"
                                                                    },
                                                                    onclick: move |_| {
                                                                        let rid = rid_expand.clone();
                                                                        let cur = role_expanded.peek().clone();
                                                                        if cur.as_deref() == Some(rid.as_str()) {
                                                                            role_expanded.set(None);
                                                                        } else {
                                                                            role_expanded.set(Some(rid));
                                                                        }
                                                                    },
                                                                    Icon { icon: HiShieldCheck, width: 12, height: 12 }
                                                                    {tid!("settings-roles-permissions-btn")}
                                                                }
                                                                // Rename button (hidden while editing)
                                                                if !is_editing {
                                                                    button {
                                                                        class: "p-1 text-secondary hover:text-primary cursor-pointer bg-transparent border-0",
                                                                        title: "Rename",
                                                                        onclick: move |_| {
                                                                            role_edit_name.set(rname_for_edit.clone());
                                                                            role_editing.set(Some(rid_edit_btn.clone()));
                                                                        },
                                                                        Icon { icon: HiPencil, width: 14, height: 14 }
                                                                    }
                                                                }
                                                                // Delete button
                                                                button {
                                                                    class: "p-1 text-secondary hover:text-error cursor-pointer bg-transparent border-0",
                                                                    title: "Delete role",
                                                                    onclick: move |_| {
                                                                        let rid = rid_delete.clone();
                                                                        async move {
                                                                            match api::workspace_role::delete_role(rid.clone()).await {
                                                                                Ok(()) => {
                                                                                    roles_with_perms.write().retain(|r| r.id != rid);
                                                                                    toasts.push_success("Role deleted");
                                                                                }
                                                                                Err(e) => toasts.push_error(e.to_string()),
                                                                            }
                                                                        }
                                                                    },
                                                                    Icon { icon: HiTrash, width: 14, height: 14 }
                                                                }
                                                            }
                                                        }

                                                        // Expandable permissions panel
                                                        if is_expanded {
                                                            div { class: "border-t border-border px-3 py-3 bg-[var(--color-surface)]",
                                                                if perms_snapshot.is_empty() {
                                                                    p { class: "text-xs text-secondary", {tid!("settings-roles-no-permissions")} }
                                                                } else {
                                                                    div { class: "flex flex-wrap gap-x-4 gap-y-2",
                                                                        for perm in perms_snapshot.iter() {
                                                                            {
                                                                                let perm = perm.clone();
                                                                                let pid = perm.id.clone();
                                                                                let pname = perm.name.clone();
                                                                                let pname_check = perm.name.clone();
                                                                                let rid_perm = rid.clone();
                                                                                let is_granted = role_perm_names.contains(&perm.name);

                                                                                rsx! {
                                                                                    label {
                                                                                        key: "{pid}",
                                                                                        class: "flex items-center gap-1.5 text-xs cursor-pointer select-none",
                                                                                        input {
                                                                                            r#type: "checkbox",
                                                                                            checked: is_granted,
                                                                                            onchange: move |e| {
                                                                                                let pid = pid.clone();
                                                                                                let pname = pname.clone();
                                                                                                let rid = rid_perm.clone();
                                                                                                let checked = e.checked();
                                                                                                async move {
                                                                                                    let result = if checked {
                                                                                                        api::workspace_role::grant_role_permission(rid.clone(), pid).await
                                                                                                    } else {
                                                                                                        api::workspace_role::revoke_role_permission(rid.clone(), pid).await
                                                                                                    };
                                                                                                    match result {
                                                                                                        Ok(()) => {
                                                                                                            if let Some(r) = roles_with_perms.write().iter_mut().find(|r| r.id == rid) {
                                                                                                                if checked {
                                                                                                                    if !r.permissions.contains(&pname) {
                                                                                                                        r.permissions.push(pname);
                                                                                                                    }
                                                                                                                } else {
                                                                                                                    r.permissions.retain(|p| p != &pname);
                                                                                                                }
                                                                                                            }
                                                                                                        }
                                                                                                        Err(e) => toasts.push_error(e.to_string()),
                                                                                                    }
                                                                                                }
                                                                                            },
                                                                                        }
                                                                                        "{pname_check}"
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

                                        // Create new role row
                                        div { class: "flex items-center gap-2 pt-3 border-t border-border",
                                            div { class: "flex-1",
                                                Input {
                                                    placeholder: tid!("settings-roles-new-placeholder"),
                                                    value: new_role_name.read().clone(),
                                                    oninput: move |e: FormEvent| new_role_name.set(e.value()),
                                                }
                                            }
                                            Button {
                                                disabled: new_role_name.read().trim().is_empty(),
                                                onclick: move |_| async move {
                                                    let name = new_role_name.peek().trim().to_string();
                                                    if name.is_empty() { return; }
                                                    match api::workspace_role::create_role(name.clone()).await {
                                                        Ok(id) => {
                                                            roles_with_perms.write().push(WorkspaceRoleDto {
                                                                id,
                                                                name,
                                                                permissions: vec![],
                                                            });
                                                            new_role_name.set(String::new());
                                                            toasts.push_success("Role created");
                                                        }
                                                        Err(e) => toasts.push_error(e.to_string()),
                                                    }
                                                },
                                                Icon { icon: HiPlus, width: 14, height: 14 }
                                                {tid!("common-create")}
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Danger zone card — full width, not yet implemented
                        div { class: "settings-grid-full settings-card-disabled",
                            Card {
                                data_size: "md",
                                class: "settings-danger-card",
                                CardHeader {
                                    CardTitle {
                                        div { class: "flex items-center gap-2 settings-danger-title",
                                            Icon { icon: HiTrash, width: 16, height: 16 }
                                            {tid!("settings-danger-zone-title")}
                                            span { class: "settings-coming-soon-badge", "Coming soon" }
                                        }
                                    }
                                }
                                CardContent {
                                    div { class: "settings-notif-rows",
                                        div { class: "settings-row-spaced",
                                            div { class: "settings-row-label",
                                                span { class: "settings-row-label-title", {tid!("settings-danger-export")} }
                                                span { class: "settings-row-label-desc", {tid!("settings-danger-export-desc")} }
                                            }
                                            Button {
                                                variant: ButtonVariant::Outline,
                                                Icon { icon: HiDownload, width: 14, height: 14 }
                                                {tid!("settings-danger-export-btn")}
                                            }
                                        }
                                        div { class: "settings-row-spaced",
                                            div { class: "settings-row-label",
                                                span { class: "settings-row-label-title settings-danger-title", {tid!("settings-danger-delete-workspace")} }
                                                span { class: "settings-row-label-desc", {tid!("settings-danger-delete-workspace-desc")} }
                                            }
                                            Button {
                                                variant: ButtonVariant::Destructive,
                                                Icon { icon: HiTrash, width: 14, height: 14 }
                                                {tid!("settings-danger-delete-workspace-btn")}
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Save button — full width
                        div { class: "settings-grid-full",
                            Button {
                                onclick: on_save_workspace,
                                disabled: *ws_saving.read(),
                                Icon { icon: HiSave, width: 16, height: 16 }
                                if *ws_saving.read() { {tid!("common-saving")} } else { {tid!("common-save-settings")} }
                            }
                        }

                        // Plugin-contributed workspace settings sections (§12.2 — workspace.settings.sections).
                        PluginSlot::<PluginHostCtx> { name: "workspace.settings.sections".to_string() }
                    }
                }

                // ── Members tab ───────────────────────────────────────────────
                if *active_tab.read() == Tab::Members {

                    // Members roster card
                    Card { data_size: "md",
                        CardHeader {
                            CardTitle {
                                div { class: "flex items-center gap-2",
                                    Icon { icon: HiUsers, width: 18, height: 18 }
                                    {tid!("settings-members-title")}
                                }
                            }
                        }
                        CardContent {
                            if members_snapshot.is_empty() {
                                p { class: "text-sm text-secondary", {tid!("settings-members-empty")} }
                            } else {
                                div { class: "space-y-3",
                                    for member in members_snapshot.iter() {
                                        {
                                            let member = member.clone();
                                            let uid = member.user_id.clone();
                                            let uid_remove = uid.clone();
                                            let uid_dropdown = uid.clone();
                                            let uid_assign = uid.clone();
                                            let member_initials = name_initials(
                                                if member.name.is_empty() { &member.email } else { &member.name }
                                            );
                                            let member_name = member.name.clone();
                                            let member_email = member.email.clone();
                                            let member_role_ids = member.role_ids.clone();
                                            let member_role_ids_for_filter = member.role_ids.clone();
                                            let is_dropdown_open = current_member_dropdown.as_deref() == Some(uid.as_str());

                                            // Available roles to add (not yet assigned)
                                            let available_roles: Vec<WorkspaceRoleDto> = roles_snapshot
                                                .iter()
                                                .filter(|r| !member_role_ids_for_filter.contains(&r.id))
                                                .cloned()
                                                .collect();

                                            rsx! {
                                                div {
                                                    key: "{uid}",
                                                    class: "flex items-start gap-3 py-2 border-b border-border last:border-0",

                                                    // Avatar
                                                    div { class: "settings-avatar shrink-0", "{member_initials}" }

                                                    // Name + email + role badges
                                                    div { class: "flex-1 min-w-0",
                                                        div { class: "flex flex-col gap-0.5 mb-1.5",
                                                            span { class: "text-sm font-medium truncate",
                                                                if member_name.is_empty() { "{member_email}" } else { "{member_name}" }
                                                            }
                                                            if !member_name.is_empty() {
                                                                span { class: "text-xs text-secondary truncate", "{member_email}" }
                                                            }
                                                        }

                                                        // Role badges
                                                        div { class: "flex flex-wrap items-center gap-1",
                                                            for role_id in member_role_ids.iter() {
                                                                {
                                                                    let rid = role_id.clone();
                                                                    let uid_revoke = uid.clone();
                                                                    let role_name = roles_snapshot
                                                                        .iter()
                                                                        .find(|r| r.id == rid)
                                                                        .map(|r| r.name.clone())
                                                                        .unwrap_or_else(|| rid.clone());

                                                                    rsx! {
                                                                        span {
                                                                            key: "{rid}",
                                                                            class: "inline-flex items-center gap-1 px-2 py-0.5 text-xs rounded-full bg-[var(--color-primary-faint)] text-[var(--color-primary)] border border-[var(--color-primary-faint)]",
                                                                            "{role_name}"
                                                                            button {
                                                                                class: "ml-0.5 opacity-60 hover:opacity-100 cursor-pointer bg-transparent border-0 p-0 leading-none",
                                                                                title: "Revoke role",
                                                                                onclick: move |_| {
                                                                                    let uid = uid_revoke.clone();
                                                                                    let rid = rid.clone();
                                                                                    async move {
                                                                                        match api::member::revoke_member_role(uid.clone(), rid.clone()).await {
                                                                                            Ok(()) => {
                                                                                                if let Some(m) = members.write().iter_mut().find(|m| m.user_id == uid) {
                                                                                                    m.role_ids.retain(|r| r != &rid);
                                                                                                }
                                                                                            }
                                                                                            Err(e) => toasts.push_error(e.to_string()),
                                                                                        }
                                                                                    }
                                                                                },
                                                                                Icon { icon: HiX, width: 10, height: 10 }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }

                                                            // Add role button / inline select
                                                            if !available_roles.is_empty() {
                                                                if is_dropdown_open {
                                                                    div { class: "flex items-center gap-1",
                                                                        Select::<String> {
                                                                            options: available_roles.iter().map(|r| SelectOption::new(r.id.clone(), r.name.clone())).collect(),
                                                                            value: None,
                                                                            on_change: move |selected_rid: String| {
                                                                                let uid = uid_assign.clone();
                                                                                let rid = selected_rid.clone();
                                                                                async move {
                                                                                    match api::member::assign_member_role(uid.clone(), rid.clone()).await {
                                                                                        Ok(()) => {
                                                                                            if let Some(m) = members.write().iter_mut().find(|m| m.user_id == uid) {
                                                                                                if !m.role_ids.contains(&rid) {
                                                                                                    m.role_ids.push(rid);
                                                                                                }
                                                                                            }
                                                                                            member_role_dropdown.set(None);
                                                                                        }
                                                                                        Err(e) => toasts.push_error(e.to_string()),
                                                                                    }
                                                                                }
                                                                            },
                                                                            placeholder: tid!("settings-invite-role-placeholder"),
                                                                        }
                                                                        button {
                                                                            class: "p-1 text-secondary hover:text-primary cursor-pointer bg-transparent border-0",
                                                                            onclick: move |_| member_role_dropdown.set(None),
                                                                            Icon { icon: HiX, width: 12, height: 12 }
                                                                        }
                                                                    }
                                                                } else {
                                                                    button {
                                                                        class: "flex items-center gap-0.5 px-1.5 py-0.5 text-xs rounded border border-dashed border-border text-secondary hover:border-primary hover:text-primary cursor-pointer bg-transparent",
                                                                        onclick: move |_| member_role_dropdown.set(Some(uid_dropdown.clone())),
                                                                        Icon { icon: HiPlus, width: 10, height: 10 }
                                                                        {tid!("common-role")}
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }

                                                    // Remove member button
                                                    button {
                                                        class: "shrink-0 flex items-center gap-1 px-2 py-1 text-xs rounded border border-border text-secondary hover:text-error hover:border-error transition-colors cursor-pointer bg-transparent",
                                                        title: "Remove from workspace",
                                                        onclick: move |_| {
                                                            let uid = uid_remove.clone();
                                                            async move {
                                                                match api::member::remove_member(uid.clone()).await {
                                                                    Ok(()) => {
                                                                        members.write().retain(|m| m.user_id != uid);
                                                                        toasts.push_success("Member removed");
                                                                    }
                                                                    Err(e) => toasts.push_error(e.to_string()),
                                                                }
                                                            }
                                                        },
                                                        Icon { icon: HiTrash, width: 12, height: 12 }
                                                        {tid!("settings-members-remove-btn")}
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

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
                                        options: roles_with_perms.read().iter().map(|r| SelectOption::new(r.id.clone(), r.name.clone())).collect(),
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
                                                                title: "Revoke invitation",
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

                // ── SMTP tab ──────────────────────────────────────────────────
                if *active_tab.read() == Tab::Smtp {
                    {
                        let is_xoauth2 = *smtp_auth_method.read() == "xoauth2";
                        let show_common = *smtp_auth_method.read() != "none";
                        let auth_method_options = vec![
                            crate::components::atoms::SelectOption::new("none".to_string(), "No email (disabled)"),
                            crate::components::atoms::SelectOption::new("password".to_string(), "Username / Password"),
                            crate::components::atoms::SelectOption::new("xoauth2".to_string(), "Microsoft 365 (OAuth2)"),
                        ];

                        rsx! {
                            // SMTP configuration card
                            Card { data_size: "md",
                                CardHeader {
                                    CardTitle {
                                        div { class: "flex items-center gap-2",
                                            Icon { icon: HiMail, width: 16, height: 16 }
                                            {tid!("settings-tab-smtp")}
                                        }
                                    }
                                }
                                CardContent {
                                    div { class: "space-y-4",
                                        div { class: "form-field",
                                            label { class: "form-label", {tid!("setup-smtp-auth-method-label")} }
                                            Select::<String> {
                                                options: auth_method_options,
                                                value: Some(smtp_auth_method.read().clone()),
                                                on_change: move |v: String| smtp_auth_method.set(v),
                                            }
                                        }

                                        if show_common {
                                            div { class: "form-field",
                                                label { class: "form-label", r#for: "s_smtp_host",
                                                    {tid!("setup-smtp-host-label")}
                                                }
                                                Input {
                                                    id: "s_smtp_host",
                                                    value: smtp_host.read().clone(),
                                                    oninput: move |e: FormEvent| smtp_host.set(e.value()),
                                                }
                                            }
                                            div { class: "form-field",
                                                label { class: "form-label", r#for: "s_smtp_port",
                                                    {tid!("setup-smtp-port-label")}
                                                }
                                                Input {
                                                    id: "s_smtp_port",
                                                    r#type: "number",
                                                    value: smtp_port.read().to_string(),
                                                    oninput: move |e: FormEvent| {
                                                        if let Ok(p) = e.value().parse::<u32>() { smtp_port.set(p); }
                                                    },
                                                }
                                            }
                                            div { class: "form-field",
                                                label { class: "form-label", r#for: "s_smtp_from",
                                                    {tid!("setup-smtp-from-address-label")}
                                                }
                                                Input {
                                                    id: "s_smtp_from",
                                                    r#type: "email",
                                                    value: smtp_from_address.read().clone(),
                                                    oninput: move |e: FormEvent| smtp_from_address.set(e.value()),
                                                }
                                            }

                                            div { class: "flex items-center gap-2",
                                                input {
                                                    id: "s_smtp_tls", r#type: "checkbox", class: "cursor-pointer",
                                                    checked: *smtp_use_tls.read(),
                                                    onchange: move |e: FormEvent| smtp_use_tls.set(e.checked()),
                                                }
                                                label { r#for: "s_smtp_tls", class: "text-sm cursor-pointer",
                                                    {tid!("setup-smtp-use-tls-label")}
                                                }
                                            }

                                            if !is_xoauth2 {
                                                div { class: "form-field",
                                                    label { class: "form-label", r#for: "s_smtp_user",
                                                        {tid!("setup-smtp-username-label")}
                                                    }
                                                    Input {
                                                        id: "s_smtp_user",
                                                        value: smtp_username.read().clone(),
                                                        oninput: move |e: FormEvent| smtp_username.set(e.value()),
                                                    }
                                                }
                                                div { class: "form-field",
                                                    label { class: "form-label", r#for: "s_smtp_pw",
                                                        {tid!("setup-smtp-password-label")}
                                                    }
                                                    Input {
                                                        id: "s_smtp_pw",
                                                        r#type: "password",
                                                        placeholder: if *smtp_password_is_set.read() {
                                                            tid!("setup-smtp-password-keep-placeholder")
                                                        } else {
                                                            String::new()
                                                        },
                                                        oninput: move |e: FormEvent| smtp_password.set(e.value()),
                                                    }
                                                }
                                            }

                                            if is_xoauth2 {
                                                div { class: "form-field",
                                                    label { class: "form-label", r#for: "s_smtp_cid",
                                                        {tid!("setup-smtp-client-id-label")}
                                                    }
                                                    Input {
                                                        id: "s_smtp_cid",
                                                        value: smtp_client_id.read().clone(),
                                                        oninput: move |e: FormEvent| smtp_client_id.set(e.value()),
                                                    }
                                                }
                                                div { class: "form-field",
                                                    label { class: "form-label", r#for: "s_smtp_tid",
                                                        {tid!("setup-smtp-tenant-id-label")}
                                                    }
                                                    Input {
                                                        id: "s_smtp_tid",
                                                        value: smtp_tenant_id.read().clone(),
                                                        oninput: move |e: FormEvent| smtp_tenant_id.set(e.value()),
                                                    }
                                                }
                                                div { class: "form-field",
                                                    label { class: "form-label", r#for: "s_smtp_sec",
                                                        {tid!("setup-smtp-client-secret-label")}
                                                    }
                                                    Input {
                                                        id: "s_smtp_sec",
                                                        r#type: "password",
                                                        placeholder: if *smtp_client_secret_is_set.read() {
                                                            tid!("setup-smtp-client-secret-keep-placeholder")
                                                        } else {
                                                            String::new()
                                                        },
                                                        oninput: move |e: FormEvent| smtp_client_secret.set(e.value()),
                                                    }
                                                }
                                                div { class: "form-field",
                                                    label { class: "form-label", r#for: "s_smtp_oemail",
                                                        {tid!("setup-smtp-oauth2-email-label")}
                                                    }
                                                    Input {
                                                        id: "s_smtp_oemail",
                                                        r#type: "email",
                                                        value: smtp_oauth2_email.read().clone(),
                                                        oninput: move |e: FormEvent| smtp_oauth2_email.set(e.value()),
                                                    }
                                                }

                                                // OAuth2 authorization row
                                                div { class: "flex flex-wrap items-center gap-3 mt-2",
                                                    if *smtp_oauth2_authorized.read() {
                                                        span { class: "flex items-center gap-1 text-sm text-green-600",
                                                            Icon { icon: HiBadgeCheck, width: 16, height: 16 }
                                                            {tid!("setup-smtp-authorized")}
                                                        }
                                                    } else {
                                                        Button {
                                                            r#type: "button",
                                                            variant: ButtonVariant::Outline,
                                                            onclick: move |_| {
                                                                let client_id = smtp_client_id.read().clone();
                                                                let tenant_id = smtp_tenant_id.read().clone();
                                                                let secret_v = smtp_client_secret.read().clone();
                                                                let oauth2_email_v = smtp_oauth2_email.read().clone();
                                                                async move {
                                                                    let secret_opt = if secret_v.is_empty() { None } else { Some(secret_v) };
                                                                    let email_opt = if oauth2_email_v.is_empty() { None } else { Some(oauth2_email_v) };
                                                                    let _ = api::smtp::save_smtp_config(
                                                                        "xoauth2".to_string(),
                                                                        smtp_host.read().clone(),
                                                                        u16::try_from(*smtp_port.read()).unwrap_or(587),
                                                                        smtp_username.read().clone(),
                                                                        smtp_from_address.read().clone(),
                                                                        *smtp_use_tls.read(),
                                                                        None,
                                                                        if client_id.is_empty() { None } else { Some(client_id.clone()) },
                                                                        secret_opt,
                                                                        if tenant_id.is_empty() { None } else { Some(tenant_id.clone()) },
                                                                        email_opt,
                                                                    )
                                                                    .await;
                                                                    if let Ok(url) = api::smtp::start_microsoft_oauth2(client_id, tenant_id).await {
                                                                        smtp_oauth2_url.set(Some(url));
                                                                    }
                                                                }
                                                            },
                                                            {tid!("setup-smtp-authorize-button")}
                                                        }
                                                        if let Some(url) = smtp_oauth2_url.read().clone() {
                                                            a {
                                                                href: "{url}",
                                                                target: "_blank",
                                                                rel: "noopener noreferrer",
                                                                class: "text-sm underline text-primary",
                                                                {tid!("setup-smtp-open-link")}
                                                            }
                                                        }
                                                        Button {
                                                            r#type: "button",
                                                            variant: ButtonVariant::Secondary,
                                                            onclick: move |_| async move {
                                                                if let Ok(true) = api::smtp::oauth2_status().await {
                                                                    smtp_oauth2_authorized.set(true);
                                                                    toasts.push_success("OAuth2 authorization confirmed");
                                                                }
                                                            },
                                                            Icon { icon: HiRefresh, width: 14, height: 14 }
                                                            {tid!("setup-smtp-authorizing")}
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                CardFooter {
                                    Button {
                                        disabled: *smtp_saving.read(),
                                        onclick: on_save_smtp,
                                        Icon { icon: HiSave, width: 16, height: 16 }
                                        if *smtp_saving.read() { {tid!("common-saving")} } else { {tid!("settings-smtp-save")} }
                                    }
                                }
                            }

                            // Test email card
                            Card { data_size: "md",
                                CardHeader {
                                    CardTitle {
                                        div { class: "flex items-center gap-2",
                                            Icon { icon: HiCheck, width: 16, height: 16 }
                                            {tid!("settings-smtp-test-label")}
                                        }
                                    }
                                }
                                CardContent {
                                    div { class: "flex items-end gap-3",
                                        div { class: "form-field flex-1",
                                            label { class: "form-label", r#for: "smtp_test_to",
                                                {tid!("settings-smtp-test-label")}
                                            }
                                            Input {
                                                id: "smtp_test_to",
                                                r#type: "email",
                                                placeholder: "recipient@example.com",
                                                value: smtp_test_to.read().clone(),
                                                oninput: move |e: FormEvent| smtp_test_to.set(e.value()),
                                            }
                                        }
                                        Button {
                                            disabled: *smtp_testing.read() || smtp_test_to.read().is_empty(),
                                            onclick: move |_| {
                                                let to = smtp_test_to.read().clone();
                                                async move {
                                                    smtp_testing.set(true);
                                                    match api::smtp::test_smtp_connection(to).await {
                                                        Ok(()) => toasts.push_success(tid!("settings-smtp-test-success")),
                                                        Err(e) => toasts.push_error(e.to_string()),
                                                    }
                                                    smtp_testing.set(false);
                                                }
                                            },
                                            Icon { icon: HiMail, width: 14, height: 14 }
                                            if *smtp_testing.read() { {tid!("common-saving")} } else { {tid!("settings-smtp-test-button")} }
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
