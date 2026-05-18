# Configuring Microsoft 365 SMTP (OAuth2 / XOAUTH2)

Zeitrak supports sending transactional emails (invitations, email verification) through
Microsoft 365 / Exchange Online using OAuth2 instead of a plain password. This guide
walks through the one-time Azure setup and the in-app configuration.

---

## Prerequisites

- A **Microsoft 365 account** with a mailbox you want to send from.
- Access to **Microsoft Entra ID** (formerly Azure Active Directory) in the
  [Azure portal](https://portal.azure.com) — Global Administrator or Application
  Administrator role is required to register an app and grant consent.
- SMTP AUTH must be enabled for the mailbox (see Step 4).

---

## Step 1 — Register an Azure application

1. Go to [portal.azure.com](https://portal.azure.com) → **Microsoft Entra ID** →
   **App registrations** → **New registration**.
2. Fill in the form:
   - **Name**: anything descriptive, e.g. `Zeitrak SMTP`
   - **Supported account types**: *Accounts in this organizational directory only*
     (single-tenant)
   - **Redirect URI**: select **Web**, then enter
     `https://<your-zeitrak-domain>/api/smtp/oauth2/callback`
     (use `http://localhost:<port>` for local development)
3. Click **Register**.
4. On the overview page, copy the two values you will need later:
   - **Application (client) ID**
   - **Directory (tenant) ID**

---

## Step 2 — Create a client secret

1. In your app, go to **Certificates & secrets** → **Client secrets** →
   **New client secret**.
2. Choose an expiry (e.g. 24 months) and click **Add**.
3. **Copy the secret `Value` immediately** — it is only shown once.

---

## Step 3 — Add API permissions

1. Go to **API permissions** → **Add a permission** → **Microsoft Graph** →
   **Delegated permissions**.
2. Search for and select:
   - `SMTP.Send`
   - `offline_access` (required to obtain a refresh token)
3. Click **Add permissions**.
4. Click **Grant admin consent for \<your tenant\>** and confirm.
   The status column must show a green tick for both permissions.

> **Alternative:** If Microsoft Graph does not show `SMTP.Send`, use
> **Office 365 Exchange Online** → **Delegated permissions** → `SMTP.Send` instead.

---

## Step 4 — Enable SMTP AUTH on the mailbox

By default, Microsoft 365 disables legacy authentication per mailbox. OAuth2 SMTP
still requires the "Authenticated SMTP" setting to be on.

1. Go to the [Microsoft 365 Admin Center](https://admin.microsoft.com) →
   **Users** → **Active users** → click the mailbox you want to send from.
2. In the side panel, open the **Mail** tab → click **Manage email apps**.
3. Enable **Authenticated SMTP** and save.

You can also do this in Exchange Online PowerShell:
```powershell
Set-CASMailbox -Identity user@example.com -SmtpClientAuthenticationDisabled $false
```

---

## Step 5 — Configure Zeitrak

Open the **Setup wizard** (first-time setup) or navigate to
**Settings → SMTP / Email** (existing installation).

| Field | Value |
|---|---|
| Authentication method | **Microsoft 365 (OAuth2)** |
| SMTP host | `smtp.office365.com` |
| Port | `587` |
| From address | The mailbox address, e.g. `noreply@yourcompany.com` |
| STARTTLS | Enabled |
| Application (client) ID | From Step 1 |
| Directory (tenant) ID | From Step 1 |
| Client secret | From Step 2 |
| Microsoft 365 mailbox address | Same as the from address |

Click **Authorize with Microsoft**. A new browser tab opens with the Microsoft
login page. Sign in with the mailbox account, review the requested permissions, and
accept. Microsoft redirects back to Zeitrak automatically.

Back in the SMTP form, click **Check authorization status**. The badge should
change to **Authorized**. Click **Save SMTP settings**.

Use the **Send test email** card (Settings page only) to verify that delivery works
end-to-end before relying on it for invitations.

---

## How it works internally

Zeitrak uses the **OAuth2 authorization code flow**:

1. The **Authorize** button saves the current config and calls
   `/api/smtp/oauth2/microsoft/start`, which generates a random CSRF `state`,
   stores it in the `smtp_config` table, and returns an authorization URL pointing
   to `https://login.microsoftonline.com/{tenant}/oauth2/v2.0/authorize`.
2. After the user consents, Microsoft redirects to
   `/api/smtp/oauth2/callback?code=…&state=…`. The callback handler exchanges the
   code for tokens, validates the CSRF state, and stores the encrypted
   `refresh_token` in the admin database.
3. When Zeitrak sends an email, it exchanges the refresh token for a short-lived
   `access_token` (cached in memory for up to its expiry minus 60 seconds) and
   authenticates to `smtp.office365.com:587` using `XOAUTH2`.

Sensitive values (`password`, `client_secret`, `refresh_token`) are encrypted at
rest with **AES-256-GCM**. The encryption key is derived from
`authentication_secret` in your Zeitrak config via SHA-256.

---

## Troubleshooting

| Symptom | Likely cause |
|---|---|
| Authorization URL opens but consent fails | App permissions not granted or admin consent not given |
| "Check authorization status" stays unauthorized | OAuth2 callback did not reach Zeitrak — check the redirect URI in Azure matches exactly |
| Emails fail with `535 5.7.139 Authentication unsuccessful` | SMTP AUTH is disabled on the mailbox (Step 4) |
| Emails fail with `535 5.7.3 Authentication unsuccessful` | The access token scope is wrong — verify `SMTP.Send` permission is granted |
| Client secret expired | Rotate the secret in Azure, update it in Zeitrak settings, re-authorize |
