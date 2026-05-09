use dioxus::prelude::*;

/// Alignment for a table column / cell.
#[derive(Clone, PartialEq, Default, Debug)]
pub enum ColAlign {
    #[default]
    Left,
    Center,
    Right,
}

impl ColAlign {
    fn class(&self) -> &'static str {
        match self {
            ColAlign::Left => "text-left",
            ColAlign::Center => "text-center",
            ColAlign::Right => "text-right",
        }
    }
}

/// Column header definition passed to [`DataTable`].
#[derive(Clone, PartialEq, Debug)]
pub struct ColumnDef {
    pub label: String,
    pub align: ColAlign,
    /// Optional fixed width (e.g. `"120px"`, `"15%"`).  `None` → auto.
    pub width: Option<String>,
}

impl ColumnDef {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            align: ColAlign::Left,
            width: None,
        }
    }

    pub fn right(mut self) -> Self {
        self.align = ColAlign::Right;
        self
    }

    pub fn center(mut self) -> Self {
        self.align = ColAlign::Center;
        self
    }

    pub fn width(mut self, w: impl Into<String>) -> Self {
        self.width = Some(w.into());
        self
    }
}

// ── DataTable ─────────────────────────────────────────────────────────────────

/// A generic data-table shell.
///
/// - `columns`        — header definitions (label, align, optional width)
/// - `total`          — total number of records (used for pagination)
/// - `page`           — current 0-based page index
/// - `page_size`      — records per page
/// - `loading`        — when `true` the body is replaced by skeleton rows
/// - `on_page_change` — fires with the new 0-based page index when the user
///                      clicks a pagination button; parent owns the Signal
/// - `children`       — `<TableRow>` / expand-row elements
///
/// Client-side example
/// ```rust,ignore
/// let mut page = use_signal(|| 0_usize);
/// let page_size = 10_usize;
/// let visible: Vec<_> = items.read()
///     .iter()
///     .skip(page * page_size)
///     .take(page_size)
///     .cloned()
///     .collect();
///
/// DataTable {
///     columns: vec![ColumnDef::new("Name"), ColumnDef::new("Status").right()],
///     total: items.read().len(),
///     page: *page.read(),
///     page_size,
///     on_page_change: move |p| page.set(p),
///     for item in visible { TableRow { TableCell { "{item.name}" } } }
/// }
/// ```
#[component]
pub fn DataTable(
    columns: Vec<ColumnDef>,
    total: usize,
    page: usize,
    page_size: usize,
    #[props(default)] loading: bool,
    on_page_change: EventHandler<usize>,
    children: Element,
) -> Element {
    let col_count = columns.len();
    let total_pages = if page_size == 0 {
        1
    } else {
        total.div_ceil(page_size).max(1)
    };
    let has_prev = page > 0;
    let has_next = page + 1 < total_pages;
    let showing_start = if total == 0 { 0 } else { page * page_size + 1 };
    let showing_end = ((page + 1) * page_size).min(total);

    // Build page-number list with ellipsis (max 7 slots).
    let page_numbers: Vec<Option<usize>> = build_page_numbers(page, total_pages);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        div { class: "dt-wrapper",
            div { class: "dt-scroll",
                table { class: "dt-table",
                    colgroup {
                        for col in &columns {
                            col {
                                style: col.width.as_deref().map(|w| format!("width:{w}")).unwrap_or_default(),
                            }
                        }
                    }
                    thead {
                        tr {
                            for col in &columns {
                                th { class: "dt-th {col.align.class()}", "{col.label}" }
                            }
                        }
                    }
                    tbody {
                        if loading {
                            for _ in 0..page_size.min(5) {
                                tr { class: "dt-row",
                                    td { colspan: col_count as i64, class: "dt-td",
                                        div { class: "dt-skeleton" }
                                    }
                                }
                            }
                        } else {
                            {children}
                        }
                        if !loading && total == 0 {
                            tr { class: "dt-row",
                                td { colspan: col_count as i64, class: "dt-td dt-empty",
                                    "No records found."
                                }
                            }
                        }
                    }
                }
            }
            // ── Pagination footer ──────────────────────────────────────────
            if total_pages > 1 {
                div { class: "dt-footer",
                    span { class: "dt-count",
                        "{showing_start}–{showing_end} of {total}"
                    }
                    div { class: "dt-pager",
                        button {
                            class: "dt-page-btn",
                            disabled: !has_prev,
                            onclick: move |_| on_page_change.call(page - 1),
                            "‹"
                        }
                        for slot in &page_numbers {
                            match slot {
                                None => rsx! {
                                    span { class: "dt-page-ellipsis", "…" }
                                },
                                Some(p) => {
                                    let p = *p;
                                    let current = p == page;
                                    rsx! {
                                        button {
                                            key: "{p}",
                                            class: if current { "dt-page-btn dt-page-btn--active" } else { "dt-page-btn" },
                                            onclick: move |_| on_page_change.call(p),
                                            "{p + 1}"
                                        }
                                    }
                                }
                            }
                        }
                        button {
                            class: "dt-page-btn",
                            disabled: !has_next,
                            onclick: move |_| on_page_change.call(page + 1),
                            "›"
                        }
                    }
                }
            }
        }
    }
}

// ── TableRow ──────────────────────────────────────────────────────────────────

/// A `<tr>` row inside a [`DataTable`] body.
#[component]
pub fn TableRow(
    #[props(default)] muted: bool,
    #[props(default)] expanded: bool,
    children: Element,
) -> Element {
    let cls = match (muted, expanded) {
        (_, true) => "dt-row dt-row--expanded",
        (true, _) => "dt-row dt-row--muted",
        _ => "dt-row",
    };
    rsx! {
        tr { class: cls, {children} }
    }
}

// ── TableCell ─────────────────────────────────────────────────────────────────

/// A `<td>` cell inside a [`TableRow`].
#[component]
pub fn TableCell(
    #[props(default)] align: ColAlign,
    #[props(default)] mono: bool,
    #[props(default)] colspan: Option<i64>,
    children: Element,
) -> Element {
    let mut cls = format!("dt-td {}", align.class());
    if mono {
        cls.push_str(" font-mono text-xs");
    }
    rsx! {
        td {
            class: cls,
            colspan: colspan,
            {children}
        }
    }
}

// ── TableExpandRow ────────────────────────────────────────────────────────────

/// A full-width expand row used for inline edit forms.
/// Renders as a `<tr><td colspan=N>…</td></tr>` with a distinct background.
#[component]
pub fn TableExpandRow(col_count: usize, children: Element) -> Element {
    rsx! {
        tr { class: "dt-expand-row",
            td { colspan: col_count as i64, class: "dt-expand-cell",
                {children}
            }
        }
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn build_page_numbers(current: usize, total: usize) -> Vec<Option<usize>> {
    if total <= 7 {
        return (0..total).map(Some).collect();
    }
    let mut pages: Vec<Option<usize>> = Vec::new();
    // Always show first and last; add ellipsis where there are gaps.
    let window = 2_usize; // pages around current to always show
    let mut prev = None::<usize>;
    for p in 0..total {
        let show = p == 0
            || p == total - 1
            || p.abs_diff(current) <= window;
        if show {
            if let Some(prev_p) = prev {
                if p > prev_p + 1 {
                    pages.push(None); // ellipsis
                }
            }
            pages.push(Some(p));
            prev = Some(p);
        }
    }
    pages
}
