//! Display-formatting helpers that respect user and workspace settings.
//!
//! All timestamps in the system are stored as UTC RFC-3339.  These functions
//! convert them to the user's local timezone using `chrono-tz` and format
//! them with the user's chosen `chrono` date-format string.
//!
//! ## datetime-local inputs
//!
//! HTML `<input type="datetime-local">` is always timezone-naive.  The round-
//! trip helpers (`to_input` / `from_input`) translate between UTC RFC-3339 and
//! the naive local time the browser shows, so the server always receives proper
//! RFC-3339 strings and never needs to know about the browser timezone.

use chrono::{DateTime, LocalResult, NaiveDateTime, Utc};
use chrono_tz::Tz;

/// Parse a timezone name string into a `Tz`, falling back to UTC on error.
fn parse_tz(tz_name: &str) -> Tz {
    tz_name.parse().unwrap_or(chrono_tz::UTC)
}

/// Normalise legacy date-format strings to proper chrono format codes.
///
/// Early versions of the settings migration used `"YYYY-MM-DD"` (a common but
/// non-chrono format) as the default.  This function maps those values to their
/// chrono equivalents so that `format_datetime` / `format_date` always receive
/// valid format strings.
fn normalise_date_fmt(fmt: &str) -> &str {
    match fmt {
        "YYYY-MM-DD" => "%Y-%m-%d",
        "DD.MM.YYYY" => "%d.%m.%Y",
        "MM/DD/YYYY" => "%m/%d/%Y",
        "DD/MM/YYYY" => "%d/%m/%Y",
        _ => fmt,
    }
}

/// Format a UTC RFC-3339 timestamp as `<date> <HH:MM>` in the user's timezone,
/// using their chosen date-format string (chrono format codes).
///
/// Returns the raw RFC-3339 string unchanged on parse error.
pub fn format_datetime(rfc3339: &str, tz_name: &str, date_fmt: &str) -> String {
    let Ok(utc) = DateTime::parse_from_rfc3339(rfc3339) else {
        return rfc3339.to_string();
    };
    let local = utc.with_timezone(&parse_tz(tz_name));
    let fmt = normalise_date_fmt(date_fmt);
    local.format(&format!("{fmt} %H:%M")).to_string()
}

/// Format a UTC RFC-3339 timestamp as just the date part in the user's timezone.
pub fn format_date(rfc3339: &str, tz_name: &str, date_fmt: &str) -> String {
    let Ok(utc) = DateTime::parse_from_rfc3339(rfc3339) else {
        return rfc3339.to_string();
    };
    let local = utc.with_timezone(&parse_tz(tz_name));
    local.format(normalise_date_fmt(date_fmt)).to_string()
}

/// Convert a UTC RFC-3339 string to the `YYYY-MM-DDTHH:MM` value expected by
/// `<input type="datetime-local">`, expressed in the user's timezone.
pub fn to_input(rfc3339: &str, tz_name: &str) -> String {
    let Ok(utc) = DateTime::parse_from_rfc3339(rfc3339) else {
        // Graceful fallback: strip offset, keep naive part.
        return rfc3339.get(..16).unwrap_or(rfc3339).to_string();
    };
    let local = utc.with_timezone(&parse_tz(tz_name));
    local.format("%Y-%m-%dT%H:%M").to_string()
}

/// Convert a `datetime-local` input value (`YYYY-MM-DDTHH:MM` or with seconds),
/// **interpreted in the user's timezone**, back to UTC RFC-3339 for server submission.
///
/// Falls back to appending `Z` (UTC) when the timezone conversion is ambiguous
/// (DST gap) or the string cannot be parsed at all.
pub fn from_input(local_str: &str, tz_name: &str) -> String {
    let tz = parse_tz(tz_name);
    let naive = NaiveDateTime::parse_from_str(local_str, "%Y-%m-%dT%H:%M:%S")
        .or_else(|_| NaiveDateTime::parse_from_str(local_str, "%Y-%m-%dT%H:%M"));

    let Ok(ndt) = naive else {
        // Unknown format — return as-is; server's `parse_datetime_utc` will handle it.
        return local_str.to_string();
    };

    match ndt.and_local_timezone(tz) {
        LocalResult::Single(dt) | LocalResult::Ambiguous(dt, _) => {
            dt.with_timezone(&Utc).to_rfc3339()
        }
        // DST gap: the wall-clock time doesn't exist in this timezone. Treat as UTC.
        LocalResult::None => DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc).to_rfc3339(),
    }
}

/// Format a monetary amount stored in **cents** as a human-readable string
/// using the workspace currency code (ISO 4217).
pub fn format_money(cents: i64, currency: &str) -> String {
    let symbol = currency_symbol(currency);
    let amount = cents as f64 / 100.0;
    if symbol.is_empty() {
        format!("{amount:.2} {currency}")
    } else {
        format!("{symbol}{amount:.2}")
    }
}

/// Map the most common ISO 4217 codes to their Unicode symbols.
/// Returns an empty string for unknown codes so callers can fall back to the code.
fn currency_symbol(code: &str) -> &'static str {
    match code {
        "USD" => "$",
        "EUR" => "€",
        "GBP" => "£",
        "JPY" => "¥",
        "CHF" => "Fr",
        "CAD" => "CA$",
        "AUD" => "A$",
        "NOK" | "SEK" | "DKK" => "kr",
        _ => "",
    }
}
