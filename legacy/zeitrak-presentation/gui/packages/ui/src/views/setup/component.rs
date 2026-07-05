use crate::components::atoms::{
    Button, ButtonVariant, Card, CardContent, CardFooter, Form, FormField, Input, Label, Select,
    SelectOption, TabContent, TabList, TabTrigger, Tabs,
};
use crate::layouts::DefaultLayout;
use dioxus::prelude::*;
use dioxus_free_icons::icons::hi_solid_icons::{
    HiBadgeCheck, HiCheck, HiChevronLeft, HiChevronRight, HiRefresh,
};
use dioxus_free_icons::Icon;
use dioxus_i18n::tid;

#[component]
pub fn Setup() -> Element {
    // ── Tab state ─────────────────────────────────────────────────────────────
    let mut active_tab = use_signal(|| Some("admin".to_string()));

    // ── Admin tab ─────────────────────────────────────────────────────────────
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut confirm_password = use_signal(String::new);

    // ── Workspace tab ─────────────────────────────────────────────────────────
    let mut workspace_name = use_signal(String::new);

    // ── SMTP tab ──────────────────────────────────────────────────────────────
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

    // ── Shared ────────────────────────────────────────────────────────────────
    let mut error = use_signal(|| None::<String>);
    let mut submitting = use_signal(|| false);

    let navigator = use_navigator();

    // ── Pre-fill SMTP values from server on mount ─────────────────────────────
    use_resource(move || async move {
        if let Ok(dto) = api::smtp::smtp_prefill().await {
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

    // ── Submit: save SMTP (if configured) then create admin + workspace ────────
    let on_submit = move |_| {
        let email_val = email.read().clone();
        let password_val = password.read().clone();
        let workspace_name_val = workspace_name.read().clone();
        let auth_method_val = smtp_auth_method.read().clone();
        let host_val = smtp_host.read().clone();
        let port_val = *smtp_port.read();
        let username_val = smtp_username.read().clone();
        let from_address_val = smtp_from_address.read().clone();
        let use_tls_val = *smtp_use_tls.read();
        let pw_val = smtp_password.read().clone();
        let client_id_val = smtp_client_id.read().clone();
        let tenant_id_val = smtp_tenant_id.read().clone();
        let secret_val = smtp_client_secret.read().clone();
        let oauth2_email_val = smtp_oauth2_email.read().clone();

        async move {
            submitting.set(true);
            error.set(None);

            let skip = auth_method_val == "none";

            if !skip && !host_val.is_empty() {
                let pw_opt = if pw_val.is_empty() {
                    None
                } else {
                    Some(pw_val)
                };
                let secret_opt = if secret_val.is_empty() {
                    None
                } else {
                    Some(secret_val)
                };

                if let Err(e) = api::smtp::setup_save_smtp_config(
                    auth_method_val,
                    host_val,
                    u16::try_from(port_val).unwrap_or(587),
                    username_val,
                    from_address_val,
                    use_tls_val,
                    pw_opt,
                    if client_id_val.is_empty() {
                        None
                    } else {
                        Some(client_id_val)
                    },
                    secret_opt,
                    if tenant_id_val.is_empty() {
                        None
                    } else {
                        Some(tenant_id_val)
                    },
                    if oauth2_email_val.is_empty() {
                        None
                    } else {
                        Some(oauth2_email_val)
                    },
                )
                .await
                {
                    error.set(Some(e.to_string()));
                    submitting.set(false);
                    return;
                }
            }

            match api::setup::setup(
                "admin".to_string(),
                email_val,
                password_val,
                workspace_name_val,
            )
            .await
            {
                Ok(()) => {
                    navigator.push("/login");
                }
                Err(e) => {
                    error.set(Some(e.to_string()));
                    submitting.set(false);
                }
            }
        }
    };

    let on_skip = move |_| {
        let email_val = email.read().clone();
        let password_val = password.read().clone();
        let workspace_name_val = workspace_name.read().clone();
        async move {
            submitting.set(true);
            error.set(None);
            match api::setup::setup(
                "admin".to_string(),
                email_val,
                password_val,
                workspace_name_val,
            )
            .await
            {
                Ok(()) => {
                    navigator.push("/login");
                }
                Err(e) => {
                    error.set(Some(e.to_string()));
                    submitting.set(false);
                }
            }
        }
    };

    let auth_method_options = vec![
        SelectOption::new("none".to_string(), tid!("setup-smtp-auth-method-none")),
        SelectOption::new(
            "password".to_string(),
            tid!("setup-smtp-auth-method-password"),
        ),
        SelectOption::new(
            "xoauth2".to_string(),
            tid!("setup-smtp-auth-method-xoauth2"),
        ),
    ];

    let current_tab = active_tab.read().clone().unwrap_or_default();
    let is_xoauth2 = *smtp_auth_method.read() == "xoauth2";
    let show_common = *smtp_auth_method.read() != "none";

    rsx! {
        DefaultLayout {
            Card {
                class: "w-full",
                data_size: "md",
                Tabs {
                    value: active_tab,
                    default_value: "admin",
                    on_value_change: move |v: String| active_tab.set(Some(v)),
                    CardContent {
                        TabList {
                            class: "w-full mb-4",
                            TabTrigger { value: "admin", index: 0usize, {tid!("setup-tab-admin")} }
                            TabTrigger { value: "workspace", index: 1usize, {tid!("setup-tab-workspace")} }
                            TabTrigger { value: "smtp", index: 2usize, {tid!("setup-tab-smtp")} }
                        }

                        // ── Admin tab ─────────────────────────────────────────
                        TabContent { value: "admin", index: 0usize,
                            Form {
                                FormField {
                                    Label { html_for: "username", class: "w-full", {tid!("setup-username-label")} }
                                    Input { id: "username", class: "w-full", disabled: true, placeholder: "admin" }
                                }
                                FormField {
                                    Label { html_for: "email", class: "w-full", {tid!("common-email")} }
                                    Input {
                                        id: "email", r#type: "email", class: "w-full",
                                        oninput: move |e: FormEvent| email.set(e.value()),
                                    }
                                }
                                FormField {
                                    Label { html_for: "password", class: "w-full", {tid!("common-password")} }
                                    Input {
                                        id: "password", r#type: "password", class: "w-full",
                                        oninput: move |e: FormEvent| password.set(e.value()),
                                    }
                                }
                                FormField {
                                    Label { html_for: "confirm_password", class: "w-full", {tid!("setup-confirm-password-label")} }
                                    Input {
                                        id: "confirm_password", r#type: "password", class: "w-full",
                                        oninput: move |e: FormEvent| confirm_password.set(e.value()),
                                    }
                                }
                            }
                        }

                        // ── Workspace tab ─────────────────────────────────────
                        TabContent { value: "workspace", index: 1usize,
                            Form {
                                FormField {
                                    Label { html_for: "workspace_name", class: "w-full", {tid!("common-workspace-name")} }
                                    Input {
                                        id: "workspace_name", class: "w-full",
                                        oninput: move |e: FormEvent| workspace_name.set(e.value()),
                                    }
                                }
                            }
                        }

                        // ── SMTP tab ──────────────────────────────────────────
                        TabContent { value: "smtp", index: 2usize,
                            p { class: "text-sm text-muted mb-4", {tid!("setup-smtp-description")} }
                            Form {
                                FormField {
                                    Label { html_for: "smtp_method", class: "w-full", {tid!("setup-smtp-auth-method-label")} }
                                    Select::<String> {
                                        options: auth_method_options.clone(),
                                        value: Some(smtp_auth_method.read().clone()),
                                        on_change: move |v: String| smtp_auth_method.set(v),
                                    }
                                }

                                if show_common {
                                    FormField {
                                        Label { html_for: "smtp_host", class: "w-full", {tid!("setup-smtp-host-label")} }
                                        Input {
                                            id: "smtp_host", class: "w-full",
                                            value: smtp_host.read().clone(),
                                            oninput: move |e: FormEvent| smtp_host.set(e.value()),
                                        }
                                    }
                                    FormField {
                                        Label { html_for: "smtp_port", class: "w-full", {tid!("setup-smtp-port-label")} }
                                        Input {
                                            id: "smtp_port", r#type: "number", class: "w-full",
                                            value: smtp_port.read().to_string(),
                                            oninput: move |e: FormEvent| {
                                                if let Ok(p) = e.value().parse::<u32>() { smtp_port.set(p); }
                                            },
                                        }
                                    }
                                    FormField {
                                        Label { html_for: "smtp_from", class: "w-full", {tid!("setup-smtp-from-address-label")} }
                                        Input {
                                            id: "smtp_from", r#type: "email", class: "w-full",
                                            value: smtp_from_address.read().clone(),
                                            oninput: move |e: FormEvent| smtp_from_address.set(e.value()),
                                        }
                                    }

                                    div { class: "form-field flex items-center gap-2 my-2",
                                        input {
                                            id: "smtp_tls", r#type: "checkbox", class: "cursor-pointer",
                                            checked: *smtp_use_tls.read(),
                                            onchange: move |e: FormEvent| smtp_use_tls.set(e.checked()),
                                        }
                                        label { r#for: "smtp_tls", class: "text-sm cursor-pointer",
                                            {tid!("setup-smtp-use-tls-label")}
                                        }
                                    }

                                    if !is_xoauth2 {
                                        FormField {
                                            Label { html_for: "smtp_user", class: "w-full", {tid!("setup-smtp-username-label")} }
                                            Input {
                                                id: "smtp_user", class: "w-full",
                                                value: smtp_username.read().clone(),
                                                oninput: move |e: FormEvent| smtp_username.set(e.value()),
                                            }
                                        }
                                        FormField {
                                            Label { html_for: "smtp_pw", class: "w-full", {tid!("setup-smtp-password-label")} }
                                            Input {
                                                id: "smtp_pw", r#type: "password", class: "w-full",
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
                                        FormField {
                                            Label { html_for: "smtp_client_id", class: "w-full", {tid!("setup-smtp-client-id-label")} }
                                            Input {
                                                id: "smtp_client_id", class: "w-full",
                                                value: smtp_client_id.read().clone(),
                                                oninput: move |e: FormEvent| smtp_client_id.set(e.value()),
                                            }
                                        }
                                        FormField {
                                            Label { html_for: "smtp_tenant_id", class: "w-full", {tid!("setup-smtp-tenant-id-label")} }
                                            Input {
                                                id: "smtp_tenant_id", class: "w-full",
                                                value: smtp_tenant_id.read().clone(),
                                                oninput: move |e: FormEvent| smtp_tenant_id.set(e.value()),
                                            }
                                        }
                                        FormField {
                                            Label { html_for: "smtp_secret", class: "w-full", {tid!("setup-smtp-client-secret-label")} }
                                            Input {
                                                id: "smtp_secret", r#type: "password", class: "w-full",
                                                placeholder: if *smtp_client_secret_is_set.read() {
                                                    tid!("setup-smtp-client-secret-keep-placeholder")
                                                } else {
                                                    String::new()
                                                },
                                                oninput: move |e: FormEvent| smtp_client_secret.set(e.value()),
                                            }
                                        }
                                        FormField {
                                            Label { html_for: "smtp_oauth_email", class: "w-full", {tid!("setup-smtp-oauth2-email-label")} }
                                            Input {
                                                id: "smtp_oauth_email", r#type: "email", class: "w-full",
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
                                                        let client_secret_v = smtp_client_secret.read().clone();
                                                        let oauth2_email_v = smtp_oauth2_email.read().clone();
                                                        async move {
                                                            let secret_opt = if client_secret_v.is_empty() { None } else { Some(client_secret_v) };
                                                            let email_opt = if oauth2_email_v.is_empty() { None } else { Some(oauth2_email_v) };
                                                            let _ = api::smtp::setup_save_smtp_config(
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
                                                            if let Ok(url) = api::smtp::setup_start_microsoft_oauth2(client_id, tenant_id).await {
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
                                                        if let Ok(true) = api::smtp::setup_oauth2_status().await {
                                                            smtp_oauth2_authorized.set(true);
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

                        if let Some(msg) = error.read().as_deref() {
                            p { class: "text-red-500 text-sm mt-2", "{msg}" }
                        }
                    }
                }

                // ── Footer navigation ─────────────────────────────────────────
                CardFooter { class: "flex gap-2",
                    if current_tab == "workspace" {
                        Button {
                            variant: ButtonVariant::Secondary,
                            onclick: move |_| active_tab.set(Some("admin".to_string())),
                            Icon { icon: HiChevronLeft, width: 16, height: 16 }
                            {tid!("common-back")}
                        }
                    }
                    if current_tab == "smtp" {
                        Button {
                            variant: ButtonVariant::Secondary,
                            onclick: move |_| active_tab.set(Some("workspace".to_string())),
                            Icon { icon: HiChevronLeft, width: 16, height: 16 }
                            {tid!("common-back")}
                        }
                    }

                    div { class: "ms-auto flex gap-2",
                        if current_tab == "admin" {
                            Button {
                                onclick: move |_| active_tab.set(Some("workspace".to_string())),
                                {tid!("common-next")}
                                Icon { icon: HiChevronRight, width: 16, height: 16 }
                            }
                        } else if current_tab == "workspace" {
                            Button {
                                onclick: move |_| active_tab.set(Some("smtp".to_string())),
                                {tid!("common-next")}
                                Icon { icon: HiChevronRight, width: 16, height: 16 }
                            }
                        } else {
                            Button {
                                variant: ButtonVariant::Secondary,
                                disabled: *submitting.read(),
                                onclick: on_skip,
                                {tid!("setup-smtp-skip")}
                            }
                            Button {
                                r#type: "submit",
                                disabled: *submitting.read(),
                                onclick: on_submit,
                                if *submitting.read() {
                                    Icon { icon: HiRefresh, width: 16, height: 16 }
                                    {tid!("common-submitting")}
                                } else {
                                    Icon { icon: HiCheck, width: 16, height: 16 }
                                    {tid!("setup-smtp-finish")}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
