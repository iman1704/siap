//! siap — opinionated taskwarrior TUI.
//!
//! E-hybrid UI styled from `wargames-tokyonight-color-palette.md` and
//! shaped by `openspec/changes/siap-v2` plus `spec-update.md`:
//!
//! - A top tab bar replaces the sidebar: Inbox / Today / All plus one tab
//!   per project, each with a live count, plus open/due/active/sync totals.
//!   Search lives on `/` instead of a tab.
//! - Two-line task rows carry a priority dot, description, and urgency
//!   on the first line, then due/project/tags/recur/blocked-by on the second.
//! - Active tasks keep the mint ◉ start indicator; completed work sits dimmed
//!   at the bottom behind a rule.
//! - The list header stays fixed while task rows scroll; `Enter` opens a
//!   split inspector, read-only in `VIEW`; `e` edits one field. Narrow
//!   terminals use a full-body overlay instead.
//!
//! ```sh
//! cargo run            # the UI (live taskwarrior reads)
//! cargo run -- --help  # keys
//! ```

#[allow(dead_code)]
mod backend;

use std::cell::Cell;
use std::collections::{HashMap, HashSet};
use std::io::{self, stdout};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use chrono::{DateTime, FixedOffset, Local, NaiveDate, NaiveDateTime, TimeZone, Utc};

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use nucleo_matcher::pattern::{
    CaseMatching as NucleoCase, Normalization as NucleoNorm, Pattern as NucleoPattern,
};
use nucleo_matcher::{
    Config as NucleoConfig, Matcher as NucleoMatcher, Utf32Str as NucleoUtf32Str,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Terminal,
};

// ---------------------------------------------------------------------------
// theme — Tokyo Night // WOPR (WarGames Edition)
// ---------------------------------------------------------------------------

/// `#0a0f1e` Editor Background (Night)
const BG: Color = Color::Rgb(10, 15, 30);
/// `#0d1424` Slightly lifted panel/footer surface.
const PANEL: Color = Color::Rgb(13, 20, 36);
/// `#16213c` Selection tint for the cursor row, focused input, and inspector.
const TINT: Color = Color::Rgb(22, 33, 60);
/// `#1e2742` Hairline borders and rules.
const LINE: Color = Color::Rgb(30, 39, 66);
/// `#a8b6d4` Editor Foreground — primary text
const INK: Color = Color::Rgb(168, 182, 212);
/// `#d6e2ff` Variables — bright text (cursor row, titles)
const BRIGHT: Color = Color::Rgb(214, 226, 255);
/// `#8a9abf` Markdown Text — secondary text
const INK2: Color = Color::Rgb(138, 154, 191);
/// `#3a4a6d` Comments — dividers, faint labels
const DIM: Color = Color::Rgb(58, 74, 109);
/// `#3da9ff` Terminal Blue — cursor bar, focus, links
const BLUE: Color = Color::Rgb(61, 169, 255);
/// `#5ac8ff` Terminal Cyan — tags, headings
const CYAN: Color = Color::Rgb(90, 200, 255);
/// `#5cffc6` Terminal Green — active marker
const MINT: Color = Color::Rgb(92, 255, 198);
/// `#ff5c6c` Terminal Red — H, overdue, destructive
const RED: Color = Color::Rgb(255, 92, 108);
/// `#ffcc66` Terminal Yellow — M, due-today
const YELLOW: Color = Color::Rgb(255, 204, 102);
/// `#ffbd5e` Number constants — urgency value
const ORANGE: Color = Color::Rgb(255, 189, 94);
/// `#ff7a9e` Terminal Magenta — blocked, VISUAL mode
const MAGENTA: Color = Color::Rgb(255, 122, 158);

/// Palette accents available for projects (danger red is excluded).
const PROJECT_PALETTE: [Color; 6] = [BLUE, MINT, YELLOW, MAGENTA, CYAN, ORANGE];

/// Stable preferred palette slot for a project name (FNV-1a over bytes).
fn project_preferred_slot(name: &str) -> usize {
    let mut hash: u64 = 14695981039346656037;
    for b in name.bytes() {
        hash ^= u64::from(b);
        hash = hash.wrapping_mul(1099511628211);
    }
    (hash % PROJECT_PALETTE.len() as u64) as usize
}

/// Assign each sorted project a distinct palette color. The preferred slot
/// comes from a stable hash; collisions walk forward to the next free slot
/// in sorted order so the current set is unique and stable across refreshes.
fn build_project_colors(projects: &[String]) -> HashMap<String, Color> {
    let mut out = HashMap::new();
    let mut used = [false; 6];
    for name in projects {
        let mut slot = project_preferred_slot(name);
        for _ in 0..PROJECT_PALETTE.len() {
            if !used[slot] {
                break;
            }
            slot = (slot + 1) % PROJECT_PALETTE.len();
        }
        used[slot] = true;
        out.insert(name.clone(), PROJECT_PALETTE[slot]);
    }
    out
}

fn prio_label(p: char) -> &'static str {
    match p {
        'H' => "H",
        'M' => "M",
        _ => "L",
    }
}

fn prio_color(label: &str) -> Color {
    match label {
        "H" => RED,
        "M" => YELLOW,
        _ => DIM,
    }
}

// ---------------------------------------------------------------------------
// model — tasks carry the section UDA; the registry is derived from them
// ---------------------------------------------------------------------------

/// One timestamped note attached to a task.
/// `entry` is the raw `task export` timestamp (`YYYYMMDDTHHMMSSZ`);
/// `text` is the annotation description taskwarrior matches on denotate.
#[derive(Clone, Debug, PartialEq)]
struct Annotation {
    entry: String,
    text: String,
}

/// One task. `status` is pending | active | completed | deleted.
/// `uuid` is stable identity; `short_id` is display-only.
/// `depends` holds blocking UUIDs from native `depends` (flat DAG, no hierarchy).
#[derive(Clone)]
struct Task {
    uuid: String,
    // Display-only numeric id from `task export`; identity is `uuid`.
    #[allow(dead_code)]
    short_id: Option<u32>,
    desc: String,
    project: String,
    due: String,
    // Raw ISO `due` from `task export` (`YYYYMMDDTHHMMSSZ`); `None` for
    // legacy mocks that carry only the friendly `due` label.
    due_raw: Option<String>,
    prio: char, // H | M | L
    tags: Vec<String>,
    section: String,
    urgency: f32,
    active: bool,
    status: String,
    depends: Vec<String>,
    recur: String,
    // Hidden recurring-template UUID on a generated occurrence; empty for
    // one-off tasks (the template itself is never loaded).
    recur_parent: String,
    start_end: String,
    // Raw ISO `start` timestamp from `task export` for the relative
    // `start` age label (`1 hour ago`). `None` when never started.
    start_raw: Option<String>,
    // Raw ISO `end` timestamp from `task export` for completed-tail sorting.
    // `None` for open tasks and legacy mocks without export data.
    end: Option<String>,
    annotations: Vec<Annotation>,
}

/// Registry order: Now, Next, Later when present, then custom names A–Z.
/// Matching is case-insensitive so `Now` and `now` are one section.
fn section_key(name: &str) -> (u8, String) {
    let rank = if name.eq_ignore_ascii_case("Now") {
        0
    } else if name.eq_ignore_ascii_case("Next") {
        1
    } else if name.eq_ignore_ascii_case("Later") {
        2
    } else {
        3
    };
    (rank, name.to_lowercase())
}

/// Sort a section name list into registry order and drop case duplicates.
fn sort_sections(names: &mut Vec<String>) {
    names.sort_by(|a, b| section_key(a).cmp(&section_key(b)).then_with(|| a.cmp(b)));
    names.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
}

fn is_open(t: &Task) -> bool {
    t.status == "pending" || t.status == "active"
}

/// Inbox intake: open task with no project. Due date is ignored.
fn is_inbox(t: &Task) -> bool {
    is_open(t) && t.project.is_empty()
}

fn is_overdue(t: &Task) -> bool {
    if let Some(raw) = t.due_raw.as_deref() {
        return due_is_overdue(raw, Local::now());
    }
    // Legacy mocks carry only the friendly label.
    t.due.contains("overdue")
}

fn is_due_today(t: &Task) -> bool {
    if let Some(raw) = t.due_raw.as_deref() {
        return due_is_today(raw, Local::now());
    }
    t.due.starts_with("today")
}

/// Parse a `task export` timestamp (`YYYYMMDDTHHMMSSZ`) as UTC.
fn parse_task_datetime(iso: &str) -> Option<DateTime<Utc>> {
    NaiveDateTime::parse_from_str(iso, "%Y%m%dT%H%M%SZ")
        .ok()
        .map(|naive| Utc.from_utc_datetime(&naive))
}

/// Local calendar date of an export timestamp under an explicit offset.
///
/// Pure for unit testing Kuching (+8h) against UTC edge cases.
#[allow(dead_code)] // live wiring lands in 3.x refresh
fn export_date_in_offset(iso: &str, offset_secs: i32) -> Option<NaiveDate> {
    let utc = parse_task_datetime(iso)?;
    let offset = FixedOffset::east_opt(offset_secs)?;
    Some(utc.with_timezone(&offset).date_naive())
}

fn due_is_overdue(raw: &str, now: DateTime<Local>) -> bool {
    let Some(due_utc) = parse_task_datetime(raw) else {
        return false;
    };
    due_utc.with_timezone(&Local).date_naive() < now.date_naive()
}

fn due_is_today(raw: &str, now: DateTime<Local>) -> bool {
    let Some(due_utc) = parse_task_datetime(raw) else {
        return false;
    };
    due_utc.with_timezone(&Local).date_naive() == now.date_naive()
}

/// Friendly due label from an export timestamp using the local calendar.
///
/// - absent → `—`; overdue → `{n}d overdue`; today → `today HH:MM` (or
///   `today` at midnight); tomorrow → `tomorrow`; within a week →
///   lowercase weekday (`fri`); else `YYYY-MM-DD`.
#[allow(dead_code)] // live wiring lands in 3.x refresh
fn format_due_label(raw: Option<&str>, now: DateTime<Local>) -> String {
    let Some(iso) = raw else {
        return "—".into();
    };
    let Some(due_utc) = parse_task_datetime(iso) else {
        return "—".into();
    };
    let local = due_utc.with_timezone(&Local);
    let today = now.date_naive();
    let date = local.date_naive();
    if date < today {
        let days = (today - date).num_days();
        return format!("{days}d overdue");
    }
    if date == today {
        let (h, m) = (
            local.format("%H").to_string(),
            local.format("%M").to_string(),
        );
        if h == "00" && m == "00" {
            return "today".into();
        }
        return format!("today {h}:{m}");
    }
    if date == today.succ_opt().unwrap_or(today) {
        return "tomorrow".into();
    }
    let delta = (date - today).num_days();
    if (0..7).contains(&delta) {
        return local.format("%a").to_string().to_lowercase();
    }
    date.format("%Y-%m-%d").to_string()
}

/// Relative age of a `task export` `start` timestamp (`15 min ago`).
/// Empty for missing/unparseable input or future clock skew beyond a minute.
fn format_start_ago(iso: &str, now: DateTime<Local>) -> String {
    if iso.trim().is_empty() {
        return String::new();
    }
    let Some(start_utc) = parse_task_datetime(iso.trim()) else {
        return String::new();
    };
    let mins = (now - start_utc.with_timezone(&Local)).num_minutes();
    if mins < 1 {
        return "just now".into();
    }
    if mins < 60 {
        return format!("{mins} min ago");
    }
    let hours = mins / 60;
    if hours < 24 {
        return if hours == 1 {
            "1 hour ago".into()
        } else {
            format!("{hours} hours ago")
        };
    }
    let days = hours / 24;
    if days == 1 {
        "1 day ago".into()
    } else {
        format!("{days} days ago")
    }
}

/// Current time as a `task export` timestamp for mock `start` values.
fn now_export_iso() -> String {
    Utc::now().format("%Y%m%dT%H%M%SZ").to_string()
}

/// `task export` timestamp `mins` minutes in the past (mock `start` ages).
fn mock_start_iso(mins_ago: i64) -> String {
    (Utc::now() - chrono::Duration::minutes(mins_ago))
        .format("%Y%m%dT%H%M%SZ")
        .to_string()
}

/// Popup `start` line: relative age while started, blank otherwise.
fn start_display(t: &Task, now: DateTime<Local>) -> String {
    if let Some(raw) = t.start_raw.as_deref() {
        let label = format_start_ago(raw, now);
        if !label.is_empty() {
            return label;
        }
    }
    if t.active {
        return "active".into();
    }
    String::new()
}

fn mock_uuid(id: u32) -> String {
    format!("mock-uuid-{:03}", id)
}

fn mock_tasks() -> Vec<Task> {
    vec![
        Task {
            uuid: mock_uuid(1),
            short_id: Some(1),
            desc: "Triage inbox pile".into(),
            project: "".into(),
            due: "".into(),
            due_raw: None,
            prio: 'M',
            tags: vec![],
            section: "Now".into(),
            urgency: 5.0,
            active: false,
            status: "pending".into(),
            depends: Vec::new(),
            recur: "none".into(),
            recur_parent: String::new(),
            start_end: "not started".into(),
            start_raw: None,
            end: None,
            annotations: Vec::new(),
        },
        Task {
            uuid: mock_uuid(2),
            short_id: Some(2),
            desc: "Read later link dump".into(),
            project: "".into(),
            due: "".into(),
            due_raw: None,
            prio: 'L',
            tags: vec!["read".into()],
            section: "Next".into(),
            urgency: 2.0,
            active: false,
            status: "pending".into(),
            depends: Vec::new(),
            recur: "none".into(),
            recur_parent: String::new(),
            start_end: "not started".into(),
            start_raw: None,
            end: None,
            annotations: Vec::new(),
        },
        Task {
            uuid: mock_uuid(3),
            short_id: Some(3),
            desc: "Reply to Nadia about contract".into(),
            project: "work".into(),
            due: "today 17:00".into(),
            due_raw: None,
            prio: 'H',
            tags: vec!["legal".into()],
            section: "Now".into(),
            urgency: 11.4,
            active: false,
            status: "pending".into(),
            depends: vec![mock_uuid(12)],
            recur: "none".into(),
            recur_parent: String::new(),
            start_end: "not started".into(),
            start_raw: None,
            end: None,
            annotations: vec![Annotation {
                entry: String::new(),
                text: "redline before 5".into(),
            }],
        },
        Task {
            uuid: mock_uuid(4),
            short_id: Some(4),
            desc: "Draft Q4 planning doc".into(),
            project: "work".into(),
            due: "today".into(),
            due_raw: None,
            prio: 'M',
            tags: vec!["planning".into()],
            section: "Now".into(),
            urgency: 8.2,
            active: true,
            status: "active".into(),
            depends: Vec::new(),
            recur: "none".into(),
            recur_parent: String::new(),
            start_end: "09:20 – now · active".into(),
            start_raw: Some(mock_start_iso(65)),
            end: None,
            annotations: Vec::new(),
        },
        Task {
            uuid: mock_uuid(5),
            short_id: Some(5),
            desc: "Pay domain renewal".into(),
            project: "personal".into(),
            due: "2d overdue".into(),
            due_raw: None,
            prio: 'H',
            tags: vec!["admin".into()],
            section: "Now".into(),
            urgency: 9.5,
            active: false,
            status: "pending".into(),
            depends: Vec::new(),
            recur: "none".into(),
            recur_parent: String::new(),
            start_end: "not started".into(),
            start_raw: None,
            end: None,
            annotations: Vec::new(),
        },
        Task {
            uuid: mock_uuid(6),
            short_id: Some(6),
            desc: "Future spike (started)".into(),
            project: "work".into(),
            due: "fri".into(),
            due_raw: None,
            prio: 'M',
            tags: vec!["eng".into()],
            section: "Next".into(),
            urgency: 7.0,
            active: true,
            status: "active".into(),
            depends: Vec::new(),
            recur: "none".into(),
            recur_parent: String::new(),
            start_end: "10:00 – now · active".into(),
            start_raw: Some(mock_start_iso(15)),
            end: None,
            annotations: Vec::new(),
        },
        Task {
            uuid: mock_uuid(7),
            short_id: Some(7),
            desc: "Book dentist appointment".into(),
            project: "personal".into(),
            due: "tomorrow".into(),
            due_raw: None,
            prio: 'L',
            tags: vec!["health".into()],
            section: "Next".into(),
            urgency: 4.1,
            active: false,
            status: "pending".into(),
            depends: Vec::new(),
            recur: "none".into(),
            recur_parent: String::new(),
            start_end: "not started".into(),
            start_raw: None,
            end: None,
            annotations: Vec::new(),
        },
        Task {
            uuid: mock_uuid(8),
            short_id: Some(8),
            desc: "Fix flaky auth test".into(),
            project: "work".into(),
            due: "fri".into(),
            due_raw: None,
            prio: 'M',
            tags: vec!["bug".into()],
            section: "Next".into(),
            urgency: 7.7,
            active: false,
            status: "pending".into(),
            depends: Vec::new(),
            recur: "none".into(),
            recur_parent: String::new(),
            start_end: "not started".into(),
            start_raw: None,
            end: None,
            annotations: vec![Annotation {
                entry: String::new(),
                text: "fails on main".into(),
            }],
        },
        Task {
            uuid: mock_uuid(9),
            short_id: Some(9),
            desc: "Read Shape Up ch. 4".into(),
            project: "reading".into(),
            due: "—".into(),
            due_raw: None,
            prio: 'L',
            tags: vec![],
            section: "Later".into(),
            urgency: 2.2,
            active: false,
            status: "pending".into(),
            depends: Vec::new(),
            recur: "none".into(),
            recur_parent: String::new(),
            start_end: "not started".into(),
            start_raw: None,
            end: None,
            annotations: Vec::new(),
        },
        Task {
            uuid: mock_uuid(10),
            short_id: Some(10),
            desc: "Migrate staging DB to v15".into(),
            project: "work".into(),
            due: "thu".into(),
            due_raw: None,
            prio: 'M',
            tags: vec!["infra".into()],
            section: "Later".into(),
            urgency: 6.6,
            active: false,
            status: "pending".into(),
            depends: Vec::new(),
            recur: "none".into(),
            recur_parent: String::new(),
            start_end: "not started".into(),
            start_raw: None,
            end: None,
            annotations: vec![Annotation {
                entry: String::new(),
                text: "pg upgrade notes".into(),
            }],
        },
        Task {
            uuid: mock_uuid(11),
            short_id: Some(11),
            desc: "Collect legal redlines".into(),
            project: "work".into(),
            due: "today".into(),
            due_raw: None,
            prio: 'H',
            tags: vec![],
            section: "Now".into(),
            urgency: 9.9,
            active: false,
            status: "completed".into(),
            depends: Vec::new(),
            recur: "none".into(),
            recur_parent: String::new(),
            start_end: "done".into(),
            start_raw: None,
            end: Some("20261101T120000Z".into()),
            annotations: Vec::new(),
        },
        Task {
            uuid: mock_uuid(12),
            short_id: Some(12),
            desc: "Draft cover email".into(),
            project: "work".into(),
            due: "today".into(),
            due_raw: None,
            prio: 'M',
            tags: vec![],
            section: "Now".into(),
            urgency: 7.9,
            active: false,
            status: "pending".into(),
            depends: Vec::new(),
            recur: "none".into(),
            recur_parent: String::new(),
            start_end: "not started".into(),
            start_raw: None,
            end: None,
            annotations: Vec::new(),
        },
        Task {
            uuid: mock_uuid(13),
            short_id: Some(13),
            desc: "Renew passport".into(),
            project: "personal".into(),
            due: "sat".into(),
            due_raw: None,
            prio: 'M',
            tags: vec!["admin".into()],
            section: "Later".into(),
            urgency: 0.0,
            active: false,
            status: "completed".into(),
            depends: Vec::new(),
            recur: "none".into(),
            recur_parent: String::new(),
            start_end: "done 2d ago".into(),
            start_raw: None,
            end: Some("20261110T120000Z".into()),
            annotations: Vec::new(),
        },
        Task {
            uuid: mock_uuid(15),
            short_id: Some(15),
            desc: "Old inbox triage".into(),
            project: "".into(),
            due: "—".into(),
            due_raw: None,
            prio: 'L',
            tags: vec![],
            section: "Later".into(),
            urgency: 0.0,
            active: false,
            status: "completed".into(),
            depends: Vec::new(),
            recur: "none".into(),
            recur_parent: String::new(),
            start_end: "done 3d ago".into(),
            start_raw: None,
            end: Some("20261109T120000Z".into()),
            annotations: Vec::new(),
        },
        Task {
            uuid: mock_uuid(14),
            short_id: Some(14),
            desc: "Plan Someday trip".into(),
            project: "personal".into(),
            due: "—".into(),
            due_raw: None,
            prio: 'L',
            tags: vec!["travel".into()],
            section: "Later".into(),
            urgency: 1.5,
            active: false,
            status: "pending".into(),
            depends: Vec::new(),
            recur: "none".into(),
            recur_parent: String::new(),
            start_end: "not started".into(),
            start_raw: None,
            end: None,
            annotations: Vec::new(),
        },
    ]
}

/// Read a string field from export JSON, treating missing/null as empty.
#[allow(dead_code)] // live wiring lands in 3.x refresh
fn export_str(v: &serde_json::Value, key: &str) -> String {
    v.get(key)
        .and_then(|x| x.as_str())
        .unwrap_or_default()
        .to_string()
}

/// Section value from export JSON with display-without-write fallback.
///
/// A missing or empty `siapsection` means unsectioned: project views render
/// it as a flat list above the section headers, and the value is written
/// only on an explicit move or section edit, never backfilled on read.
/// Dotted keys stay as fixture fallbacks (task 3.5 rejects dotted UDAs).
#[allow(dead_code)] // live wiring lands in 3.x refresh
fn export_section(v: &serde_json::Value) -> String {
    for key in ["siapsection", "siap.section", "uda.siap.section"] {
        if let Some(s) = v.get(key).and_then(|x| x.as_str()) {
            if !s.trim().is_empty() {
                return s.trim().to_string();
            }
        }
    }
    String::new()
}

/// Depends as a UUID list. Export uses an array; accept a comma string too
/// for forward compatibility with older task versions.
#[allow(dead_code)] // live wiring lands in 3.x refresh
fn export_depends(v: &serde_json::Value) -> Vec<String> {
    match v.get("depends") {
        Some(serde_json::Value::Array(items)) => items
            .iter()
            .filter_map(|x| x.as_str().map(String::from))
            .filter(|s| !s.is_empty())
            .collect(),
        Some(serde_json::Value::String(s)) => s
            .split([',', ' '])
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect(),
        _ => Vec::new(),
    }
}

/// Annotations from an export value: one entry per `annotations[]` item
/// with its `entry` timestamp and `description` text. Empty when none.
#[allow(dead_code)] // live wiring lands in 3.x refresh
fn export_annotations(v: &serde_json::Value) -> Vec<Annotation> {
    match v.get("annotations") {
        Some(serde_json::Value::Array(items)) => items
            .iter()
            .filter_map(|a| {
                a.get("description")
                    .and_then(|d| d.as_str())
                    .map(|text| Annotation {
                        entry: a
                            .get("entry")
                            .and_then(|e| e.as_str())
                            .unwrap_or_default()
                            .to_string(),
                        text: text.to_string(),
                    })
            })
            .filter(|a| !a.text.is_empty())
            .collect(),
        _ => Vec::new(),
    }
}

/// Joined annotation texts for compact checks; `—` when none.
#[allow(dead_code)] // shared by unit tests; render reads the Vec directly
fn annotations_text(annotations: &[Annotation]) -> String {
    if annotations.is_empty() {
        return "—".into();
    }
    annotations
        .iter()
        .map(|a| a.text.clone())
        .collect::<Vec<_>>()
        .join("; ")
}

/// One `Task` from an export value. `now` pins the local clock for labels.
#[allow(dead_code)] // live wiring lands in 3.x refresh
fn task_from_value(v: &serde_json::Value, now: DateTime<Local>) -> Task {
    let uuid = export_str(v, "uuid");
    let short_id = v
        .get("id")
        .and_then(|x| x.as_u64())
        .map(|n| n as u32)
        .filter(|n| *n > 0);
    let status = {
        let s = export_str(v, "status");
        if s.is_empty() {
            "pending".into()
        } else {
            s
        }
    };
    let active = v
        .get("start")
        .and_then(|x| x.as_str())
        .is_some_and(|s| !s.is_empty())
        || status == "active";
    let due_raw = v
        .get("due")
        .and_then(|x| x.as_str())
        .filter(|s| !s.is_empty())
        .map(String::from);
    let due = format_due_label(due_raw.as_deref(), now);
    let prio = match v.get("priority").and_then(|x| x.as_str()) {
        Some("H") => 'H',
        Some("M") => 'M',
        _ => 'L', // absent priority renders L
    };
    let tags = match v.get("tags") {
        Some(serde_json::Value::Array(items)) => items
            .iter()
            .filter_map(|x| x.as_str().map(String::from))
            .collect(),
        _ => Vec::new(),
    };
    let urgency = v.get("urgency").and_then(|x| x.as_f64()).unwrap_or(0.0) as f32;
    let section = export_section(v);
    let end = v
        .get("end")
        .and_then(|x| x.as_str())
        .filter(|s| !s.is_empty())
        .map(String::from);
    let recur = {
        let r = export_str(v, "recur");
        if r.is_empty() {
            "none".into()
        } else {
            r
        }
    };
    let start_raw = v
        .get("start")
        .and_then(|x| x.as_str())
        .filter(|s| !s.is_empty())
        .map(String::from);
    let start_end = if status == "completed" {
        "done".into()
    } else if active {
        "started – now · active".into()
    } else {
        "not started".into()
    };
    Task {
        uuid,
        short_id,
        desc: export_str(v, "description"),
        project: export_str(v, "project"),
        due,
        due_raw,
        prio,
        tags,
        section,
        urgency,
        active,
        status,
        depends: export_depends(v),
        recur,
        recur_parent: export_str(v, "parent"),
        start_end,
        start_raw,
        end,
        annotations: export_annotations(v),
    }
}

/// All tasks from an export JSON string. `depends` stays as native
/// blocking UUIDs (flat DAG); `deleted` never renders.
/// Blocked state is derived per task: any `depends` UUID still open.
#[allow(dead_code)] // live wiring lands in 3.x refresh
fn tasks_from_export(json: &str, now: DateTime<Local>) -> Vec<Task> {
    let values: Vec<serde_json::Value> = serde_json::from_str(json).unwrap_or_default();
    let mut tasks: Vec<Task> = values.iter().map(|v| task_from_value(v, now)).collect();
    // Deleted never renders; drop it here so views need no extra guard.
    tasks.retain(|t| t.status != "deleted");
    tasks
}

// ---------------------------------------------------------------------------
// views
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
enum View {
    Search,
    Inbox,
    Today,
    All,
    Project(String),
}

impl View {
    fn title(&self) -> String {
        match self {
            View::Search => "Search".into(),
            View::Inbox => "Inbox".into(),
            View::Today => "Today".into(),
            View::All => "All".into(),
            View::Project(p) => p.clone(),
        }
    }
    fn matches(&self, t: &Task) -> bool {
        match self {
            View::Search => is_open(t),
            View::Inbox => is_inbox(t),
            View::Today => is_open(t) && (is_overdue(t) || is_due_today(t) || t.active),
            View::All => is_open(t),
            View::Project(p) => is_open(t) && &t.project == p,
        }
    }
    fn empty_hint(&self) -> &'static str {
        match self {
            View::Search => "Type to search tasks.",
            View::Inbox => "Nothing in intake. Press a to capture.",
            View::Today => "Nothing due. Enjoy the quiet.",
            View::All => "No pending tasks. Press a to add one.",
            View::Project(_) => "No tasks here. Press a to add one.",
        }
    }
}

// ---------------------------------------------------------------------------
// popup — read-only VIEW by default, e enters EDIT
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Debug)]
enum PopupMode {
    View,
    Edit,
}

/// Draft edits for the popup. Seven editable fields live in one flat list;
/// urgency, status, and start stay read-only, while `annotation`
/// is interactive in VIEW (`a` add, `[`/`]` select, `D` remove) and
/// `BLOCKED BY` / `BLOCKS` close the popup as read-only relation rows.
/// VIEW moves with `j/k`; `e` edits the selected field at once,
/// `Enter` saves back to VIEW, `esc` discards to VIEW.
struct Popup {
    task_id: String,
    field: usize, // 0..6 editable; read-only rows never take focus
    mode: PopupMode,
    suggest_idx: usize,
    /// Text cursor (char offset) inside the focused EDIT field.
    edit_cursor: usize,
    desc: String,
    project: String,
    due: String,
    prio: char,
    tags: String,
    depends: String,
    recur: String,
    /// New-annotation draft typed in popup VIEW (`a` to start).
    annotation_draft: String,
    /// Char caret inside `annotation_draft`.
    annotation_cursor: usize,
    /// Selected existing annotation for `[`/`]` + `D` delete.
    annotation_idx: usize,
    /// True while the `annotation` row owns typing (VIEW sub-mode).
    annotating: bool,
    dirty: bool,
}

/// Field index shared by keyboard, render, and suggestions.
///
/// `status` stays read-only (no field index);
/// the `annotation` block is interactive in VIEW with its own keys;
/// `BLOCKED BY` / `BLOCKS` are derived.
const F_DESC: usize = 0;
const F_PROJECT: usize = 1;
const F_DUE: usize = 2;
const F_PRIO: usize = 3;
const F_TAGS: usize = 4;
const F_DEPENDS: usize = 5;
const F_RECUR: usize = 6;

fn field_label(i: usize) -> &'static str {
    match i {
        F_DESC => "description",
        F_PROJECT => "project",
        F_DUE => "due",
        F_PRIO => "priority",
        F_TAGS => "tags",
        F_DEPENDS => "depends",
        F_RECUR => "recur",
        _ => "?",
    }
}

/// Shared contextual-input language for the add box and the popup editor.
/// Both surfaces render the same grey `e.g.` empty states (except the
/// fixed-list `depends`/`recur` pickers, which take no text), the same explicit
/// clear rows, and the same visible suggestion cap; only the key handling
/// differs (add-box filters vs editor draft text).
const CTX_SUGGEST_CAP: usize = 5;
const CTX_CLEAR_PROJECT: &str = "no project";
const CTX_CLEAR_TAGS: &str = "no tags";
const CTX_CLEAR_DATE: &str = "no date";
const CTX_CLEAR_DEPENDS: &str = "no dependencies";
const CTX_CLEAR_RECUR: &str = "no recurrence";
const CTX_EXAMPLE_PROJECT: &str = "e.g. work";
const CTX_EXAMPLE_TAGS: &str = "e.g. work, urgent";
const CTX_EXAMPLE_DUE: &str = "e.g. 2026-11-20";

/// Fixed recurrence presets offered in both surfaces. `taskwarrior`
/// accepts durations, but the UI stays to these plus an explicit clear.
fn recur_presets() -> Vec<String> {
    vec![
        "daily".into(),
        "weekly".into(),
        "monthly".into(),
        "yearly".into(),
        CTX_CLEAR_RECUR.into(),
    ]
}

/// True when a task carries native recurrence (`recur` set, not `none`).
fn is_recurring(t: &Task) -> bool {
    !backend::is_recur_cleared(&t.recur)
}

/// Taskwarrior's error when a UUID no longer exists. Recurring cleanup
/// treats it as already done so a partly finished detach can retry.
fn is_missing_task_error(err: &str) -> bool {
    err.to_lowercase().contains("no tasks specified")
}

/// Split the open occurrences of one recurring template around `selected`.
/// Earlier occurrences are kept and detached; later ones are deleted with
/// the template. Equal or missing due dates count as later so cleanup never
/// keeps an ambiguous duplicate.
fn split_recurring_siblings(
    tasks: &[Task],
    selected: &Task,
    parent: &str,
) -> (Vec<String>, Vec<String>) {
    let mut previous = Vec::new();
    let mut later = Vec::new();
    for task in tasks {
        if task.uuid == selected.uuid || !is_open(task) || task.recur_parent != parent {
            continue;
        }
        let is_previous = match (task.due_raw.as_deref(), selected.due_raw.as_deref()) {
            (Some(sibling), Some(selected)) => sibling < selected,
            _ => false,
        };
        if is_previous {
            previous.push(task.uuid.clone());
        } else {
            later.push(task.uuid.clone());
        }
    }
    (previous, later)
}

/// Editable due text for the popup: overdue tasks open the raw calendar
/// date (`YYYY-MM-DD`) instead of the friendly `Xd overdue` label, so
/// editing fixes the date rather than the label.
fn editable_due_input(t: &Task) -> String {
    if t.due == "—" {
        return String::new();
    }
    if is_overdue(t) {
        if let Some(raw) = t.due_raw.as_deref() {
            if let Some(utc) = parse_task_datetime(raw) {
                return utc.with_timezone(&Local).format("%Y-%m-%d").to_string();
            }
            return raw.to_string();
        }
        // Legacy mocks carry only the label; start blank so the label
        // itself is never edited as if it were a date.
        return String::new();
    }
    t.due.clone()
}

/// True when any draft field differs from its saved task value.
fn popup_is_dirty(app: &App, p: &Popup, t: &Task) -> bool {
    if p.desc != t.desc {
        return true;
    }
    if p.project != t.project {
        return true;
    }
    if p.due != editable_due_input(t) {
        return true;
    }
    if p.prio != t.prio {
        return true;
    }
    let cur_tags: Vec<String> = p
        .tags
        .split([',', ' '])
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.trim_start_matches(['+', '#']).to_string())
        .collect();
    if cur_tags != t.tags {
        return true;
    }
    // Depends compares as resolved UUID sets; unresolvable text counts as dirty
    // so a bad ID is not silently clean. `status` is read-only and never dirty.
    match parse_depends_uuids(app, &p.depends) {
        Ok(cur) => {
            let mut a = cur;
            let mut b = t.depends.clone();
            a.sort();
            b.sort();
            if a != b {
                return true;
            }
        }
        Err(_) => return true,
    }
    // Recurrence compares normalized: empty and `none` are both clear.
    let cur_recur = p.recur.trim();
    let saved_recur = t.recur.trim();
    if backend::is_recur_cleared(cur_recur) != backend::is_recur_cleared(saved_recur) {
        return true;
    }
    if !backend::is_recur_cleared(cur_recur)
        && cur_recur.to_lowercase() != saved_recur.to_lowercase()
    {
        return true;
    }
    false
}

impl Popup {
    fn open(app: &App, t: &Task) -> Self {
        let due = editable_due_input(t);
        let mut p = Self {
            task_id: t.uuid.clone(),
            field: 0,
            mode: PopupMode::View,
            suggest_idx: 0,
            edit_cursor: 0,
            desc: t.desc.clone(),
            project: t.project.clone(),
            due,
            prio: t.prio,
            tags: t.tags.join(", "),
            depends: depends_display_for(app, t),
            recur: if backend::is_recur_cleared(&t.recur) {
                String::new()
            } else {
                t.recur.clone()
            },
            annotation_draft: String::new(),
            annotation_cursor: 0,
            annotation_idx: 0,
            annotating: false,
            dirty: false,
        };
        p.edit_cursor = p.field_len(p.field);
        p
    }
    fn field_count() -> usize {
        7
    }
    #[allow(dead_code)]
    fn reload(&mut self, app: &App, t: &Task) {
        *self = Self {
            mode: PopupMode::View,
            ..Self::open(app, t)
        };
    }
    fn enter_edit(&mut self) {
        self.mode = PopupMode::Edit;
        self.suggest_idx = 0;
        self.edit_cursor = self.field_len(self.field);
    }
    fn move_field(&mut self, delta: i32) {
        let n = Self::field_count() as i32;
        let next = (self.field as i32 + delta).clamp(0, n - 1);
        if next as usize != self.field {
            self.field = next as usize;
            self.suggest_idx = 0;
        }
        self.edit_cursor = self.field_len(self.field);
    }
    /// Length of a field in chars (cursor clamps to this).
    fn field_len(&self, field: usize) -> usize {
        popup_field_text(self, field).chars().count()
    }
    fn clamp_cursor(&mut self) {
        self.edit_cursor = self.edit_cursor.min(self.field_len(self.field));
    }
    fn move_cursor(&mut self, delta: i32) {
        let len = self.field_len(self.field) as i32;
        self.edit_cursor = (self.edit_cursor as i32 + delta).clamp(0, len) as usize;
    }
}

/// Project cache, already derived from task data and sorted.
fn all_projects(app: &App) -> Vec<String> {
    app.projects.clone()
}

/// Unique tags across all tasks, sorted.
fn all_tags(app: &App) -> Vec<String> {
    let mut set = HashSet::new();
    for t in &app.tasks {
        for g in &t.tags {
            set.insert(g.clone());
        }
    }
    let mut out: Vec<String> = set.into_iter().collect();
    out.sort();
    out
}

fn due_presets() -> Vec<String> {
    vec!["today".into(), "tomorrow".into(), CTX_CLEAR_DATE.into()]
}

/// Last in-progress token of a comma/space-separated tag field.
/// Used for filtering and for Tab/Enter completion so `leet` completes to
/// `leetcode` instead of appending both.
fn tag_last_token(tags: &str) -> String {
    if tags.is_empty() || tags.ends_with(',') || tags.ends_with(' ') {
        return String::new();
    }
    tags.split([',', ' '])
        .rfind(|s| !s.is_empty())
        .unwrap_or_default()
        .to_string()
}

/// Apply a tag suggestion by replacing the in-progress last token.
/// `work, leet` + `leetcode` → `work, leetcode`; `leet` → `leetcode`.
fn apply_tag_pick(current: &str, pick: &str) -> String {
    if current.trim().is_empty() {
        return pick.to_string();
    }
    if current.ends_with(',') || current.ends_with(' ') {
        let base = current.trim_end_matches([',', ' ']);
        if base.is_empty() {
            return pick.to_string();
        }
        return format!("{base}, {pick}");
    }
    // Replace the last token, preserving its separator.
    let bytes = current.as_bytes();
    let mut sep_pos: Option<usize> = None;
    for (i, b) in bytes.iter().enumerate() {
        if *b == b',' || *b == b' ' {
            sep_pos = Some(i);
        }
    }
    match sep_pos {
        None => pick.to_string(),
        Some(pos) => {
            // Include the separator run (`, ` or spaces) in the prefix.
            let mut end = pos + 1;
            while end < current.len()
                && (current.as_bytes()[end] == b',' || current.as_bytes()[end] == b' ')
            {
                // Keep only the last separator shape: normalize to ", " for
                // commas, " " otherwise.
                end += 1;
            }
            let raw_sep = &current[pos..end];
            let sep = if raw_sep.contains(',') { ", " } else { " " };
            let mut prefix = current[..pos].trim_end().to_string();
            if prefix.is_empty() {
                return pick.to_string();
            }
            prefix.push_str(sep);
            prefix.push_str(pick);
            prefix
        }
    }
}

/// Short numeric ID for display, else a UUID prefix.
fn task_id_display(t: &Task) -> String {
    if let Some(id) = t.short_id {
        return id.to_string();
    }
    t.uuid.chars().take(8).collect()
}

/// Parse the Search query into a fuzzy pattern. `None` when blank, so an
/// empty query keeps matching everything.
fn search_pattern(query: &str) -> Option<NucleoPattern> {
    let q = query.trim();
    if q.is_empty() {
        return None;
    }
    Some(NucleoPattern::parse(
        q,
        NucleoCase::Ignore,
        NucleoNorm::Smart,
    ))
}

/// One fuzzy check of a haystack against an already-parsed pattern.
fn fuzzy_match_text(haystack: &str, pattern: &NucleoPattern, matcher: &mut NucleoMatcher) -> bool {
    let mut buf = Vec::new();
    pattern
        .score(NucleoUtf32Str::new(haystack, &mut buf), matcher)
        .is_some()
}

/// ID match for Search: short-ID substring plus UUID prefix, both
/// case-insensitive. Stays exact (not fuzzy) so `1` finds 1/12/15.
fn search_id_match(t: &Task, q: &str) -> bool {
    if q.is_empty() {
        return false;
    }
    if let Some(id) = t.short_id {
        if id.to_string().contains(q) {
            return true;
        }
    }
    t.uuid.to_lowercase().starts_with(q)
}

/// Full Search predicate shared by open rows, the completed tail, and the
/// tab count. Substring stays a fast path; fuzzy covers the rest across
/// description, project, and tags. Sort stays urgency-desc; only the match
/// set widens.
fn task_matches_search_query(
    t: &Task,
    query: &str,
    pattern: &NucleoPattern,
    matcher: &mut NucleoMatcher,
) -> bool {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return true;
    }
    if search_id_match(t, &q) {
        return true;
    }
    if t.desc.to_lowercase().contains(&q) || t.project.to_lowercase().contains(&q) {
        return true;
    }
    if t.tags.iter().any(|g| g.to_lowercase().contains(&q)) {
        return true;
    }
    fuzzy_match_text(&t.desc, pattern, matcher)
        || fuzzy_match_text(&t.project, pattern, matcher)
        || t.tags.iter().any(|g| fuzzy_match_text(g, pattern, matcher))
}

/// Comma-separated short IDs for a task's native `depends` (UUIDs stored).
/// Falls back to an 8-char UUID prefix when no short ID exists. Sorted by
/// display ID for a stable draft.
fn depends_display_for(app: &App, task: &Task) -> String {
    let mut ids: Vec<String> = task
        .depends
        .iter()
        .filter_map(|d| app.task(d))
        .map(task_id_display)
        .collect();
    ids.sort();
    ids.join(", ")
}

/// Raw ID tokens from a depends draft (`12, 14` or `12 14`).
fn parse_depends_tokens(draft: &str) -> Vec<String> {
    draft
        .split([',', ' '])
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

/// Resolve one ID token to a UUID: short-ID exact match first, then a unique
/// UUID prefix / full UUID. Returns `None` for unknown tokens.
fn resolve_depends_token(app: &App, token: &str) -> Option<String> {
    let q = token.trim();
    if q.is_empty() {
        return None;
    }
    if let Some(t) = app
        .tasks
        .iter()
        .find(|t| t.short_id.map(|id| id.to_string() == q).unwrap_or(false))
    {
        return Some(t.uuid.clone());
    }
    let mut hits: Vec<String> = app
        .tasks
        .iter()
        .filter(|t| t.uuid == q || t.uuid.starts_with(q))
        .map(|t| t.uuid.clone())
        .collect();
    hits.sort();
    hits.dedup();
    if hits.len() == 1 {
        return hits.into_iter().next();
    }
    None
}

/// Parse a depends draft to UUIDs, preserving order and dropping duplicates.
/// Returns `Err(token)` for the first unknown ID.
fn parse_depends_uuids(app: &App, draft: &str) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    for tok in parse_depends_tokens(draft) {
        match resolve_depends_token(app, &tok) {
            Some(uuid) => {
                if !out.contains(&uuid) {
                    out.push(uuid);
                }
            }
            None => return Err(tok),
        }
    }
    Ok(out)
}

/// True when linking `task_id` to `new_deps` would close a dependency cycle.
/// Follows existing `depends` edges from each candidate: if `task_id` is
/// reachable, the new link loops back (covers direct, mutual, and longer
/// transitive chains).
fn creates_cycle(tasks: &[Task], task_id: &str, new_deps: &[String]) -> bool {
    for dep in new_deps {
        let mut stack = vec![dep.clone()];
        let mut seen = HashSet::new();
        while let Some(cur) = stack.pop() {
            if cur == task_id {
                return true;
            }
            if !seen.insert(cur.clone()) {
                continue;
            }
            if let Some(t) = tasks.iter().find(|t| t.uuid == cur) {
                stack.extend(t.depends.iter().cloned());
            }
        }
    }
    false
}

/// Dropdown candidates: open tasks excluding self, scoped to the same project
/// (empty matches empty, so no-project tasks never see project tasks).
/// Sorted by short ID, UUID fallback last.
fn depends_candidates<'a>(app: &'a App, self_uuid: &str) -> Vec<&'a Task> {
    let project = app
        .task(self_uuid)
        .map(|t| t.project.clone())
        .unwrap_or_default();
    let mut out: Vec<&Task> = app
        .tasks
        .iter()
        .filter(|t| t.uuid != self_uuid && is_open(t) && t.project == project)
        .collect();
    out.sort_by(|a, b| match (a.short_id, b.short_id) {
        (Some(x), Some(y)) => x.cmp(&y).then_with(|| a.uuid.cmp(&b.uuid)),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => a.uuid.cmp(&b.uuid),
    });
    out
}

/// One dropdown row: `ID  description…`, desc truncated to fit the modal.
fn depends_suggestion_text(t: &Task) -> String {
    format!("{}  {}", task_id_display(t), fit(&t.desc, 36))
}

/// Leading ID token of a `depends` suggestion row (`12  desc…` → `12`).
fn depends_suggestion_id(row: &str) -> String {
    row.split_whitespace()
        .next()
        .unwrap_or_default()
        .to_string()
}

/// Contextual suggestions for the focused EDIT field, filtered by
/// what is already typed. Tags filter by the in-progress last token.
/// Powers the inline dropdown under the input.
fn popup_suggestions(app: &App, p: &Popup) -> Vec<String> {
    if p.field == F_DEPENDS {
        let draft_ids: Vec<String> = parse_depends_tokens(&p.depends);
        let cands = depends_candidates(app, &p.task_id);
        let mut out: Vec<String> = cands.iter().map(|t| depends_suggestion_text(t)).collect();
        // Hide tasks already added so a pick always appends a new link.
        out.retain(|row| !draft_ids.contains(&depends_suggestion_id(row)));
        // Fixed list: no typing, so nothing narrows the candidates.
        // `no dependencies` always closes the list so an explicit clear is
        // one highlight away even from an empty draft, mirroring
        // `no project` / `no tags`.
        out.push(CTX_CLEAR_DEPENDS.into());
        return out;
    }
    if p.field == F_RECUR {
        // Fixed list: presets plus an explicit clear, no filtering.
        return recur_presets();
    }
    let mut out = match p.field {
        F_PROJECT => {
            let mut v = all_projects(app);
            v.insert(0, CTX_CLEAR_PROJECT.into());
            v
        }
        F_TAGS => {
            let mut v = all_tags(app);
            v.insert(0, CTX_CLEAR_TAGS.into());
            v
        }
        F_DUE => due_presets(),
        F_PRIO => vec!["H".into(), "M".into(), "L".into()],
        _ => vec![],
    };
    if p.field == F_TAGS {
        let q = tag_last_token(&p.tags).to_lowercase();
        if !q.is_empty() {
            out.retain(|s| s.to_lowercase().contains(&q));
        }
    } else if p.field == F_PROJECT || p.field == F_DUE {
        let q = popup_field_text(p, p.field).to_lowercase();
        if !q.is_empty() {
            out.retain(|s| s.to_lowercase().contains(&q));
        }
    }
    out.truncate(CTX_SUGGEST_CAP);
    out
}

fn popup_field_text(p: &Popup, field: usize) -> String {
    match field {
        F_DESC => p.desc.clone(),
        F_PROJECT => p.project.clone(),
        F_DUE => p.due.clone(),
        F_PRIO => prio_label(p.prio).to_string(),
        F_TAGS => p.tags.clone(),
        F_DEPENDS => p.depends.clone(),
        F_RECUR => p.recur.clone(),
        _ => String::new(),
    }
}

// ---------------------------------------------------------------------------
// app state
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// add-task box — Todoist-style: description has immediate focus, tag/date
// pickers sit one key away (# tags, ~ due), + opens the rest.
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Debug)]
enum AddFocus {
    Desc,
    Tags,
    Due,
    Project,
    Priority,
    Depends,
    Recur,
    More,
}

#[derive(Clone, Debug)]
struct AddDraft {
    desc: String,
    /// Char caret inside `desc`.
    desc_cursor: usize,
    tags: Vec<String>,
    due: String,
    project: String,
    /// Section stamped on commit when `a` starts from a section context.
    section: String,
    prio: char,
    prio_set: bool,
    depends: Vec<String>,
    recur: String,
    focus: AddFocus,
    filter: String,
    /// Char caret inside the focused picker's `filter`.
    filter_cursor: usize,
    picker: usize,
}

impl AddDraft {
    fn fresh() -> Self {
        Self {
            desc: String::new(),
            desc_cursor: 0,
            tags: vec![],
            due: String::new(),
            project: String::new(),
            section: String::new(),
            prio: 'M',
            prio_set: false,
            depends: vec![],
            recur: String::new(),
            focus: AddFocus::Desc,
            filter: String::new(),
            filter_cursor: 0,
            picker: 0,
        }
    }

    /// Fresh draft prefilled with the section under the cursor, so `a` on a
    /// section header (or a task inside one) creates the task there.
    fn fresh_in_section(section: String) -> Self {
        let mut draft = Self::fresh();
        draft.section = section;
        draft
    }

    /// Clear the picker filter and park its caret at the start.
    fn clear_filter(&mut self) {
        self.filter.clear();
        self.filter_cursor = 0;
    }

    /// Clear the description and park its caret at the start.
    fn clear_desc(&mut self) {
        self.desc.clear();
        self.desc_cursor = 0;
    }
}

/// Inline footer prompt for the generic `A` / `R` / `X` section actions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SectionPromptKind {
    Add,
    Rename,
    Remove,
}

#[derive(Clone, Debug)]
struct SectionPrompt {
    kind: SectionPromptKind,
    buffer: String,
    /// Char caret inside `buffer`.
    cursor: usize,
    /// Section the rename applies to (`Rename` only).
    old_section: Option<String>,
}

/// Footer text before the typed section name, shared by the hint line and
/// the caret placement so the two never drift apart.
fn section_prompt_prefix(p: &SectionPrompt) -> String {
    let action = match p.kind {
        SectionPromptKind::Add => "ADD SECTION",
        SectionPromptKind::Rename => "RENAME SECTION",
        SectionPromptKind::Remove => "REMOVE SECTION",
    };
    match &p.old_section {
        Some(old) => format!("  {action}  {} → ", old.to_uppercase()),
        None => format!("  {action}  "),
    }
}

/// Dropdown candidates for the add-box `depends` picker: open tasks in
/// the same project as the draft (empty matches empty, so no-project
/// drafts never see project tasks). Sorted by short ID, UUID fallback
/// last — the same order as the popup picker.
fn add_depends_candidates<'a>(app: &'a App, project: &str) -> Vec<&'a Task> {
    let mut out: Vec<&Task> = app
        .tasks
        .iter()
        .filter(|t| is_open(t) && t.project == project)
        .collect();
    out.sort_by(|a, b| match (a.short_id, b.short_id) {
        (Some(x), Some(y)) => x.cmp(&y).then_with(|| a.uuid.cmp(&b.uuid)),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => a.uuid.cmp(&b.uuid),
    });
    out
}

/// Unique hint list for whichever add picker currently has focus.
fn add_suggestions(app: &App, draft: &AddDraft) -> Vec<String> {
    if draft.focus == AddFocus::Depends {
        let cands = add_depends_candidates(app, draft.project.as_str());
        let mut out: Vec<String> = cands.iter().map(|t| depends_suggestion_text(t)).collect();
        // Hide tasks already added so a pick always appends a new link.
        out.retain(|row| !draft.depends.contains(&depends_suggestion_id(row)));
        // Fixed list: no typing, so nothing narrows the candidates.
        // The explicit clear row always closes the list, mirroring the
        // popup editor.
        out.push(CTX_CLEAR_DEPENDS.into());
        return out;
    }
    if draft.focus == AddFocus::Recur {
        // Fixed list: presets plus an explicit clear, no filtering.
        return recur_presets();
    }
    let q = draft.filter.to_lowercase();
    let mut out = match draft.focus {
        AddFocus::Tags => {
            let mut v = all_tags(app);
            v.insert(0, CTX_CLEAR_TAGS.into());
            v
        }
        AddFocus::Project => {
            let mut v = all_projects(app);
            v.insert(0, CTX_CLEAR_PROJECT.into());
            v
        }
        AddFocus::Due => due_presets(),
        AddFocus::Priority => vec!["H".into(), "M".into(), "L".into()],
        AddFocus::Depends => unreachable!("handled above"),
        AddFocus::Recur => unreachable!("handled above"),
        AddFocus::More => vec![
            "@ project".into(),
            "! priority".into(),
            "# tags".into(),
            "~ due".into(),
            "& depends".into(),
            "* recur".into(),
        ],
        AddFocus::Desc => vec![],
    };
    if !q.is_empty() && draft.focus != AddFocus::More {
        out.retain(|s| s.to_lowercase().contains(&q));
    }
    out.truncate(CTX_SUGGEST_CAP);
    out
}

/// The focused attribute has a value the footer can advertise removing.
fn add_can_remove(draft: &AddDraft) -> bool {
    match draft.focus {
        AddFocus::Tags => !draft.tags.is_empty(),
        AddFocus::Due => !draft.due.is_empty(),
        AddFocus::Project => !draft.project.is_empty(),
        AddFocus::Priority => draft.prio_set,
        AddFocus::Depends => !draft.depends.is_empty(),
        AddFocus::Recur => !backend::is_recur_cleared(&draft.recur),
        AddFocus::Desc | AddFocus::More => false,
    }
}

/// Add-box pickers that own a text filter (and therefore a caret).
fn add_filter_editable(focus: AddFocus) -> bool {
    matches!(focus, AddFocus::Tags | AddFocus::Due | AddFocus::Project)
}

/// Move the picker highlight with the same clamp in both directions.
fn step_add_picker(app: &mut App, delta: isize) {
    let n = add_suggestions(app, &app.add.clone()).len();
    if n == 0 {
        return;
    }
    let next = (app.add.picker as isize + delta).clamp(0, n as isize - 1);
    app.add.picker = next as usize;
}

/// Move the popup suggestion highlight without touching fields.
/// `Tab`/`Shift+Tab`/`Up`/`Down` share this so the contextual list has one
/// cursor and `Tab` never duplicates a partial tag plus its completion.
fn step_popup_suggest(app: &mut App, delta: isize) {
    let n = match app.popup.as_ref() {
        Some(p) => popup_suggestions(app, p).len(),
        None => 0,
    };
    if n == 0 {
        return;
    }
    if let Some(p) = app.popup.as_mut() {
        if delta < 0 {
            p.suggest_idx = p.suggest_idx.saturating_sub((-delta) as usize);
        } else {
            p.suggest_idx = (p.suggest_idx + delta as usize).min(n - 1);
        }
    }
}

/// Single-level undo snapshot: restores the list after `u`.
#[derive(Clone)]
struct UndoState {
    tasks: Vec<Task>,
    sections: Vec<String>,
    manual_sections: Vec<String>,
}

struct App {
    view: View,
    /// Derived cache: unique non-empty task projects, sorted A–Z.
    projects: Vec<String>,
    /// Deterministic color per cached project, shared by rows, chips, and the inspector.
    project_colors: HashMap<String, Color>,
    tasks: Vec<Task>,
    /// Active section registry: task values plus in-memory manual names.
    sections: Vec<String>,
    /// Names added with `A`; they live for the process until a task carries
    /// the value (no empty-section persistence).
    manual_sections: Vec<String>,
    cursor: usize,
    /// Collapsed section headers (uppercase labels, `COMPLETED` included).
    /// Cursor can land on headers; `Enter` toggles. Survives refresh.
    collapsed: HashSet<String>,
    /// Single-level undo snapshot (tasks + section registries).
    undo_snapshot: Option<UndoState>,
    /// View to return to when leaving Search (`/` opens it as a mode).
    search_origin: Option<View>,
    /// First task-list item currently shown in the scrolling viewport.
    list_offset: Cell<usize>,
    selected: HashSet<String>,
    visual: bool,
    visual_anchor: Option<usize>,
    add_mode: bool,
    add: AddDraft,
    /// Search-view query: typed in Search input mode, cleared on exit.
    search_query: String,
    /// Char caret inside `search_query`.
    search_cursor: usize,
    /// Search-view input focus: true types into the query, false is list mode.
    /// Entering Search via the tab bar starts in input mode; `i` focuses,
    /// `esc` exits back to list mode.
    search_input: bool,
    show_help: bool,
    confirm_delete: Option<Vec<String>>,
    section_prompt: Option<SectionPrompt>,
    popup: Option<Popup>,
    /// Split-view focus: false is the inspector (fields), true is the task
    /// list behind it (`k` focuses the list, `i`/`Enter` focus back in).
    /// Only read while `popup` is open; reset whenever it opens or closes.
    popup_list_focus: bool,
    toast: String,
    /// When the current toast was set; toasts auto-clear after ~2.5s.
    toast_at: Option<Instant>,
    next_id: u32,
    /// Isolated `TASKDATA`/`TASKRC` for tests. `None` inherits process env.
    /// In test builds backend writes run only when set, keeping existing
    /// mock tests offline; production always uses the backend.
    taskdata_dir: Option<PathBuf>,
    taskrc_path: Option<PathBuf>,
    /// Last backend refresh time plus whether it succeeded. The header
    /// renders `synced HH:MM` or `sync failed`.
    last_sync: Option<DateTime<Local>>,
    sync_ok: bool,
}

impl App {
    fn new() -> Self {
        let mut app = Self {
            view: View::Inbox,
            projects: Vec::new(),
            project_colors: HashMap::new(),
            tasks: mock_tasks(),
            sections: Vec::new(),
            manual_sections: Vec::new(),
            cursor: 0,
            collapsed: HashSet::new(),
            undo_snapshot: None,
            search_origin: None,
            list_offset: Cell::new(0),
            selected: HashSet::new(),
            visual: false,
            visual_anchor: None,
            add_mode: false,
            add: AddDraft::fresh(),
            search_query: String::new(),
            search_cursor: 0,
            search_input: false,
            show_help: false,
            confirm_delete: None,
            section_prompt: None,
            popup: None,
            popup_list_focus: false,
            toast: String::new(),
            toast_at: None,
            next_id: 100,
            taskdata_dir: None,
            taskrc_path: None,
            last_sync: None,
            sync_ok: true,
        };
        // Mock tasks seed the offline fallback; live data reseeds on load.
        app.sync_projects();
        app.sync_sections();
        app.cursor_to_first_task();
        app
    }

    /// Project color shared by rows, chips, and the inspector. Cached projects use the
    /// collision-resolved map; unknown names fall back to their preferred
    /// palette slot so drafts still preview a stable color.
    fn project_color(&self, name: &str) -> Color {
        if let Some(c) = self.project_colors.get(name) {
            return *c;
        }
        PROJECT_PALETTE[project_preferred_slot(name)]
    }

    /// Set a toast with a timestamp so it auto-clears after ~2.5s.
    fn set_toast(&mut self, msg: String) {
        self.toast = msg;
        self.toast_at = Some(Instant::now());
    }

    /// Clear any toast immediately.
    fn clear_toast(&mut self) {
        self.toast.clear();
        self.toast_at = None;
    }

    /// Drop the toast once it has been visible for ~2.5s.
    fn expire_toast(&mut self) {
        if let Some(at) = self.toast_at {
            if at.elapsed() > Duration::from_millis(2500) {
                self.clear_toast();
                self.toast_at = None;
            }
        } else if !self.toast.is_empty() {
            // Direct assignments (tests/fixtures) still expire.
            self.toast_at = Some(Instant::now());
        }
    }

    /// Backend writes run in production always; in tests only with an
    /// explicit isolated dir so mock tests never touch live data.
    fn uses_backend(&self) -> bool {
        #[cfg(test)]
        {
            self.taskdata_dir.is_some()
        }
        #[cfg(not(test))]
        {
            let _ = &self.taskdata_dir;
            true
        }
    }

    fn run_backend(&self, args: &[&str]) -> Result<String, String> {
        backend::run_write_with_env(
            args,
            self.taskdata_dir.as_deref(),
            self.taskrc_path.as_deref(),
        )
    }

    /// Register the section UDA without user action; keep the list on failure.
    fn bootstrap_backend(&mut self) {
        if !self.uses_backend() {
            return;
        }
        let args = backend::section_uda_args();
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        if let Err(e) = self.run_backend(&refs) {
            self.set_toast(format!("uda setup failed: {e}"));
        }
    }

    /// Re-export after a successful write, holding the cursor by `uuid`.
    /// Failures toast and preserve list/selection/cursor.
    fn refresh_from_backend(&mut self) {
        if !self.uses_backend() {
            return;
        }
        let hold = self.cursor_id();
        let pending = self.run_backend(&["+PENDING", "export"]);
        let completed = self.run_backend(&["+COMPLETED", "export"]);
        let (pending_json, completed_json) = match (pending, completed) {
            (Ok(p), Ok(c)) => (p, c),
            (Err(e), _) | (_, Err(e)) => {
                self.set_toast(format!("refresh failed: {e}"));
                self.last_sync = Some(Local::now());
                self.sync_ok = false;
                return;
            }
        };
        let now = Local::now();
        // Merge both exports so `depends` links survive the split;
        // cap the completed tail at 10 recent by `end` desc.
        let mut values: Vec<serde_json::Value> =
            serde_json::from_str(&pending_json).unwrap_or_default();
        let mut done: Vec<serde_json::Value> =
            serde_json::from_str(&completed_json).unwrap_or_default();
        done.sort_by(|a, b| {
            let ea = a.get("end").and_then(|x| x.as_str()).unwrap_or("");
            let eb = b.get("end").and_then(|x| x.as_str()).unwrap_or("");
            eb.cmp(ea)
        });
        done.truncate(10);
        values.extend(done);
        let json = serde_json::to_string(&values).unwrap_or_default();
        self.tasks = tasks_from_export(&json, now);
        // Header project tabs follow live task data; stale names leave.
        self.sync_projects();
        // Section headers follow the live task values; manual names survive
        // for the process even when no task carries them.
        self.sync_sections();
        self.last_sync = Some(Local::now());
        self.sync_ok = true;
        if let Some(id) = hold {
            let rows = self.navigable_rows();
            if let Some(idx) = rows.iter().position(|r| match r {
                NavRow::Task(u) | NavRow::CompletedTask(u) => u == &id,
                _ => false,
            }) {
                self.cursor = idx;
                return;
            }
        }
        self.clamp_cursor();
    }

    /// Run backend writes in order, then re-export once on success.
    /// A failed write toasts with stderr and preserves the list,
    /// selection, and cursor (no refresh, no state cleared).
    /// Returns true when the writes and the refresh both succeeded.
    fn backend_commit(
        &mut self,
        batches: &[Vec<String>],
        ok_toast: &str,
        fail_prefix: &str,
    ) -> bool {
        for args in batches {
            let refs: Vec<&str> = args.iter().map(String::as_str).collect();
            if let Err(e) = self.run_backend(&refs) {
                self.set_toast(format!("{fail_prefix}: {e}"));
                return false;
            }
        }
        self.refresh_from_backend();
        if self.toast.starts_with("refresh failed") {
            return false;
        }
        self.set_toast(ok_toast.to_string());
        true
    }

    fn task(&self, uuid: &str) -> Option<&Task> {
        self.tasks.iter().find(|t| t.uuid == uuid)
    }

    /// Tasks this task waits on (`depends`), open only for display.
    fn blocked_by(&self, uuid: &str) -> Vec<&Task> {
        let deps: Vec<String> = self
            .task(uuid)
            .map(|t| t.depends.clone())
            .unwrap_or_default();
        let mut out: Vec<&Task> = deps
            .iter()
            .filter_map(|d| self.task(d))
            .filter(|t| is_open(t))
            .collect();
        out.sort_by(|a, b| a.uuid.cmp(&b.uuid));
        out
    }

    /// Tasks waiting on this task (reverse `depends`), open only for display.
    fn blocking(&self, uuid: &str) -> Vec<&Task> {
        let mut out: Vec<&Task> = self
            .tasks
            .iter()
            .filter(|t| t.depends.iter().any(|d| d == uuid) && is_open(t))
            .collect();
        out.sort_by(|a, b| a.uuid.cmp(&b.uuid));
        out
    }

    /// Open (non-completed) task ids in display order.
    /// Search, Inbox, Today, and All are flat: urgency descending with no
    /// section grouping. Project views group by section, then urgency.
    /// Native `depends` never hides rows; blocked tasks stay visible dimmed.
    fn open_rows(&self) -> Vec<String> {
        let pattern = (self.view == View::Search)
            .then(|| search_pattern(&self.search_query))
            .flatten();
        let mut matcher = NucleoMatcher::new(NucleoConfig::DEFAULT);
        let mut tops: Vec<&Task> = self
            .tasks
            .iter()
            .filter(|t| {
                if !self.view.matches(t) {
                    return false;
                }
                match &pattern {
                    Some(p) => task_matches_search_query(t, &self.search_query, p, &mut matcher),
                    None => true,
                }
            })
            .collect();
        if matches!(
            self.view,
            View::Search | View::Inbox | View::Today | View::All
        ) {
            tops.sort_by(|a, b| {
                b.urgency
                    .partial_cmp(&a.urgency)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| a.uuid.cmp(&b.uuid))
            });
        } else {
            // Unsectioned first, then the registry order, urgency desc within.
            tops.sort_by(|a, b| {
                self.section_rank(&a.section)
                    .cmp(&self.section_rank(&b.section))
                    .then_with(|| a.section.to_lowercase().cmp(&b.section.to_lowercase()))
                    .then_with(|| {
                        b.urgency
                            .partial_cmp(&a.urgency)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .then_with(|| a.uuid.cmp(&b.uuid))
            });
        }
        let mut rows = Vec::new();
        for t in tops {
            rows.push(t.uuid.clone());
        }
        rows
    }

    fn completed_rows(&self) -> Vec<String> {
        // Dimmed COMPLETED tail: recent completions by `end` desc, capped at
        // 10, still filtered by the current view. `deleted` never reaches
        // here (`is_open` + status filter); missing `end` sorts last.
        let pattern = (self.view == View::Search)
            .then(|| search_pattern(&self.search_query))
            .flatten();
        let mut matcher = NucleoMatcher::new(NucleoConfig::DEFAULT);
        let mut done: Vec<&Task> = self
            .tasks
            .iter()
            .filter(|t| {
                if t.status != "completed" {
                    return false;
                }
                let search_ok = match &pattern {
                    Some(p) => task_matches_search_query(t, &self.search_query, p, &mut matcher),
                    None => true,
                };
                search_ok
                    && match &self.view {
                        View::Search => true,
                        View::All => true,
                        View::Project(p) => &t.project == p,
                        View::Inbox => t.project.is_empty(),
                        View::Today => is_overdue(t) || is_due_today(t),
                    }
            })
            .collect();
        done.sort_by(|a, b| b.end.cmp(&a.end).then_with(|| a.uuid.cmp(&b.uuid)));
        done.truncate(10);
        done.iter().map(|t| t.uuid.clone()).collect()
    }
}

/// Navigable list rows: section headers plus task rows, then the
/// COMPLETED header plus completed rows. Headers are cursor stops;
/// `Enter` on one toggles collapse. Collapsed groups hide their rows.
#[derive(Clone, PartialEq)]
enum NavRow {
    SectionHeader(String),
    Task(String),
    CompletedHeader,
    CompletedTask(String),
}

impl App {
    fn clamp_cursor(&mut self) {
        let n = self.navigable_rows().len();
        if n == 0 {
            self.cursor = 0;
        } else {
            self.cursor = self.cursor.min(n - 1);
        }
    }

    fn cursor_id(&self) -> Option<String> {
        match self.navigable_rows().get(self.cursor) {
            Some(NavRow::Task(id)) | Some(NavRow::CompletedTask(id)) => Some(id.clone()),
            _ => None,
        }
    }

    fn cursor_nav(&self) -> Option<NavRow> {
        self.navigable_rows().get(self.cursor).cloned()
    }

    fn is_collapsed(&self, key: &str) -> bool {
        self.collapsed.contains(key)
    }

    fn toggle_collapsed(&mut self, key: &str) {
        if !self.collapsed.remove(key) {
            self.collapsed.insert(key.to_string());
        }
    }

    /// Visible rows in display order, honoring `collapsed`.
    /// Search, Inbox, Today, and All stay flat; only project views render
    /// section headers. The COMPLETED tail stays collapsible everywhere.
    fn navigable_rows(&self) -> Vec<NavRow> {
        let ids = self.open_rows();
        let mut rows = Vec::new();
        if matches!(
            self.view,
            View::Search | View::Inbox | View::Today | View::All
        ) {
            for id in &ids {
                rows.push(NavRow::Task(id.clone()));
            }
        } else {
            // Project views: unsectioned tasks render as a flat list with no
            // header on top; sections appear below it once added.
            for (idx, id) in ids.iter().enumerate() {
                let unsectioned = self.task(id).is_some_and(|t| t.section.trim().is_empty());
                if unsectioned {
                    // `ids` is already urgency-desc within the unsectioned rank.
                    let _ = idx;
                    rows.push(NavRow::Task(id.clone()));
                }
            }
            for (label, idxs) in section_groups(self, &ids) {
                rows.push(NavRow::SectionHeader(label.clone()));
                if !self.is_collapsed(&label) {
                    for i in idxs {
                        if let Some(id) = ids.get(i) {
                            rows.push(NavRow::Task(id.clone()));
                        }
                    }
                }
            }
        }
        let done = self.completed_rows();
        if !done.is_empty() {
            rows.push(NavRow::CompletedHeader);
            if !self.is_collapsed("COMPLETED") {
                for id in done {
                    rows.push(NavRow::CompletedTask(id));
                }
            }
        }
        rows
    }

    /// After structural changes, prefer a task row over a header.
    fn cursor_to_first_task(&mut self) {
        let rows = self.navigable_rows();
        if let Some(pos) = rows
            .iter()
            .position(|r| matches!(r, NavRow::Task(_) | NavRow::CompletedTask(_)))
        {
            // Keep the cursor when already on a task; otherwise land on the
            // first task so headers never steal the initial position.
            match rows.get(self.cursor) {
                Some(NavRow::Task(_)) | Some(NavRow::CompletedTask(_)) => {}
                _ => self.cursor = pos,
            }
        }
        self.clamp_cursor();
    }

    fn push_undo(&mut self) {
        self.undo_snapshot = Some(UndoState {
            tasks: self.tasks.clone(),
            sections: self.sections.clone(),
            manual_sections: self.manual_sections.clone(),
        });
    }

    fn undo(&mut self) {
        if self.uses_backend() {
            let refs = ["rc.confirmation=no", "undo"];
            match self.run_backend(&refs) {
                Ok(_) => {
                    self.undo_snapshot = None;
                    self.refresh_from_backend();
                    if !self.toast.starts_with("refresh failed") {
                        self.set_toast("undone".into());
                    }
                    self.cursor_to_first_task();
                }
                Err(e) => {
                    self.set_toast(format!("undo failed: {e}"));
                }
            }
            return;
        }
        if let Some(snap) = self.undo_snapshot.take() {
            self.tasks = snap.tasks;
            self.sections = snap.sections;
            self.manual_sections = snap.manual_sections;
            self.selected.clear();
            self.visual = false;
            self.sync_projects();
            // `sync_sections` rebuilds from tasks + manual names.
            self.sync_sections();
            self.cursor_to_first_task();
            self.set_toast("undone".into());
        } else {
            self.set_toast("nothing to undo".into());
        }
    }

    fn count_for(&self, view: &View) -> usize {
        if *view == View::Search {
            let Some(pattern) = search_pattern(&self.search_query) else {
                return self.tasks.iter().filter(|t| view.matches(t)).count();
            };
            let mut matcher = NucleoMatcher::new(NucleoConfig::DEFAULT);
            return self
                .tasks
                .iter()
                .filter(|t| {
                    view.matches(t)
                        && task_matches_search_query(t, &self.search_query, &pattern, &mut matcher)
                })
                .count();
        }
        self.tasks.iter().filter(|t| view.matches(t)).count()
    }

    fn header_stats(&self) -> (usize, usize, usize) {
        let rows = self.open_rows();
        let open = rows.len();
        let today = rows
            .iter()
            .filter_map(|id| self.task(id))
            .filter(|t| is_due_today(t) || is_overdue(t))
            .count();
        let active = rows
            .iter()
            .filter_map(|id| self.task(id))
            .filter(|t| t.active)
            .count();
        (open, today, active)
    }

    fn is_blocked(&self, uuid: &str) -> bool {
        let Some(t) = self.task(uuid) else {
            return false;
        };
        t.depends.iter().any(|d| self.task(d).is_some_and(is_open))
    }

    // -- navigation ---------------------------------------------------------
    fn move_up(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
        self.extend_visual();
    }
    fn move_down(&mut self) {
        let n = self.navigable_rows().len();
        if n > 0 {
            self.cursor = (self.cursor + 1).min(n - 1);
            self.extend_visual();
        }
    }

    /// Switch to an explicit view. Leaving Search clears its query so the
    /// next visit starts clean; entering Search always lands in input mode.
    fn switch_to_view(&mut self, next: View) {
        if self.view == View::Search && next != View::Search {
            self.search_query.clear();
            self.search_cursor = 0;
            self.search_input = false;
            self.search_origin = None;
        }
        self.view = next;
        if self.view == View::Search {
            self.search_input = true;
        }
        self.cursor = 0;
        self.list_offset.set(0);
        self.selected.clear();
        self.visual = false;
        self.cursor_to_first_task();
        self.follow_cursor_with_popup();
    }

    /// Re-seat an open inspector on the cursor task. View switches park the
    /// cursor in the new view, so a clean VIEW preview follows it instead of
    /// going stale; an empty view (no task rows at all) closes the inspector.
    /// Headers (including COMPLETED) keep the current preview so it re-seats
    /// as soon as the cursor returns to a task row.
    /// Dirty, editing, and annotating drafts are never touched.
    fn follow_cursor_with_popup(&mut self) {
        let clean = self
            .popup
            .as_ref()
            .is_some_and(|p| p.mode == PopupMode::View && !p.dirty && !p.annotating);
        if !clean {
            return;
        }
        if let Some(t) = self.cursor_id().and_then(|id| self.task(&id).cloned()) {
            self.popup = Some(Popup::open(self, &t));
        } else if self
            .navigable_rows()
            .iter()
            .all(|r| !matches!(r, NavRow::Task(_) | NavRow::CompletedTask(_)))
        {
            self.popup = None;
            self.popup_list_focus = false;
        }
        // Otherwise (cursor on a header): keep the preview open so moving
        // back to a task row re-seats it.
    }

    /// `/` opens Search as a mode over the current view, remembering where
    /// to return on `esc`.
    fn open_search(&mut self) {
        if self.view != View::Search {
            self.search_origin = Some(self.view.clone());
        }
        self.switch_to_view(View::Search);
    }

    /// Leave Search back to the view it was opened from (Inbox if that
    /// project disappeared while searching).
    fn leave_search(&mut self) {
        let origin = self.search_origin.take().unwrap_or(View::Inbox);
        let target = match origin {
            View::Project(name) if self.projects.iter().any(|p| p == &name) => View::Project(name),
            View::Project(_) | View::Search => View::Inbox,
            other => other,
        };
        self.switch_to_view(target);
    }

    /// Tabs in header order: Inbox, Today, All, then projects A–Z. Search
    /// is intentionally absent; it is only reachable with `/`.
    fn tab_views(&self) -> Vec<View> {
        let mut views = vec![View::Inbox, View::Today, View::All];
        views.extend(self.projects.iter().cloned().map(View::Project));
        views
    }

    /// Cycle the header tabs, including every project. No wrap-around:
    /// moving past either end stays put. Search is a mode,
    /// so leaving it with Tab returns to its origin view.
    fn cycle_view(&mut self, delta: isize) {
        if self.view == View::Search {
            self.leave_search();
            return;
        }
        let tabs = self.tab_views();
        let current = tabs.iter().position(|v| *v == self.view).unwrap_or(0) as isize;
        let next = (current + delta).clamp(0, tabs.len() as isize - 1) as usize;
        self.switch_to_view(tabs[next].clone());
    }

    /// Number-key shortcuts: `0` is Search (for `/`), `1`–`3` are the
    /// permanent views, and `4`+ pick a project tab.
    fn switch_view(&mut self, n: usize) {
        match n {
            0 => self.open_search(),
            1 => self.switch_to_view(View::Inbox),
            2 => self.switch_to_view(View::Today),
            3 => self.switch_to_view(View::All),
            i => {
                if let Some(project) = self.projects.get(i - 4).cloned() {
                    self.switch_to_view(View::Project(project));
                }
            }
        }
    }

    // -- selection ----------------------------------------------------------
    fn toggle_select(&mut self, uuid: String) {
        if !self.selected.remove(&uuid) {
            self.selected.insert(uuid);
        }
    }
    fn enter_visual(&mut self) {
        self.visual = true;
        self.visual_anchor = Some(self.cursor);
        if let Some(id) = self.cursor_id() {
            self.selected.insert(id);
        }
    }
    fn exit_visual(&mut self, keep: bool) {
        self.visual = false;
        self.visual_anchor = None;
        if !keep {
            self.selected.clear();
        }
    }
    fn extend_visual(&mut self) {
        if !self.visual {
            return;
        }
        let anchor = self.visual_anchor.unwrap_or(self.cursor);
        let (lo, hi) = if anchor <= self.cursor {
            (anchor, self.cursor)
        } else {
            (self.cursor, anchor)
        };
        let rows = self.navigable_rows();
        self.selected.clear();
        for i in lo..=hi {
            match rows.get(i) {
                Some(NavRow::Task(id)) | Some(NavRow::CompletedTask(id)) => {
                    self.selected.insert(id.clone());
                }
                _ => {}
            }
        }
    }

    // -- operations ---------------------------------------------------------
    fn single_start_stop(&mut self) {
        let Some(id) = self.cursor_id() else {
            return;
        };
        self.push_undo();
        if self.uses_backend() {
            let active = self.task(&id).is_some_and(|t| t.active);
            let args = if active {
                backend::build_stop_args(&id)
            } else {
                backend::build_start_args(&id)
            };
            self.backend_commit(&[args], "toggled start/stop", "start/stop failed");
            return;
        }
        if let Some(t) = self.tasks.iter_mut().find(|t| t.uuid == id) {
            t.active = !t.active;
            t.status = if t.active {
                "active".into()
            } else {
                "pending".into()
            };
            t.start_end = if t.active {
                "started – now · active".into()
            } else {
                "not started".into()
            };
            t.start_raw = if t.active {
                Some(now_export_iso())
            } else {
                None
            };
        }
        self.set_toast("toggled start/stop".into());
    }
    fn single_done(&mut self) {
        if let Some(id) = self.cursor_id() {
            self.toggle_ids(&[id]);
        }
    }
    /// `d` toggles done/undone: open tasks complete, completed tasks reopen
    /// via `task <uuid> modify status:pending`. Works on the selection or
    /// the cursor row (open and COMPLETED rows alike).
    fn toggle_ids(&mut self, ids: &[String]) {
        if ids.is_empty() {
            return;
        }
        let (mut to_done, mut to_open) = (Vec::new(), Vec::new());
        for id in ids {
            match self.task(id).map(|t| t.status.clone()) {
                Some(s) if s == "completed" => to_open.push(id.clone()),
                Some(_) => to_done.push(id.clone()),
                None => {}
            }
        }
        if to_done.is_empty() && to_open.is_empty() {
            return;
        }
        self.push_undo();
        // Warn when completing blocked tasks (open depends remain), but
        // still complete: taskwarrior has no blocking enforcement.
        let blocked_n = to_done.iter().filter(|id| self.is_blocked(id)).count();
        let blocked_suffix = if blocked_n > 0 {
            format!(" · {blocked_n} blocked!")
        } else {
            String::new()
        };
        if self.uses_backend() {
            let mut batches: Vec<Vec<String>> = Vec::new();
            for id in &to_done {
                batches.push(backend::build_done_args(id));
            }
            for id in &to_open {
                batches.push(backend::build_undone_args(id));
            }
            let toast = if to_open.is_empty() {
                format!("completed {} task(s){blocked_suffix}", to_done.len())
            } else if to_done.is_empty() {
                format!("reopened {} task(s)", to_open.len())
            } else {
                format!("toggled {} task(s){blocked_suffix}", ids.len())
            };
            if self.backend_commit(&batches, &toast, "done failed") {
                self.selected.clear();
                self.visual = false;
            }
            return;
        }
        for id in &to_done {
            if let Some(t) = self
                .tasks
                .iter_mut()
                .find(|t| t.uuid.as_str() == id.as_str())
            {
                t.status = "completed".into();
                t.active = false;
            }
        }
        for id in &to_open {
            if let Some(t) = self
                .tasks
                .iter_mut()
                .find(|t| t.uuid.as_str() == id.as_str())
            {
                t.status = "pending".into();
                t.active = false;
            }
        }
        self.selected.clear();
        self.visual = false;
        self.clamp_cursor();
        let msg = if to_open.is_empty() {
            format!("completed {} task(s){blocked_suffix}", to_done.len())
        } else if to_done.is_empty() {
            format!("reopened {} task(s)", to_open.len())
        } else {
            format!("toggled {} task(s){blocked_suffix}", ids.len())
        };
        self.set_toast(msg);
    }
    #[allow(dead_code)] // tests drive bulk-done directly; UI toggles via `toggle_ids`
    fn complete_ids(&mut self, ids: &[String]) {
        if ids.is_empty() {
            return;
        }
        self.push_undo();
        if self.uses_backend() {
            let batches: Vec<Vec<String>> =
                ids.iter().map(|id| backend::build_done_args(id)).collect();
            if self.backend_commit(
                &batches,
                &format!("completed {} task(s)", ids.len()),
                "done failed",
            ) {
                self.selected.clear();
                self.visual = false;
            }
            return;
        }
        for id in ids {
            if let Some(t) = self
                .tasks
                .iter_mut()
                .find(|t| t.uuid.as_str() == id.as_str())
            {
                t.status = "completed".into();
                t.active = false;
            }
        }
        self.selected.clear();
        self.visual = false;
        self.clamp_cursor();
        self.set_toast(format!("completed {} task(s)", ids.len()));
    }
    fn request_delete(&mut self) {
        if !self.selected.is_empty() {
            let ids: Vec<String> = self.selected.iter().cloned().collect();
            self.confirm_delete = Some(ids);
        } else if let Some(id) = self.cursor_id() {
            self.confirm_delete = Some(vec![id]);
        }
    }
    fn confirm_delete_yes(&mut self) {
        if let Some(ids) = self.confirm_delete.take() {
            self.push_undo();
            if self.uses_backend() {
                // Native `depends`: single delete only, no cascade.
                // Taskwarrior strips the UUID from other tasks' `depends`.
                let batches: Vec<Vec<String>> = ids
                    .iter()
                    .map(|uuid| backend::build_delete_args(uuid))
                    .collect();
                if self.backend_commit(
                    &batches,
                    &format!("deleted {} task(s)", ids.len()),
                    "delete failed",
                ) {
                    self.selected.clear();
                    self.visual = false;
                }
                return;
            }
            for id in &ids {
                if let Some(t) = self
                    .tasks
                    .iter_mut()
                    .find(|t| t.uuid.as_str() == id.as_str())
                {
                    t.status = "deleted".into();
                    t.active = false;
                }
            }
            self.selected.clear();
            self.visual = false;
            self.clamp_cursor();
            self.set_toast(format!("deleted {} task(s)", ids.len()));
        }
    }
    fn bulk_move_project(&mut self) {
        // Mock project picker: work -> personal -> reading -> work.
        let ids: Vec<String> = self.selected.iter().cloned().collect();
        if ids.is_empty() {
            return;
        }
        self.push_undo();
        if self.uses_backend() {
            let next_of = |p: &str| match p {
                "work" => "personal",
                "personal" => "reading",
                _ => "work",
            };
            let batches: Vec<Vec<String>> = ids
                .iter()
                .map(|id| {
                    let cur = self.task(id).map(|t| t.project.clone()).unwrap_or_default();
                    backend::build_project_args(id, next_of(&cur))
                })
                .collect();
            if self.backend_commit(
                &batches,
                &format!("moved {} task(s)", ids.len()),
                "move failed",
            ) {
                self.selected.clear();
                self.visual = false;
            }
            return;
        }
        for id in &ids {
            if let Some(t) = self
                .tasks
                .iter_mut()
                .find(|t| t.uuid.as_str() == id.as_str())
            {
                t.project = match t.project.as_str() {
                    "work" => "personal".into(),
                    "personal" => "reading".into(),
                    _ => "work".into(),
                };
            }
        }
        self.selected.clear();
        self.visual = false;
        self.set_toast(format!("moved {} task(s)", ids.len()));
        self.sync_projects();
    }
    fn bulk_toggle_tag(&mut self) {
        let ids: Vec<String> = self.selected.iter().cloned().collect();
        if ids.is_empty() {
            return;
        }
        self.push_undo();
        if self.uses_backend() {
            let batches: Vec<Vec<String>> = ids
                .iter()
                .map(|id| {
                    let has = self
                        .task(id)
                        .is_some_and(|t| t.tags.iter().any(|g| g == "urgent"));
                    if has {
                        backend::build_tag_args(id, &[], &["urgent".to_string()])
                    } else {
                        backend::build_tag_args(id, &["urgent".to_string()], &[])
                    }
                })
                .collect();
            if self.backend_commit(
                &batches,
                &format!("retagged {} task(s)", ids.len()),
                "retag failed",
            ) {
                self.selected.clear();
                self.visual = false;
            }
            return;
        }
        for id in &ids {
            if let Some(t) = self
                .tasks
                .iter_mut()
                .find(|t| t.uuid.as_str() == id.as_str())
            {
                if t.tags.iter().any(|g| g == "urgent") {
                    t.tags.retain(|g| g != "urgent");
                } else {
                    t.tags.push("urgent".into());
                }
            }
        }
        self.selected.clear();
        self.visual = false;
        self.set_toast(format!("retagged {} task(s)", ids.len()));
    }
    /// Rebuild the registry from task values plus manual additions.
    /// Unsectioned tasks are not a registry entry; project views render
    /// them as a flat list above the section headers.
    fn sync_sections(&mut self) {
        let mut names: Vec<String> = self
            .tasks
            .iter()
            .map(|t| t.section.trim())
            .filter(|s| !s.is_empty())
            .map(String::from)
            .chain(self.manual_sections.iter().cloned())
            .collect();
        sort_sections(&mut names);
        self.sections = names;
    }

    /// Rebuild the project cache from task data, sorted A–Z. Stale names
    /// leave; when the active project disappears the view falls back to All.
    fn sync_projects(&mut self) {
        let mut names: Vec<String> = self
            .tasks
            .iter()
            .map(|t| t.project.trim())
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();
        names.sort();
        names.dedup();
        self.projects = names;
        self.project_colors = build_project_colors(&self.projects);
        if let View::Project(p) = &self.view {
            if !self.projects.iter().any(|x| x == p) {
                self.view = View::All;
                self.list_offset.set(0);
                self.cursor_to_first_task();
            }
        }
        if self
            .search_origin
            .as_ref()
            .is_some_and(|view| matches!(view, View::Project(p) if !self.projects.contains(p)))
        {
            self.search_origin = Some(View::Inbox);
        }
    }

    /// Display rank: unsectioned leads, then the registry order.
    fn section_rank(&self, section: &str) -> usize {
        let section = section.trim();
        if section.is_empty() {
            return 0;
        }
        self.sections
            .iter()
            .position(|s| s.eq_ignore_ascii_case(section))
            .map(|i| i + 1)
            .unwrap_or(self.sections.len() + 1)
    }

    fn section_exists(&self, name: &str) -> bool {
        self.sections.iter().any(|s| s.eq_ignore_ascii_case(name))
    }

    /// Keep the cursor on the same task after an in-memory sort change.
    fn hold_cursor_on(&mut self, id: &str) {
        if let Some(idx) = self.navigable_rows().iter().position(|r| match r {
            NavRow::Task(u) | NavRow::CompletedTask(u) => u == id,
            _ => false,
        }) {
            self.cursor = idx;
        } else {
            self.cursor_to_first_task();
        }
    }

    /// Section moves are disabled outside project views: flat lists show
    /// no sections, so the hidden value is preserved and a hint points
    /// at project views.
    fn refuse_section_in_inbox(&mut self) -> bool {
        if !matches!(self.view, View::Project(_)) {
            self.set_toast("sections live in project views".into());
            true
        } else {
            false
        }
    }

    fn move_section_forward(&mut self) {
        if self.refuse_section_in_inbox() {
            return;
        }
        let Some(id) = self.cursor_id() else {
            return;
        };
        if self.sections.is_empty() {
            self.set_toast("no sections yet · A adds one".into());
            return;
        }
        self.push_undo();
        let current = self
            .task(&id)
            .map(|t| t.section.clone())
            .unwrap_or_default();
        // An unsectioned (or stale) task lands on the first registry section;
        // a known section advances and wraps around the registry.
        let next = match self
            .sections
            .iter()
            .position(|s| s.eq_ignore_ascii_case(&current))
        {
            Some(i) => self.sections[(i + 1) % self.sections.len()].clone(),
            None => self.sections[0].clone(),
        };
        if self.uses_backend() {
            self.backend_commit(
                &[backend::build_section_args(&id, &next)],
                &format!("moved to {next}"),
                "section move failed",
            );
            return;
        }
        if let Some(t) = self
            .tasks
            .iter_mut()
            .find(|t| t.uuid.as_str() == id.as_str())
        {
            t.section = next.clone();
        }
        self.sync_sections();
        self.hold_cursor_on(&id);
        self.set_toast(format!("moved to {next}"));
    }

    fn open_section_prompt(&mut self, kind: SectionPromptKind) {
        if self.refuse_section_in_inbox() {
            return;
        }
        let old_section = match kind {
            SectionPromptKind::Rename => {
                let Some(id) = self.cursor_id() else {
                    self.set_toast("no task under the cursor".into());
                    return;
                };
                let section = self
                    .task(&id)
                    .map(|t| t.section.clone())
                    .unwrap_or_default();
                if section.trim().is_empty() {
                    self.set_toast("task has no section to rename".into());
                    return;
                }
                Some(section)
            }
            SectionPromptKind::Add | SectionPromptKind::Remove => None,
        };
        self.section_prompt = Some(SectionPrompt {
            kind,
            buffer: String::new(),
            cursor: 0,
            old_section,
        });
        self.clear_toast();
    }

    fn apply_section_prompt(&mut self) {
        let Some(prompt) = self.section_prompt.take() else {
            return;
        };
        let name = prompt.buffer.trim().to_string();
        if name.is_empty() {
            self.set_toast("section name required".into());
            return;
        }
        match prompt.kind {
            SectionPromptKind::Add => self.add_section_name(&name),
            SectionPromptKind::Rename => {
                let old = prompt.old_section.unwrap_or_default();
                self.rename_section(&old, &name);
            }
            SectionPromptKind::Remove => self.remove_section_name(&name),
        }
    }

    fn add_section_name(&mut self, name: &str) {
        if self.section_exists(name) {
            self.set_toast(format!("section {name} already exists"));
            return;
        }
        self.push_undo();
        self.manual_sections.push(name.to_string());
        self.sync_sections();
        self.set_toast(format!("section {name} added"));
    }

    fn rename_section(&mut self, old: &str, new: &str) {
        if new.eq_ignore_ascii_case(old) {
            self.set_toast("section unchanged".into());
            return;
        }
        if self.section_exists(new) {
            self.set_toast(format!("section {new} already exists"));
            return;
        }
        self.push_undo();
        let ids: Vec<String> = self
            .tasks
            .iter()
            .filter(|t| t.section.eq_ignore_ascii_case(old))
            .map(|t| t.uuid.clone())
            .collect();
        if self.uses_backend() {
            if ids.is_empty() {
                self.manual_sections
                    .retain(|s| !s.eq_ignore_ascii_case(old));
                self.sync_sections();
                self.set_toast(format!("renamed {old} to {new}"));
                return;
            }
            let batches: Vec<Vec<String>> = ids
                .iter()
                .map(|id| backend::build_section_args(id, new))
                .collect();
            // Drop the old manual name only after the writes land.
            if self.backend_commit(
                &batches,
                &format!("renamed {old} to {new}"),
                "section rename failed",
            ) {
                self.manual_sections
                    .retain(|s| !s.eq_ignore_ascii_case(old));
                self.sync_sections();
            }
            return;
        }
        // The old name leaves the registry; task values seed the new one.
        self.manual_sections
            .retain(|s| !s.eq_ignore_ascii_case(old));
        for t in self
            .tasks
            .iter_mut()
            .filter(|t| t.section.eq_ignore_ascii_case(old))
        {
            t.section = new.to_string();
        }
        self.sync_sections();
        self.set_toast(format!("renamed {old} to {new}"));
    }

    fn remove_section_name(&mut self, name: &str) {
        let Some(actual) = self
            .sections
            .iter()
            .find(|s| s.eq_ignore_ascii_case(name))
            .cloned()
        else {
            self.set_toast(format!("no section named {name}"));
            return;
        };
        if self
            .tasks
            .iter()
            .any(|t| t.section.eq_ignore_ascii_case(&actual))
        {
            self.set_toast(format!("{actual} is not empty"));
            return;
        }
        self.push_undo();
        self.manual_sections
            .retain(|s| !s.eq_ignore_ascii_case(&actual));
        self.sync_sections();
        self.set_toast(format!("removed empty section {actual}"));
    }
    fn quick_add_commit(&mut self) {
        // Single-create: one Enter is exactly one task; pasted newlines join.
        // Stays in add mode for rapid entry — esc leaves.
        let title: String = self
            .add
            .desc
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        if title.is_empty() {
            self.set_toast("description required".into());
            return;
        }
        // Editor parity: resolve `depends` display IDs before any write.
        // Unknown IDs toast and keep the draft, exactly like popup save.
        let mut new_deps: Vec<String> = Vec::new();
        for tok in &self.add.depends {
            match resolve_depends_token(self, tok) {
                Some(uuid) => {
                    if !new_deps.contains(&uuid) {
                        new_deps.push(uuid);
                    }
                }
                None => {
                    self.set_toast(format!("unknown id: {tok}"));
                    return;
                }
            }
        }
        self.push_undo();
        let mut project = self.add.project.clone();
        let mut due = self.add.due.clone();
        if project.is_empty() || due.is_empty() {
            let (vp, vd) = match &self.view {
                View::Project(p) => (p.clone(), String::new()),
                View::Today => (String::new(), "today".into()),
                _ => (String::new(), String::new()),
            };
            if project.is_empty() {
                project = vp;
            }
            if due.is_empty() {
                due = vd;
            }
        }
        let recur = self.add.recur.trim().to_string();
        if !backend::is_recur_cleared(&recur) && due.trim().is_empty() {
            self.set_toast("recurrence needs a due date".into());
            return;
        }
        // Write-through: `task add` then re-export on success. The draft
        // section (from `a` on a section context) rides along as
        // `siapsection:`; empty stays unsectioned.
        if self.uses_backend() {
            let prio = if self.add.prio_set {
                Some(self.add.prio)
            } else {
                None
            };
            let args = backend::build_add_args(
                &title,
                &project,
                &due,
                prio,
                &self.add.tags,
                &new_deps,
                &recur,
                &self.add.section,
            );
            let refs: Vec<&str> = args.iter().map(String::as_str).collect();
            match self.run_backend(&refs) {
                Ok(_) => {
                    self.refresh_from_backend();
                    self.add.clear_desc();
                    self.add.clear_filter();
                    self.add.picker = 0;
                    self.add.depends.clear();
                    self.add.recur.clear();
                    self.add.focus = AddFocus::Desc;
                    // Preserve a success toast unless refresh already failed.
                    if !self.toast.starts_with("refresh failed") {
                        self.set_toast("created 1 task · esc done".into());
                    }
                    self.clamp_cursor();
                }
                Err(e) => {
                    self.set_toast(format!("add failed: {e}"));
                }
            }
            return;
        }
        let short = self.next_id;
        self.next_id += 1;
        let uuid = mock_uuid(short);
        let prio = self.add.prio;
        let tags = self.add.tags.clone();
        let urgency = match prio {
            'H' => 11.0,
            'M' => 7.0,
            _ => 3.0,
        };
        let recur_value = if backend::is_recur_cleared(&recur) {
            "none".to_string()
        } else {
            recur.clone()
        };
        self.tasks.push(Task {
            uuid,
            short_id: Some(short),
            desc: title,
            project,
            due,
            due_raw: None,
            prio,
            tags,
            section: self.add.section.clone(),
            urgency,
            active: false,
            status: "pending".into(),
            depends: new_deps,
            recur: recur_value,
            recur_parent: String::new(),
            start_end: "not started".into(),
            start_raw: None,
            end: None,
            annotations: Vec::new(),
        });
        self.add.clear_desc();
        self.add.clear_filter();
        self.add.picker = 0;
        self.add.depends.clear();
        self.add.recur.clear();
        self.add.focus = AddFocus::Desc;
        self.sync_projects();
        self.sync_sections();
        self.set_toast("created 1 task · esc done".into());
        self.clamp_cursor();
    }
    /// Apply the highlighted add-picker suggestion into the draft.
    /// A non-empty tag filter creates custom tags: `Enter` toggles each
    /// whitespace/comma-separated word (stripping leading `+`/`#`) so new
    /// tags no longer require a pre-existing suggestion. A non-empty
    /// project/due filter is used as-is on `Enter`, creating a new project
    /// or a custom date instead of requiring a preset pick.
    fn apply_add_suggestion(&mut self) {
        if self.add.focus == AddFocus::Tags && !self.add.filter.trim().is_empty() {
            if self.add.filter.trim().eq_ignore_ascii_case(CTX_CLEAR_TAGS) {
                self.add.tags.clear();
                self.add.clear_filter();
                self.add.picker = 0;
                return;
            }
            let words: Vec<String> = self
                .add
                .filter
                .split([',', ' ', '\t'])
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(|s| s.trim_start_matches(['+', '#']).to_string())
                .filter(|s| !s.is_empty())
                .collect();
            for word in words {
                if self.add.tags.iter().any(|g| g == &word) {
                    self.add.tags.retain(|g| g != &word);
                } else {
                    self.add.tags.push(word);
                }
            }
            self.add.clear_filter();
            self.add.picker = 0;
            return;
        }
        // Custom project: typing a new name + Enter uses it as-is.
        if self.add.focus == AddFocus::Project && !self.add.filter.trim().is_empty() {
            let raw = self.add.filter.trim().to_string();
            if raw.eq_ignore_ascii_case(CTX_CLEAR_PROJECT) {
                self.add.project.clear();
            } else {
                // Prefer canonical casing when the filter exactly matches a known
                // project; otherwise the typed text creates a new project.
                let canonical = add_suggestions(self, &self.add.clone())
                    .into_iter()
                    .find(|s| s.eq_ignore_ascii_case(&raw));
                self.add.project = canonical.unwrap_or(raw);
            }
            self.add.focus = AddFocus::Desc;
            self.add.clear_filter();
            self.add.picker = 0;
            return;
        }
        // Fixed-list `depends`: no typed IDs; Enter always applies the
        // highlighted row below.
        // Custom due: typing a date + Enter uses it as-is.
        if self.add.focus == AddFocus::Due && !self.add.filter.trim().is_empty() {
            let raw = self.add.filter.trim().to_string();
            if raw.eq_ignore_ascii_case(CTX_CLEAR_DATE) {
                self.add.due = String::new();
            } else if let Some(canonical) = add_suggestions(self, &self.add.clone())
                .into_iter()
                .find(|s| s.eq_ignore_ascii_case(&raw))
            {
                self.add.due = canonical;
            } else {
                self.add.due = raw;
            }
            self.add.focus = AddFocus::Desc;
            self.add.clear_filter();
            self.add.picker = 0;
            return;
        }
        let sug = add_suggestions(self, &self.add.clone());
        let pick = sug.get(self.add.picker).cloned().unwrap_or_default();
        if pick.is_empty() {
            return;
        }
        match self.add.focus {
            AddFocus::Tags => {
                if pick == CTX_CLEAR_TAGS {
                    self.add.tags.clear();
                } else if self.add.tags.iter().any(|g| g == &pick) {
                    self.add.tags.retain(|g| g != &pick);
                } else {
                    self.add.tags.push(pick);
                }
                self.add.clear_filter();
                self.add.picker = 0;
            }
            AddFocus::Due => {
                self.add.due = if pick == CTX_CLEAR_DATE {
                    String::new()
                } else {
                    pick
                };
                self.add.focus = AddFocus::Desc;
                self.add.clear_filter();
                self.add.picker = 0;
            }
            AddFocus::Project => {
                self.add.project = if pick == CTX_CLEAR_PROJECT {
                    String::new()
                } else {
                    pick
                };
                self.add.focus = AddFocus::Desc;
                self.add.clear_filter();
                self.add.picker = 0;
            }
            AddFocus::Priority => {
                self.add.prio = match pick.as_str() {
                    "H" => 'H',
                    "M" => 'M',
                    _ => 'L',
                };
                self.add.prio_set = true;
                self.add.focus = AddFocus::Desc;
                self.add.clear_filter();
                self.add.picker = 0;
            }
            AddFocus::Depends => {
                // Enter appends the highlighted link instead of replacing
                // the draft, so repeated Enters accumulate several links.
                // Stays in the picker for multi-entry; esc backs out.
                if pick == CTX_CLEAR_DEPENDS {
                    self.add.depends.clear();
                } else {
                    let id = depends_suggestion_id(&pick);
                    if !id.is_empty() && !self.add.depends.iter().any(|d| d == &id) {
                        self.add.depends.push(id);
                    }
                }
                self.add.clear_filter();
                self.add.picker = 0;
            }
            AddFocus::Recur => {
                self.add.recur = if pick == CTX_CLEAR_RECUR {
                    String::new()
                } else {
                    pick
                };
                self.add.focus = AddFocus::Desc;
                self.add.clear_filter();
                self.add.picker = 0;
            }
            AddFocus::More => {
                self.add.focus = match pick.as_str() {
                    "@ project" => AddFocus::Project,
                    "! priority" => AddFocus::Priority,
                    "# tags" => AddFocus::Tags,
                    "~ due" => AddFocus::Due,
                    "& depends" => AddFocus::Depends,
                    "* recur" => AddFocus::Recur,
                    _ => AddFocus::Due,
                };
                self.add.clear_filter();
                self.add.picker = 0;
            }
            AddFocus::Desc => {}
        }
    }
    /// Delete a recurring template and the later pending occurrences while
    /// keeping the edited occurrence and its earlier occurrences. Missing
    /// UUIDs are treated as already deleted, so retrying a partly finished
    /// detach can complete the cleanup.
    fn remove_recurring_series(&mut self, parent: &str, siblings: &[String]) -> Result<(), String> {
        let mut doomed = Vec::with_capacity(siblings.len() + 1);
        if !parent.is_empty() {
            doomed.push(parent.to_string());
        }
        doomed.extend(siblings.iter().cloned());
        for uuid in doomed {
            let args = backend::build_recur_delete_args(&uuid);
            let refs: Vec<&str> = args.iter().map(String::as_str).collect();
            match self.run_backend(&refs) {
                Ok(_) => {}
                Err(e) if is_missing_task_error(&e) => {}
                Err(e) => return Err(e),
            }
            self.selected.remove(&uuid);
        }
        Ok(())
    }

    /// Strip `recur`/`parent` from one kept occurrence through `task edit`.
    fn detach_recurring_task(&self, uuid: &str) -> Result<(), String> {
        let args = backend::build_detach_recur_args(uuid);
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        self.run_backend(&refs).map(|_| ())
    }

    fn save_popup(&mut self) {
        // Enter saves and returns to VIEW; the popup stays open for tweaks.
        let (task_id, desc, project, due, prio, tags_str, depends_str, recur_str) =
            match self.popup.as_ref() {
                Some(p) => (
                    p.task_id.clone(),
                    p.desc.clone(),
                    p.project.clone(),
                    p.due.clone(),
                    p.prio,
                    p.tags.clone(),
                    p.depends.clone(),
                    p.recur.clone(),
                ),
                None => return,
            };
        if desc.trim().is_empty() {
            self.set_toast("description required".into());
            return;
        }
        // Recurrence needs a due date; taskwarrior enforces the same.
        if !backend::is_recur_cleared(&recur_str) && due.trim().is_empty() {
            self.set_toast("recurrence needs a due date".into());
            return;
        }
        // Validate `depends` before any write: unknown IDs and self-links
        // toast and stay in EDIT.
        let new_deps = match parse_depends_uuids(self, &depends_str) {
            Ok(v) => v,
            Err(bad) => {
                self.set_toast(format!("unknown id: {bad}"));
                return;
            }
        };
        if new_deps.iter().any(|d| d == &task_id) {
            self.set_toast("can't depend on itself".into());
            return;
        }
        if creates_cycle(&self.tasks, &task_id, &new_deps) {
            self.set_toast("circular dependency".into());
            return;
        }
        let old_deps: Vec<String> = self
            .task(&task_id)
            .map(|t| t.depends.clone())
            .unwrap_or_default();
        // Clearing recurrence on a generated occurrence keeps that task and
        // every earlier occurrence, but removes the hidden template and the
        // later occurrences. Kept occurrences all become one-offs.
        let saved = self.task(&task_id).cloned();
        let clearing_recur =
            saved.as_ref().is_some_and(is_recurring) && backend::is_recur_cleared(&recur_str);
        let recurring_parent = saved
            .as_ref()
            .map(|t| t.recur_parent.clone())
            .unwrap_or_default();
        let (previous_recurring, later_recurring) = match saved.as_ref() {
            Some(t) if clearing_recur && !recurring_parent.is_empty() => {
                split_recurring_siblings(&self.tasks, t, &recurring_parent)
            }
            _ => (Vec::new(), Vec::new()),
        };
        self.push_undo();
        // Write-through: `task <uuid> modify` then re-export on success.
        // Failures toast and preserve list/selection/cursor.
        if self.uses_backend() {
            if clearing_recur {
                // Detach earlier occurrences first so a failure leaves the
                // edited task recurring and the cleanup retryable.
                for uuid in &previous_recurring {
                    if let Err(e) = self.detach_recurring_task(uuid) {
                        self.set_toast(format!("recurrence detach failed: {e}"));
                        self.refresh_from_backend();
                        return;
                    }
                }
                if let Err(e) = self.remove_recurring_series(&recurring_parent, &later_recurring) {
                    self.set_toast(format!("recurrence cleanup failed: {e}"));
                    self.refresh_from_backend();
                    return;
                }
                if let Err(e) = self.detach_recurring_task(&task_id) {
                    self.set_toast(format!("recurrence detach failed: {e}"));
                    self.refresh_from_backend();
                    return;
                }
            }
            let desc = desc.trim().to_string();
            let project = project.trim().to_string();
            let due = due.trim().to_string();
            let new_tags: Vec<String> = tags_str
                .split([',', ' '])
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(|s| s.trim_start_matches(['+', '#']).to_string())
                .collect();
            let old_tags: Vec<String> = self
                .task(&task_id)
                .map(|t| t.tags.clone())
                .unwrap_or_default();
            let section = self
                .task(&task_id)
                .map(|t| t.section.clone())
                .unwrap_or_default();
            let args = backend::build_modify_args(
                &task_id, &desc, &project, &due, prio, &old_tags, &new_tags, &section, &recur_str,
            );
            let refs: Vec<&str> = args.iter().map(String::as_str).collect();
            match self.run_backend(&refs) {
                Ok(_) => {}
                Err(e) => {
                    self.set_toast(format!("save failed: {e}"));
                    return;
                }
            }
            // Native `depends`: add-if-missing / remove-one per UUID.
            let added: Vec<&String> = new_deps.iter().filter(|d| !old_deps.contains(d)).collect();
            let removed: Vec<&String> = old_deps.iter().filter(|d| !new_deps.contains(d)).collect();
            for dep in added {
                let args = backend::build_depends_add_args(&task_id, dep);
                let refs: Vec<&str> = args.iter().map(String::as_str).collect();
                if let Err(e) = self.run_backend(&refs) {
                    self.set_toast(format!("depends failed: {e}"));
                    self.refresh_from_backend();
                    return;
                }
            }
            for dep in removed {
                let args = backend::build_depends_remove_args(&task_id, dep);
                let refs: Vec<&str> = args.iter().map(String::as_str).collect();
                if let Err(e) = self.run_backend(&refs) {
                    self.set_toast(format!("depends failed: {e}"));
                    self.refresh_from_backend();
                    return;
                }
            }
            self.refresh_from_backend();
            // Re-sync the draft to the saved values (short IDs can shift and
            // text saves trim) so a later `e` + `esc` compares clean instead
            // of showing a stale `● unsaved`.
            let synced = self.task(&task_id).cloned().map(|t| {
                let due = editable_due_input(&t);
                let depends = depends_display_for(self, &t);
                let recur = if backend::is_recur_cleared(&t.recur) {
                    String::new()
                } else {
                    t.recur.clone()
                };
                (
                    t.desc,
                    t.project,
                    due,
                    t.prio,
                    t.tags.join(", "),
                    depends,
                    recur,
                )
            });
            if let Some((desc, project, due, prio, tags, depends, recur)) = synced {
                if let Some(p) = self.popup.as_mut() {
                    p.desc = desc;
                    p.project = project;
                    p.due = due;
                    p.prio = prio;
                    p.tags = tags;
                    p.depends = depends;
                    p.recur = recur;
                    p.dirty = false;
                    p.mode = PopupMode::View;
                    p.suggest_idx = 0;
                    let field = p.field;
                    p.edit_cursor = p.field_len(field);
                }
            }
            if !self.toast.starts_with("refresh failed") {
                self.set_toast("saved".into());
            }
            return;
        }
        let depends_to_store = new_deps.clone();
        if clearing_recur {
            // Earlier occurrences stay as one-offs instead of being deleted.
            for uuid in &previous_recurring {
                if let Some(t) = self.tasks.iter_mut().find(|t| &t.uuid == uuid) {
                    t.recur = "none".into();
                    t.recur_parent.clear();
                }
            }
            for uuid in &later_recurring {
                self.tasks.retain(|t| &t.uuid != uuid);
                self.selected.remove(uuid);
            }
            if let Some(t) = self.tasks.iter_mut().find(|t| t.uuid == task_id) {
                t.recur = "none".into();
                t.recur_parent.clear();
            }
        }
        if let Some(t) = self.tasks.iter_mut().find(|t| t.uuid == task_id) {
            t.desc = desc.trim().into();
            t.project = project.trim().into();
            t.due = due.trim().into();
            t.prio = prio;
            t.tags = tags_str
                .split([',', ' '])
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(|s| s.trim_start_matches(['+', '#']).to_string())
                .collect();
            // `status` stays read-only; annotations save through
            // `annotate`/`denotate`, never through this modify.
            t.depends = depends_to_store;
            let recur_draft = recur_str.trim().to_string();
            t.recur = if backend::is_recur_cleared(&recur_draft) {
                "none".into()
            } else {
                recur_draft
            };
            t.urgency = match t.prio {
                'H' => 11.0,
                'M' => 7.0,
                _ => 3.0,
            };
        }
        // Re-sync the draft to the saved values (saves trim text) so a later
        // `e` + `esc` compares clean instead of showing a stale `● unsaved`.
        let synced = self.task(&task_id).cloned().map(|t| {
            let due = editable_due_input(&t);
            let depends = depends_display_for(self, &t);
            let recur = if backend::is_recur_cleared(&t.recur) {
                String::new()
            } else {
                t.recur.clone()
            };
            (
                t.desc,
                t.project,
                due,
                t.prio,
                t.tags.join(", "),
                depends,
                recur,
            )
        });
        if let Some(p) = self.popup.as_mut() {
            if let Some((desc, project, due, prio, tags, depends, recur)) = synced {
                p.desc = desc;
                p.project = project;
                p.due = due;
                p.prio = prio;
                p.tags = tags;
                p.depends = depends;
                p.recur = recur;
            }
            p.dirty = false;
            p.mode = PopupMode::View;
            p.suggest_idx = 0;
            let field = p.field;
            p.edit_cursor = p.field_len(field);
        }
        self.sync_projects();
        self.set_toast("saved".into());
    }
    /// Apply the highlighted inline suggestion into the draft field.
    /// `Enter` applies (tags replace the in-progress token) then saves.
    /// `Tab`/`Shift+Tab`/`Up`/`Down` only move the highlight.
    fn apply_popup_suggestion(&mut self) {
        let (sug, idx, field) = match self.popup.as_ref() {
            Some(p) => (popup_suggestions(self, p), p.suggest_idx, p.field),
            None => return,
        };
        let Some(pick) = sug.get(idx).cloned() else {
            return;
        };
        if let Some(p) = self.popup.as_mut() {
            match field {
                F_PROJECT => {
                    p.project = if pick == CTX_CLEAR_PROJECT {
                        String::new()
                    } else {
                        pick
                    }
                }
                F_DUE => {
                    p.due = if pick == CTX_CLEAR_DATE {
                        String::new()
                    } else {
                        pick
                    }
                }
                F_PRIO => {
                    p.prio = match pick.as_str() {
                        "H" => 'H',
                        "M" => 'M',
                        _ => 'L',
                    }
                }
                F_TAGS => {
                    p.tags = if pick == CTX_CLEAR_TAGS {
                        String::new()
                    } else {
                        apply_tag_pick(&p.tags.clone(), &pick)
                    };
                }
                F_RECUR => {
                    p.recur = if pick == CTX_CLEAR_RECUR {
                        String::new()
                    } else {
                        pick
                    };
                }
                _ => return,
            }
            p.dirty = true;
            p.suggest_idx = idx;
            let field = p.field;
            p.edit_cursor = p.field_len(field);
        }
    }

    /// Apply the highlighted `depends` suggestion to the draft: the
    /// highlighted ID is appended, `no dependencies` clears the draft.
    /// Returns true when the draft changed (the Enter handler saves right
    /// after, so no toast here — `save_popup`'s result wins); false keeps
    /// the current toast and stays in EDIT.
    fn apply_depends_suggestion(&mut self) -> bool {
        let (sug, idx) = match self.popup.as_ref() {
            Some(p) => (popup_suggestions(self, p), p.suggest_idx),
            None => return false,
        };
        let Some(pick) = sug.get(idx).cloned() else {
            return false;
        };
        if pick == CTX_CLEAR_DEPENDS {
            if let Some(p) = self.popup.as_mut() {
                p.depends = String::new();
                p.dirty = true;
                p.suggest_idx = 0;
                let field = p.field;
                p.edit_cursor = p.field_len(field);
            }
            return true;
        }
        let id = depends_suggestion_id(&pick);
        if id.is_empty() {
            return false;
        }
        if self
            .popup
            .as_ref()
            .is_some_and(|p| parse_depends_tokens(&p.depends).iter().any(|x| x == &id))
        {
            // Already linked (hidden from the list, so unreachable by
            // picking); keep the draft and let the save confirm it.
            return true;
        }
        if let Some(p) = self.popup.as_mut() {
            // Fixed list: append the highlighted link so repeated Enters
            // accumulate several links.
            if p.depends.trim().is_empty() {
                p.depends = id;
            } else {
                p.depends = format!("{}, {id}", p.depends.trim_end_matches([',', ' ']));
            }
            p.dirty = true;
            p.suggest_idx = 0;
            let field = p.field;
            p.edit_cursor = p.field_len(field);
        }
        true
    }

    /// Revert only the focused EDIT field to its saved task value.
    /// Returns true when something was actually discarded.
    fn revert_popup_field(&mut self) -> bool {
        let (task_id, field) = match self.popup.as_ref() {
            Some(p) => (p.task_id.clone(), p.field),
            None => return false,
        };
        let Some(t) = self.task(&task_id).cloned() else {
            return false;
        };
        let changed = match self.popup.as_ref() {
            Some(p) => match field {
                F_DESC => p.desc != t.desc,
                F_PROJECT => p.project != t.project,
                F_DUE => p.due != editable_due_input(&t),
                F_PRIO => p.prio != t.prio,
                F_TAGS => {
                    let cur: Vec<String> = p
                        .tags
                        .split([',', ' '])
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .map(|s| s.trim_start_matches(['+', '#']).to_string())
                        .collect();
                    cur != t.tags
                }
                F_DEPENDS => match parse_depends_uuids(self, &p.depends) {
                    Ok(cur) => {
                        let mut a = cur;
                        let mut b = t.depends.clone();
                        a.sort();
                        b.sort();
                        a != b
                    }
                    Err(_) => true,
                },
                F_RECUR => {
                    let cur = p.recur.trim();
                    let saved = t.recur.trim();
                    backend::is_recur_cleared(cur) != backend::is_recur_cleared(saved)
                        || (!backend::is_recur_cleared(cur)
                            && cur.to_lowercase() != saved.to_lowercase())
                }
                _ => false,
            },
            None => false,
        };
        // Compute the reverted display before taking a mutable borrow.
        let depends_display = if field == F_DEPENDS {
            self.tasks
                .iter()
                .find(|x| x.uuid == t.uuid)
                .map(|task| {
                    let mut ids: Vec<String> = task
                        .depends
                        .iter()
                        .filter_map(|d| {
                            self.tasks
                                .iter()
                                .find(|x| x.uuid == *d)
                                .map(task_id_display)
                        })
                        .collect();
                    ids.sort();
                    ids.join(", ")
                })
                .unwrap_or_default()
        } else {
            String::new()
        };
        if let Some(p) = self.popup.as_mut() {
            match field {
                F_DESC => p.desc = t.desc.clone(),
                F_PROJECT => p.project = t.project.clone(),
                F_DUE => p.due = editable_due_input(&t),
                F_PRIO => p.prio = t.prio,
                F_TAGS => p.tags = t.tags.join(", "),
                F_DEPENDS => {
                    p.depends = depends_display;
                }
                F_RECUR => {
                    p.recur = if backend::is_recur_cleared(&t.recur) {
                        String::new()
                    } else {
                        t.recur.clone()
                    };
                }
                _ => {}
            }
            p.mode = PopupMode::View;
            p.suggest_idx = 0;
            p.edit_cursor = p.field_len(p.field);
        }
        // Recompute dirty across all fields so other unsaved edits survive.
        if let (Some(p), Some(task)) = (self.popup.as_ref(), self.task(&t.uuid).cloned()) {
            let dirty = popup_is_dirty(self, p, &task);
            if let Some(pp) = self.popup.as_mut() {
                pp.dirty = dirty;
            }
        }
        changed
    }

    /// Toggle done/undone for the popup task without closing the popup.
    /// Toggle done/undone for the popup task without closing the popup.
    /// `status` is read-only, so a done toggle is a saved backend change,
    /// never an unsaved draft edit: recompute dirty from the remaining
    /// draft fields so the title never shows a stale `● unsaved`.
    fn toggle_popup_done(&mut self) {
        let id = match self.popup.as_ref() {
            Some(p) => p.task_id.clone(),
            None => return,
        };
        let held_field = self.popup.as_ref().map(|p| p.field).unwrap_or(0);
        self.toggle_ids(std::slice::from_ref(&id));
        // Keep the popup open on the same task with the fresh status.
        if let Some(t) = self.task(&id).cloned() {
            if let Some(p) = self.popup.as_mut() {
                p.field = held_field.min(Popup::field_count().saturating_sub(1));
                p.suggest_idx = 0;
                p.edit_cursor = p.field_len(p.field);
            }
            if let (Some(p), Some(task)) = (self.popup.as_ref(), self.task(&id).cloned()) {
                let _ = t;
                let dirty = popup_is_dirty(self, p, &task);
                if let Some(pp) = self.popup.as_mut() {
                    pp.dirty = dirty;
                }
            }
        }
    }

    /// Save the `annotation` draft via `task <uuid> annotate <text>`.
    /// Empty drafts toast and stay open; backend failures toast with
    /// stderr. Stays in annotating mode with a cleared draft for rapid
    /// entry, mirroring the add-box commit behavior.
    fn annotate_current(&mut self) {
        let (task_id, text) = match self.popup.as_ref() {
            Some(p) => (p.task_id.clone(), p.annotation_draft.trim().to_string()),
            None => return,
        };
        if text.is_empty() {
            self.set_toast("annotation required".into());
            return;
        }
        self.push_undo();
        if self.uses_backend() {
            let args = backend::build_annotate_args(&task_id, &text);
            let refs: Vec<&str> = args.iter().map(String::as_str).collect();
            match self.run_backend(&refs) {
                Ok(_) => {}
                Err(e) => {
                    self.set_toast(format!("annotate failed: {e}"));
                    return;
                }
            }
            self.refresh_from_backend();
            if self.toast.starts_with("refresh failed") {
                return;
            }
        } else if let Some(t) = self.tasks.iter_mut().find(|t| t.uuid == task_id) {
            t.annotations.push(Annotation {
                entry: now_export_iso(),
                text,
            });
        }
        if let Some(p) = self.popup.as_mut() {
            p.annotation_draft.clear();
            p.annotation_cursor = 0;
            p.annotating = true;
        }
        let task_len = self
            .task(&task_id)
            .map(|t| t.annotations.len())
            .unwrap_or(0);
        if let Some(p) = self.popup.as_mut() {
            p.annotation_idx = task_len.saturating_sub(1);
        }
        self.set_toast("annotated".into());
    }

    /// Remove the selected annotation via `task <uuid> denotate <text>`.
    /// Taskwarrior matches by exact description text; the popup keeps its
    /// selection clamped to the remaining notes.
    fn denotate_selected(&mut self) {
        let (task_id, idx) = match self.popup.as_ref() {
            Some(p) => (p.task_id.clone(), p.annotation_idx),
            None => return,
        };
        let target = match self.task(&task_id).and_then(|t| t.annotations.get(idx)) {
            Some(a) => a.text.clone(),
            None => {
                self.set_toast("no annotation".into());
                return;
            }
        };
        self.push_undo();
        if self.uses_backend() {
            let args = backend::build_denotate_args(&task_id, &target);
            let refs: Vec<&str> = args.iter().map(String::as_str).collect();
            if let Err(e) = self.run_backend(&refs) {
                self.set_toast(format!("denotate failed: {e}"));
                return;
            }
            self.refresh_from_backend();
            if self.toast.starts_with("refresh failed") {
                return;
            }
        } else if let Some(t) = self.tasks.iter_mut().find(|t| t.uuid == task_id) {
            if idx < t.annotations.len() {
                t.annotations.remove(idx);
            }
        }
        let remaining = self
            .task(&task_id)
            .map(|t| t.annotations.len().saturating_sub(1))
            .unwrap_or(0);
        if let Some(p) = self.popup.as_mut() {
            p.annotation_idx = p.annotation_idx.min(remaining);
            p.annotating = false;
        }
        self.set_toast("annotation removed".into());
    }

    /// Cycle the selected annotation with clamping.
    fn cycle_annotation(&mut self, delta: i32) {
        let len = match self.popup.as_ref() {
            Some(p) => self
                .task(&p.task_id)
                .map(|t| t.annotations.len())
                .unwrap_or(0),
            None => return,
        };
        if len == 0 {
            return;
        }
        if let Some(p) = self.popup.as_mut() {
            let next = (p.annotation_idx as i32 + delta).clamp(0, len as i32 - 1);
            p.annotation_idx = next as usize;
        }
    }
}

// ---------------------------------------------------------------------------
// terminal boilerplate
// ---------------------------------------------------------------------------

fn enter_tui() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(out)).map_err(io::Error::other)
}

fn leave_tui(mut term: Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(term.backend_mut(), LeaveAlternateScreen)?;
    term.show_cursor()?;
    Ok(())
}

fn fit(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else if max <= 1 {
        "…".to_string()
    } else {
        let t: String = s.chars().take(max - 1).collect();
        format!("{t}…")
    }
}

fn parse_help_flag() -> bool {
    std::env::args()
        .skip(1)
        .any(|a| a == "--help" || a == "-h" || a == "help")
}

fn print_help() {
    println!("siap — taskwarrior TUI");
    println!();
    println!("Usage: cargo run");
    println!();
    println!("Keys: j/k move · space select · a new · d done/undone · s start/stop");
    println!("      i/Enter open · D delete · u undo · / search · ? help · q quit");
    println!("      Tab switch tabs (no wrap) · 1-3 Inbox/Today/All · 4+ projects");
    println!("      Search is hotkey-only (`/`); esc returns to the previous tab");
    println!("      v visual · esc cancel · A/R/X sections (project views)");
    println!("Add: # tags (Enter creates new) · ~ due (custom ok) · @ project (custom ok)");
    println!("     ! priority · & depends · * recur · + more · ↑/↓/Tab pick · Enter apply");
    println!("Inspector: j/k fields · e edit one field · Tab/↑/↓ choose · ←/→ cursor");
    println!("           Enter apply+save · esc close (VIEW) / discard field (EDIT) · d done");
}

// ---------------------------------------------------------------------------
// event loop
// ---------------------------------------------------------------------------

fn main() -> io::Result<()> {
    if parse_help_flag() {
        print_help();
        return Ok(());
    }
    // Old mockup flags are gone; anything else just runs the single UI.
    run()
}

fn run() -> io::Result<()> {
    let mut app = App::new();
    // 3.1 + 3.4: register the section UDA on launch, then load live data;
    // failures toast and keep the current list with cursor held by `uuid`.
    app.bootstrap_backend();
    app.refresh_from_backend();
    let mut term = enter_tui()?;
    loop {
        app.expire_toast();
        term.draw(|f| render(f, &app))?;
        if !event::poll(Duration::from_millis(120))? {
            continue;
        }
        if let Event::Key(k) = event::read()? {
            if k.modifiers.contains(KeyModifiers::CONTROL) && k.code == KeyCode::Char('c') {
                break;
            }
            if app.confirm_delete.is_some() {
                match k.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => app.confirm_delete_yes(),
                    KeyCode::Char('n') | KeyCode::Esc => app.confirm_delete = None,
                    _ => {}
                }
                continue;
            }
            // The inline section prompt blocks list and add keys.
            if route_key(&mut app, k.code) {
                continue;
            }
            // Add box owns keys even with the inspector open, so `a` keeps
            // working behind the split pane; Esc leaves add mode first.
            if app.add_mode {
                add_key(&mut app, k.code, k.modifiers);
                continue;
            }
            // Search input owns keys while typing, even with the inspector
            // open behind it — `/` keeps the inspector and focuses Search.
            if app.view == View::Search
                && app.search_input
                && search_key(&mut app, k.code, k.modifiers)
            {
                continue;
            }
            if app.popup.is_some() {
                if app.popup_list_focus {
                    list_focus_key(&mut app, k.code, k.modifiers);
                } else {
                    popup_key(&mut app, k.code, k.modifiers);
                }
                continue;
            }
            // Search view routes before the list keymap so input mode can
            // consume printable keys instead of firing hotkeys.
            if app.view == View::Search && search_key(&mut app, k.code, k.modifiers) {
                continue;
            }
            if k.modifiers.contains(KeyModifiers::CONTROL)
                && matches!(k.code, KeyCode::Char('z' | 'Z'))
            {
                app.undo();
                continue;
            }
            match k.code {
                KeyCode::Char('q') => {
                    if app.show_help {
                        app.show_help = false;
                    } else {
                        break;
                    }
                }
                KeyCode::Esc => {
                    if app.show_help {
                        app.show_help = false;
                    } else if app.visual || !app.selected.is_empty() {
                        app.exit_visual(false);
                        app.clear_toast();
                    }
                }
                KeyCode::Char('?') => app.show_help = !app.show_help,
                // `h`/`l` never switch views; views cycle with Tab/Shift+Tab
                // (or jump with 1-9). Inspector open/close lives in popup_key
                // and list_focus_key, which own all keys while it is open.
                KeyCode::Char('h') | KeyCode::Char('l') | KeyCode::Left => {}
                // `→` opens/toggles exactly like Enter; it never changes views.
                KeyCode::Right => open_cursor_row(&mut app),
                KeyCode::Char('i') => {
                    if app.view != View::Search && app.popup.is_none() && app.cursor_id().is_some()
                    {
                        open_cursor_row(&mut app);
                    }
                }
                KeyCode::Char('j') | KeyCode::Down => app.move_down(),
                KeyCode::Char('k') | KeyCode::Up => app.move_up(),
                KeyCode::Enter => open_cursor_row(&mut app),
                KeyCode::Char(' ') => {
                    if let Some(id) = app.cursor_id() {
                        app.toggle_select(id);
                    }
                }
                KeyCode::Char('v') => {
                    if app.visual {
                        app.exit_visual(true);
                    } else {
                        app.enter_visual();
                    }
                }
                KeyCode::Char('a') => {
                    // Search has no add box; it owns `i` for input instead.
                    if app.view != View::Search {
                        app.add_mode = true;
                        app.add = AddDraft::fresh_in_section(cursor_section(&app));
                    }
                }
                KeyCode::Char('d') => {
                    if !app.selected.is_empty() {
                        let ids: Vec<String> = app.selected.iter().cloned().collect();
                        app.toggle_ids(&ids);
                    } else {
                        app.single_done();
                    }
                }
                KeyCode::Char('u') => app.undo(),
                KeyCode::Char('s') => app.single_start_stop(),
                KeyCode::Char('D') => app.request_delete(),
                KeyCode::Char('m') => {
                    if !app.selected.is_empty() {
                        app.bulk_move_project();
                    } else {
                        app.move_section_forward();
                    }
                }
                KeyCode::Char('t') => {
                    if !app.selected.is_empty() {
                        app.bulk_toggle_tag();
                    }
                }
                KeyCode::Char('/') => app.open_search(),
                // 3.4: manual refresh picks up external edits; failures
                // toast and preserve list/selection/cursor.
                KeyCode::Char('r') => app.refresh_from_backend(),
                KeyCode::Char('A') => app.open_section_prompt(SectionPromptKind::Add),
                KeyCode::Char('R') => app.open_section_prompt(SectionPromptKind::Rename),
                KeyCode::Char('X') => app.open_section_prompt(SectionPromptKind::Remove),
                KeyCode::Char('1') => app.switch_view(1),
                KeyCode::Char('2') => app.switch_view(2),
                KeyCode::Char('3') => app.switch_view(3),
                KeyCode::Char(c @ '4'..='9') => {
                    app.switch_view(c.to_digit(10).unwrap_or(4) as usize)
                }
                KeyCode::Tab => app.cycle_view(1),
                KeyCode::BackTab => app.cycle_view(-1),
                _ => {}
            }
        }
    }
    leave_tui(term)
}

/// Open the row under the list cursor: toggle a header, or open the task
/// popup. Shared by the list keymap and Search input mode.
fn open_cursor_row(app: &mut App) {
    match app.cursor_nav() {
        Some(NavRow::SectionHeader(label)) => app.toggle_collapsed(&label),
        Some(NavRow::CompletedHeader) => app.toggle_collapsed("COMPLETED"),
        Some(NavRow::Task(_)) | Some(NavRow::CompletedTask(_)) => {
            // The inspector owns focus; drop Search input mode so its
            // query caret does not stay visible behind the detail pane.
            app.search_input = false;
            if let Some(t) = app.cursor_id().and_then(|id| app.task(&id).cloned()) {
                app.popup = Some(Popup::open(app, &t));
                app.popup_list_focus = false;
            }
        }
        None => {}
    }
}

/// List-behind-inspector keyboard for the split view. Navigation runs
/// here (`j`/`k`/`↑`/`↓` move rows and the open preview follows the cursor);
/// `i`/`Enter`/`l`/`→` hand focus back to the inspector, `a` opens the add
/// box behind it, `esc`/`q` close it. Every other list hotkey (`space`,
/// `s`, `d`, `u`, `D`, `m`, `t`, `v`, `r`, `A`/`R`/`X`, `1`–`9`, `/`, `?`)
/// works exactly like the main list; a clean preview re-seats on the cursor
/// task after each one so it never goes stale (dirty/editing drafts stay
/// untouched).
fn list_focus_key(app: &mut App, code: KeyCode, mods: KeyModifiers) {
    // Undo works behind the inspector too.
    if mods.contains(KeyModifiers::CONTROL) && matches!(code, KeyCode::Char('z' | 'Z')) {
        app.undo();
        refresh_list_preview(app);
        return;
    }
    // `a` opens the add box without closing the inspector; add mode then
    // owns keys until Esc. Search has no add box.
    if code == KeyCode::Char('a') {
        if app.view != View::Search {
            app.add_mode = true;
            app.add = AddDraft::fresh_in_section(cursor_section(app));
        }
        return;
    }
    let before = app.cursor_id();
    // True for keys that can change task data: the clean preview refreshes
    // afterwards so `s`/`d`/etc show straight away.
    let mut mutated = false;
    match code {
        KeyCode::Char('j') | KeyCode::Down => app.move_down(),
        KeyCode::Char('k') | KeyCode::Up => app.move_up(),
        KeyCode::Char('i') | KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => {
            app.popup_list_focus = false;
        }
        KeyCode::Esc => {
            if app.show_help {
                app.show_help = false;
            } else if app.visual || !app.selected.is_empty() {
                app.exit_visual(false);
                app.clear_toast();
            } else {
                app.popup = None;
                app.popup_list_focus = false;
                app.clear_toast();
            }
        }
        KeyCode::Char('q') => {
            if app.show_help {
                app.show_help = false;
            } else {
                app.popup = None;
                app.popup_list_focus = false;
                app.clear_toast();
            }
        }
        KeyCode::Char('?') => app.show_help = !app.show_help,
        // `/` keeps the inspector open and focuses Search input.
        KeyCode::Char('/') => app.open_search(),
        // Tab always cycles views, even with the inspector open behind the list.
        KeyCode::Tab => app.cycle_view(1),
        KeyCode::BackTab => app.cycle_view(-1),
        // `h` never switches views; views cycle with Tab/Shift+Tab.
        KeyCode::Char('h') | KeyCode::Left => {}
        KeyCode::Char(' ') => {
            if let Some(id) = app.cursor_id() {
                app.toggle_select(id);
                mutated = true;
            }
        }
        KeyCode::Char('v') => {
            if app.visual {
                app.exit_visual(true);
            } else {
                app.enter_visual();
            }
            mutated = true;
        }
        KeyCode::Char('d') => {
            if !app.selected.is_empty() {
                let ids: Vec<String> = app.selected.iter().cloned().collect();
                app.toggle_ids(&ids);
            } else {
                app.single_done();
            }
            mutated = true;
        }
        KeyCode::Char('u') => {
            app.undo();
            mutated = true;
        }
        KeyCode::Char('s') => {
            app.single_start_stop();
            mutated = true;
        }
        KeyCode::Char('D') => {
            app.request_delete();
            mutated = true;
        }
        KeyCode::Char('m') => {
            if !app.selected.is_empty() {
                app.bulk_move_project();
            } else {
                app.move_section_forward();
            }
            mutated = true;
        }
        KeyCode::Char('t') => {
            if !app.selected.is_empty() {
                app.bulk_toggle_tag();
                mutated = true;
            }
        }
        // 3.4: manual refresh picks up external edits; failures
        // toast and preserve list/selection/cursor.
        KeyCode::Char('r') => {
            app.refresh_from_backend();
            mutated = true;
        }
        KeyCode::Char('A') => {
            app.open_section_prompt(SectionPromptKind::Add);
            mutated = true;
        }
        KeyCode::Char('R') => {
            app.open_section_prompt(SectionPromptKind::Rename);
            mutated = true;
        }
        KeyCode::Char('X') => {
            app.open_section_prompt(SectionPromptKind::Remove);
            mutated = true;
        }
        KeyCode::Char('1') => app.switch_view(1),
        KeyCode::Char('2') => app.switch_view(2),
        KeyCode::Char('3') => app.switch_view(3),
        KeyCode::Char(c @ '4'..='9') => app.switch_view(c.to_digit(10).unwrap_or(4) as usize),
        _ => {}
    }
    // The preview follows the cursor, re-seating on the new row; headers
    // have no task, so the inspector simply holds its row there. Refocus
    // and close keys leave the cursor alone, so the open draft keeps its
    // field position instead of reopening fresh. Mutating keys refresh a
    // clean preview in place instead.
    if mutated {
        refresh_list_preview(app);
    } else if app.cursor_id() != before {
        app.follow_cursor_with_popup();
    }
}

/// Re-seat a clean inspector preview on the cursor task without losing the
/// focused field position. Dirty, editing, and annotating drafts are never
/// touched (`follow_cursor_with_popup` already guards those).
fn refresh_list_preview(app: &mut App) {
    if app.popup.is_none() {
        return;
    }
    let field = app.popup.as_ref().map(|p| p.field).unwrap_or(0);
    app.follow_cursor_with_popup();
    if let Some(p) = app.popup.as_mut() {
        let max = Popup::field_count().saturating_sub(1);
        p.field = field.min(max);
        p.edit_cursor = p.field_len(p.field);
    }
}

/// Search-view keyboard. Input mode consumes every printable key (so
/// `q`/`l`/`?` type instead of firing hotkeys), `←`/`→` move the query
/// caret, and Tab/Shift+Tab/Up/Down navigate the results; `esc` exits
/// input keeping the query. List mode takes `i` and `esc`, then returns
/// false so the shared list keymap runs.
fn search_key(app: &mut App, code: KeyCode, mods: KeyModifiers) -> bool {
    if app.search_input {
        match code {
            KeyCode::Esc => {
                if app.show_help {
                    app.show_help = false;
                } else if app.search_query.is_empty() {
                    app.leave_search();
                } else {
                    app.search_input = false;
                }
            }
            KeyCode::Backspace => {
                app.search_cursor = delete_before(&mut app.search_query, app.search_cursor);
                app.cursor = 0;
                app.clamp_cursor();
            }
            KeyCode::Delete => {
                app.search_cursor = delete_at(&mut app.search_query, app.search_cursor);
                app.cursor = 0;
                app.clamp_cursor();
            }
            KeyCode::Up | KeyCode::BackTab => app.move_up(),
            KeyCode::Down => app.move_down(),
            KeyCode::Tab => {
                if mods.contains(KeyModifiers::SHIFT) {
                    app.move_up();
                } else {
                    app.move_down();
                }
            }
            KeyCode::Left => {
                app.search_cursor = move_caret(&app.search_query, app.search_cursor, -1);
            }
            KeyCode::Right => {
                app.search_cursor = move_caret(&app.search_query, app.search_cursor, 1);
            }
            KeyCode::Enter => open_cursor_row(app),
            KeyCode::Char(c)
                if !mods.contains(KeyModifiers::CONTROL) && !mods.contains(KeyModifiers::ALT) =>
            {
                app.search_cursor = insert_char_at(&mut app.search_query, app.search_cursor, c);
                app.cursor = 0;
                app.clamp_cursor();
            }
            _ => {}
        }
        return true;
    }
    match code {
        KeyCode::Char('i') => {
            app.search_input = true;
            app.search_cursor = app.search_query.chars().count();
            true
        }
        KeyCode::Esc => {
            if app.show_help {
                app.show_help = false;
            } else if app.search_query.is_empty() {
                app.leave_search();
            } else {
                app.search_query.clear();
                app.search_cursor = 0;
                app.cursor = 0;
                app.clamp_cursor();
            }
            true
        }
        KeyCode::Tab => {
            if mods.contains(KeyModifiers::SHIFT) {
                app.move_up();
            } else {
                app.move_down();
            }
            true
        }
        KeyCode::BackTab => {
            app.move_up();
            true
        }
        _ => false,
    }
}

/// Route a key to the inline section prompt. Returns true when consumed,
/// so list and add keys are blocked while the prompt is open.
fn route_key(app: &mut App, code: KeyCode) -> bool {
    if app.section_prompt.is_none() {
        return false;
    }
    section_prompt_key(app, code);
    true
}

fn section_prompt_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Esc => {
            app.section_prompt = None;
            app.clear_toast();
        }
        KeyCode::Enter => app.apply_section_prompt(),
        KeyCode::Left => {
            if let Some(p) = app.section_prompt.as_mut() {
                p.cursor = move_caret(&p.buffer, p.cursor, -1);
            }
        }
        KeyCode::Right => {
            if let Some(p) = app.section_prompt.as_mut() {
                p.cursor = move_caret(&p.buffer, p.cursor, 1);
            }
        }
        KeyCode::Backspace => {
            if let Some(p) = app.section_prompt.as_mut() {
                p.cursor = delete_before(&mut p.buffer, p.cursor);
            }
        }
        KeyCode::Delete => {
            if let Some(p) = app.section_prompt.as_mut() {
                p.cursor = delete_at(&mut p.buffer, p.cursor);
            }
        }
        KeyCode::Char(c) if !c.is_control() => {
            if let Some(p) = app.section_prompt.as_mut() {
                p.cursor = insert_char_at(&mut p.buffer, p.cursor, c);
            }
        }
        _ => {}
    }
}

/// Add-box keyboard: Desc has immediate focus; trigger chars jump to
/// the matching picker so no hotkey needs memorizing. Each picker shows
/// only its own hints (explicit + unique inputs). `←`/`→` move the caret
/// in the description and in text pickers; `Backspace`/`Delete` edit
/// around it. Trigger chars stay reserved even mid-description.
fn add_key(app: &mut App, code: KeyCode, mods: KeyModifiers) {
    let focus = app.add.focus;
    match focus {
        AddFocus::Desc => match code {
            KeyCode::Esc => {
                app.add_mode = false;
                app.add = AddDraft::fresh();
                app.clear_toast();
            }
            KeyCode::Enter => app.quick_add_commit(),
            KeyCode::Left => {
                app.add.desc_cursor = move_caret(&app.add.desc, app.add.desc_cursor, -1);
            }
            KeyCode::Right => {
                app.add.desc_cursor = move_caret(&app.add.desc, app.add.desc_cursor, 1);
            }
            KeyCode::Backspace => {
                app.add.desc_cursor = delete_before(&mut app.add.desc, app.add.desc_cursor);
            }
            KeyCode::Delete => {
                app.add.desc_cursor = delete_at(&mut app.add.desc, app.add.desc_cursor);
            }
            KeyCode::Char(c)
                if !mods.contains(KeyModifiers::CONTROL) && !mods.contains(KeyModifiers::ALT) =>
            {
                match c {
                    '#' => {
                        app.add.focus = AddFocus::Tags;
                        app.add.clear_filter();
                        app.add.picker = 0;
                    }
                    '~' => {
                        app.add.focus = AddFocus::Due;
                        app.add.clear_filter();
                        app.add.picker = 0;
                    }
                    '@' => {
                        app.add.focus = AddFocus::Project;
                        app.add.clear_filter();
                        app.add.picker = 0;
                    }
                    '!' => {
                        app.add.focus = AddFocus::Priority;
                        app.add.clear_filter();
                        app.add.picker = 0;
                    }
                    '&' => {
                        app.add.focus = AddFocus::Depends;
                        app.add.clear_filter();
                        app.add.picker = 0;
                    }
                    '*' => {
                        app.add.focus = AddFocus::Recur;
                        app.add.clear_filter();
                        app.add.picker = 0;
                    }
                    '+' => {
                        app.add.focus = AddFocus::More;
                        app.add.clear_filter();
                        app.add.picker = 0;
                    }
                    _ => {
                        app.add.desc_cursor =
                            insert_char_at(&mut app.add.desc, app.add.desc_cursor, c);
                    }
                }
            }
            _ => {}
        },
        _ => match code {
            KeyCode::Esc => {
                app.add.focus = AddFocus::Desc;
                app.add.clear_filter();
                app.add.picker = 0;
            }
            KeyCode::Enter => {
                // apply_add_suggestion owns the next focus: tags stay,
                // due/project/priority return to desc, More opens its picker.
                app.apply_add_suggestion();
            }
            KeyCode::Up => step_add_picker(app, -1),
            KeyCode::BackTab => step_add_picker(app, -1),
            KeyCode::Tab if mods.contains(KeyModifiers::SHIFT) => step_add_picker(app, -1),
            KeyCode::Down | KeyCode::Tab => step_add_picker(app, 1),
            // Fixed lists own j/k as well as arrows/Tab: priority, the
            // fixed-list depends/recur pickers, and the + more menu have no text
            // input, so j/k moves the highlight instead of typing.
            // Tags/project/due keep j/k typable for filters like `judo`.
            KeyCode::Char('j')
                if !mods.contains(KeyModifiers::CONTROL)
                    && !mods.contains(KeyModifiers::ALT)
                    && matches!(
                        focus,
                        AddFocus::Priority | AddFocus::Depends | AddFocus::Recur | AddFocus::More
                    ) =>
            {
                step_add_picker(app, 1);
            }
            KeyCode::Char('k')
                if !mods.contains(KeyModifiers::CONTROL)
                    && !mods.contains(KeyModifiers::ALT)
                    && matches!(
                        focus,
                        AddFocus::Priority | AddFocus::Depends | AddFocus::Recur | AddFocus::More
                    ) =>
            {
                step_add_picker(app, -1);
            }
            KeyCode::Left if add_filter_editable(focus) => {
                app.add.filter_cursor = move_caret(&app.add.filter, app.add.filter_cursor, -1);
            }
            KeyCode::Right if add_filter_editable(focus) => {
                app.add.filter_cursor = move_caret(&app.add.filter, app.add.filter_cursor, 1);
            }
            KeyCode::Delete if add_filter_editable(focus) => {
                app.add.filter_cursor = delete_at(&mut app.add.filter, app.add.filter_cursor);
                if focus == AddFocus::Due {
                    app.add.due = app.add.filter.clone();
                }
                app.add.picker = 0;
            }
            KeyCode::Backspace => {
                if !app.add.filter.is_empty() {
                    // A typed filter is text first: edit it, keep the chip.
                    app.add.filter_cursor =
                        delete_before(&mut app.add.filter, app.add.filter_cursor);
                    if focus == AddFocus::Due {
                        app.add.due = app.add.filter.clone();
                    }
                } else {
                    // Empty filter: Backspace removes the focused attribute.
                    app.add.filter_cursor = 0;
                    match focus {
                        AddFocus::Tags => {
                            app.add.tags.pop();
                        }
                        AddFocus::Due => app.add.due.clear(),
                        AddFocus::Project => app.add.project.clear(),
                        AddFocus::Priority => {
                            app.add.prio_set = false;
                            app.add.prio = 'M';
                        }
                        AddFocus::Depends => {
                            app.add.depends.pop();
                        }
                        AddFocus::Recur => {
                            app.add.recur.clear();
                            // Park on `no recurrence` so Enter keeps the clear.
                            app.add.picker = recur_presets().len().saturating_sub(1);
                        }
                        AddFocus::Desc | AddFocus::More => {}
                    }
                }
                if focus != AddFocus::Recur {
                    app.add.picker = 0;
                }
            }
            KeyCode::Char(c)
                if !mods.contains(KeyModifiers::CONTROL) && !mods.contains(KeyModifiers::ALT) =>
            {
                // Arrows/Tab own the list, so j/k (and every other
                // printable) types into the filter — tags like `judo`
                // or `kettle` stay typable.
                if focus == AddFocus::Priority && matches!(c, '1' | '2' | '3') {
                    app.add.prio = match c {
                        '1' => 'H',
                        '2' => 'M',
                        _ => 'L',
                    };
                    app.add.prio_set = true;
                    app.add.focus = AddFocus::Desc;
                    app.add.clear_filter();
                    app.add.picker = 0;
                } else if focus == AddFocus::Due
                    && !matches!(c, '#' | '@' | '!' | '+' | '~' | '&' | '*')
                {
                    // Typing doubles as manual date entry, Enter applies.
                    app.add.filter_cursor =
                        insert_char_at(&mut app.add.filter, app.add.filter_cursor, c);
                    app.add.due = app.add.filter.clone();
                    app.add.picker = 0;
                } else if focus == AddFocus::Tags && c == ' ' {
                    app.apply_add_suggestion();
                } else if matches!(focus, AddFocus::Depends | AddFocus::Recur)
                    && !matches!(c, '#' | '~' | '@' | '!' | '+' | '&' | '*')
                {
                    // Fixed lists: printable keys do nothing; only the
                    // trigger chars below switch pickers.
                } else if !matches!(c, '#' | '~' | '@' | '!' | '+' | '&' | '*')
                    && matches!(focus, AddFocus::Tags | AddFocus::Project)
                {
                    app.add.filter_cursor =
                        insert_char_at(&mut app.add.filter, app.add.filter_cursor, c);
                    app.add.picker = 0;
                } else {
                    match c {
                        '#' => app.add.focus = AddFocus::Tags,
                        '~' => app.add.focus = AddFocus::Due,
                        '@' => app.add.focus = AddFocus::Project,
                        '!' => app.add.focus = AddFocus::Priority,
                        '&' => app.add.focus = AddFocus::Depends,
                        '*' => app.add.focus = AddFocus::Recur,
                        '+' => app.add.focus = AddFocus::More,
                        '1' if focus == AddFocus::More => app.add.focus = AddFocus::Project,
                        '2' if focus == AddFocus::More => app.add.focus = AddFocus::Priority,
                        _ => {}
                    }
                    app.add.clear_filter();
                    app.add.picker = 0;
                }
            }
            _ => {}
        },
    }
}

/// Popup keyboard: VIEW moves with `j/k`, `e` edits the focused field only,
/// `a` adds an annotation inline, `[`/`]` selects one and `D` removes it.
/// EDIT edits that one field: `Tab`/`Shift+Tab`/`Up`/`Down` move the
/// suggestion highlight (`j`/`k` also move it on the fixed priority list
/// and the fixed-list `depends` picker, where typing is ignored),
/// `Left`/`Right` move the text cursor (not in `depends`), `Backspace`/
/// `Delete` edit around it,
/// `Enter` applies the highlight (tags replace the last token,
/// `depends` appends it) then saves, `Backspace` in `depends` pops the
/// last link and saves (staying in EDIT), `Esc` discards only this field
/// back to VIEW. `d` toggles done.
fn popup_key(app: &mut App, code: KeyCode, mods: KeyModifiers) {
    let mode = match app.popup.as_ref() {
        Some(p) => p.mode,
        None => return,
    };
    // VIEW mode: field navigation plus an interactive `annotation` row.
    // `a` types a new note inline (`Enter` annotates, `esc` cancels);
    // `[`/`]` selects a note and `D`/`x` denotates it.
    if mode == PopupMode::View {
        let annotating = app.popup.as_ref().is_some_and(|p| p.annotating);
        if annotating {
            match code {
                KeyCode::Esc => {
                    if let Some(p) = app.popup.as_mut() {
                        p.annotating = false;
                        p.annotation_draft.clear();
                        p.annotation_cursor = 0;
                    }
                    app.set_toast("edits discarded".into());
                }
                KeyCode::Enter => app.annotate_current(),
                KeyCode::Left => {
                    if let Some(p) = app.popup.as_mut() {
                        p.annotation_cursor =
                            move_caret(&p.annotation_draft, p.annotation_cursor, -1);
                    }
                }
                KeyCode::Right => {
                    if let Some(p) = app.popup.as_mut() {
                        p.annotation_cursor =
                            move_caret(&p.annotation_draft, p.annotation_cursor, 1);
                    }
                }
                KeyCode::Backspace => {
                    if let Some(p) = app.popup.as_mut() {
                        let at = p.annotation_cursor;
                        p.annotation_cursor = delete_before(&mut p.annotation_draft, at);
                    }
                }
                KeyCode::Delete => {
                    if let Some(p) = app.popup.as_mut() {
                        let at = p.annotation_cursor;
                        p.annotation_cursor = delete_at(&mut p.annotation_draft, at);
                    }
                }
                KeyCode::Char(c)
                    if !mods.contains(KeyModifiers::CONTROL)
                        && !mods.contains(KeyModifiers::ALT) =>
                {
                    if let Some(p) = app.popup.as_mut() {
                        let at = p.annotation_cursor;
                        p.annotation_cursor = insert_char_at(&mut p.annotation_draft, at, c);
                    }
                }
                _ => {}
            }
            return;
        }
        match code {
            KeyCode::Esc | KeyCode::Char('q') => {
                app.popup = None;
                app.popup_list_focus = false;
                app.clear_toast();
            }
            KeyCode::Char('e' | 'i') => {
                if let Some(p) = app.popup.as_mut() {
                    p.annotating = false;
                    p.enter_edit();
                }
            }
            // `h`/`Left` shift focus to the task list; the inspector stays
            // open behind it (`i`/`Enter` focus back in, `Enter`/`esc` close).
            KeyCode::Char('h') | KeyCode::Left => {
                app.popup_list_focus = true;
            }
            KeyCode::Char('E') => app.set_toast("opened $EDITOR".into()),
            KeyCode::Char('d') => app.toggle_popup_done(),
            KeyCode::Char('a') => {
                if let Some(p) = app.popup.as_mut() {
                    p.annotating = true;
                    p.annotation_draft.clear();
                    p.annotation_cursor = 0;
                }
            }
            KeyCode::Char('D' | 'x') => app.denotate_selected(),
            KeyCode::Char('[') => app.cycle_annotation(-1),
            KeyCode::Char(']') => app.cycle_annotation(1),
            KeyCode::Char('j') | KeyCode::Down => {
                if let Some(p) = app.popup.as_mut() {
                    p.move_field(1);
                }
            }
            // `k` moves back to the previous field, so selection control
            // stays on j/k; `h` hands focus to the task list instead.
            // Tab always toggles views, even with the inspector open.
            // (EDIT mode keeps its own Tab picker navigation below.)
            KeyCode::Char('k') | KeyCode::Up => {
                if let Some(p) = app.popup.as_mut() {
                    p.move_field(-1);
                }
            }
            // Enter toggles the inspector closed (it opens from the list),
            // so Enter/Esc both show/hide around the same row.
            KeyCode::Enter => {
                app.popup = None;
                app.popup_list_focus = false;
                app.clear_toast();
            }
            // `/` keeps the inspector open and focuses Search input.
            KeyCode::Char('/') => app.open_search(),
            // Tab/Shift+Tab toggle views even with the inspector open.
            KeyCode::Tab => app.cycle_view(1),
            KeyCode::BackTab => app.cycle_view(-1),
            _ => {}
        }
        return;
    }
    // EDIT mode: single-field edit. Tab/Up/Down move the highlight only.
    // `depends`/`recur`/priority also use j/k for the dropdown.
    match code {
        // Esc discards only the focused field, then exits to VIEW.
        KeyCode::Esc => {
            if app.revert_popup_field() {
                app.set_toast("edits discarded".into());
            }
        }
        // Enter applies the highlight (if any) then saves back to VIEW;
        // popup stays open. `depends` appends the highlighted candidate
        // (`no dependencies` clears), then the draft saves at once.
        KeyCode::Enter => {
            let is_depends = app.popup.as_ref().is_some_and(|p| p.field == F_DEPENDS);
            if is_depends {
                let should_apply = match app.popup.as_ref() {
                    Some(p) => !popup_suggestions(app, p).is_empty(),
                    None => false,
                };
                if should_apply && !app.apply_depends_suggestion() {
                    return;
                }
            } else if match app.popup.as_ref() {
                Some(p) => !popup_suggestions(app, p).is_empty(),
                None => false,
            } {
                app.apply_popup_suggestion();
            }
            app.save_popup();
        }
        KeyCode::Char('j')
            if app.popup.as_ref().is_some_and(|p| {
                p.field == F_DEPENDS || p.field == F_PRIO || p.field == F_RECUR
            }) =>
        {
            step_popup_suggest(app, 1);
        }
        KeyCode::Char('k')
            if app.popup.as_ref().is_some_and(|p| {
                p.field == F_DEPENDS || p.field == F_PRIO || p.field == F_RECUR
            }) =>
        {
            step_popup_suggest(app, -1);
        }
        KeyCode::BackTab => step_popup_suggest(app, -1),
        KeyCode::Tab if mods.contains(KeyModifiers::SHIFT) => step_popup_suggest(app, -1),
        KeyCode::Tab => step_popup_suggest(app, 1),
        KeyCode::Up => step_popup_suggest(app, -1),
        KeyCode::Down => step_popup_suggest(app, 1),
        // Left/right move the text cursor inside the focused field.
        // Fixed-list `depends`/`recur` have no cursor: nothing to move.
        KeyCode::Left => {
            if app
                .popup
                .as_ref()
                .is_some_and(|p| p.field != F_DEPENDS && p.field != F_RECUR)
            {
                if let Some(p) = app.popup.as_mut() {
                    p.move_cursor(-1);
                }
            }
        }
        KeyCode::Right => {
            if app
                .popup
                .as_ref()
                .is_some_and(|p| p.field != F_DEPENDS && p.field != F_RECUR)
            {
                if let Some(p) = app.popup.as_mut() {
                    p.move_cursor(1);
                }
            }
        }
        // Fixed-list `depends`: Backspace pops the last link (add-box
        // parity) and saves at once — Enter always appends, so it cannot
        // commit a removal. Stays in EDIT for repeated presses; empty
        // drafts do nothing.
        KeyCode::Backspace => {
            if app.popup.as_ref().is_some_and(|p| p.field == F_DEPENDS) {
                let popped = match app.popup.as_mut() {
                    Some(p) => {
                        let mut toks = parse_depends_tokens(&p.depends);
                        if toks.pop().is_none() {
                            false
                        } else {
                            p.depends = toks.join(", ");
                            p.dirty = true;
                            p.suggest_idx = 0;
                            true
                        }
                    }
                    None => false,
                };
                if popped {
                    app.save_popup();
                    if let Some(p) = app.popup.as_mut() {
                        if p.mode == PopupMode::View {
                            p.mode = PopupMode::Edit;
                            p.suggest_idx = 0;
                            let field = p.field;
                            p.edit_cursor = p.field_len(field);
                        }
                    }
                }
            } else if app.popup.as_ref().is_some_and(|p| p.field == F_RECUR) {
                // Fixed-list `recur`: Backspace clears the draft and parks
                // the highlight on `no recurrence` so a following Enter
                // keeps the clear instead of reapplying the first preset.
                if let Some(p) = app.popup.as_mut() {
                    if !p.recur.is_empty() {
                        p.recur.clear();
                        p.dirty = true;
                        p.suggest_idx = recur_presets().len().saturating_sub(1);
                    }
                }
            } else if let Some(p) = app.popup.as_mut() {
                // Text fields only: `priority` cycles with `[`/`]` and has
                // no text caret, so Backspace leaves it alone.
                if p.field == F_PRIO {
                    return;
                }
                let field = p.field;
                let before = popup_field_text(p, field);
                popup_backspace(p);
                // No-op edits (e.g. Backspace at column 0) leave no unsaved
                // trace, so a later `esc` stays clean.
                if popup_field_text(p, field) != before {
                    p.dirty = true;
                }
                p.suggest_idx = 0;
            }
        }
        // Delete removes the char after the caret; fixed-list `depends`/`recur`
        // and cycling `priority` have no text caret.
        KeyCode::Delete => {
            if app
                .popup
                .as_ref()
                .is_some_and(|p| p.field != F_DEPENDS && p.field != F_RECUR && p.field != F_PRIO)
            {
                if let Some(p) = app.popup.as_mut() {
                    let field = p.field;
                    let before = popup_field_text(p, field);
                    popup_delete(p);
                    if popup_field_text(p, field) != before {
                        p.dirty = true;
                    }
                    p.suggest_idx = 0;
                }
            }
        }
        KeyCode::Char('E') => {
            app.set_toast("opened $EDITOR".into());
        }
        KeyCode::Char(c)
            if !mods.contains(KeyModifiers::CONTROL) && !mods.contains(KeyModifiers::ALT) =>
        {
            let field = app.popup.as_ref().map(|p| p.field).unwrap_or(0);
            if field == F_DEPENDS {
                // Fixed list: no manual typing; the highlight owns the pick.
                return;
            }
            if field == F_RECUR && c != '[' && c != ']' {
                // Fixed list: no manual typing; `[`/`]` cycles below.
                return;
            }
            if c == '[' || c == ']' {
                match field {
                    F_PRIO => {
                        if let Some(p) = app.popup.as_mut() {
                            p.prio = if c == ']' {
                                match p.prio {
                                    'H' => 'M',
                                    'M' => 'L',
                                    _ => 'H',
                                }
                            } else {
                                match p.prio {
                                    'H' => 'L',
                                    'L' => 'M',
                                    _ => 'H',
                                }
                            };
                            p.dirty = true;
                        }
                    }
                    F_PROJECT => {
                        let mut opts = all_projects(app);
                        opts.insert(0, CTX_CLEAR_PROJECT.into());
                        if let Some(p) = app.popup.as_mut() {
                            let cur = opts.iter().position(|o| *o == p.project).unwrap_or(0);
                            let next = if c == ']' {
                                (cur + 1) % opts.len()
                            } else {
                                (cur + opts.len() - 1) % opts.len()
                            };
                            let pick = opts[next].clone();
                            let next_val = if pick == CTX_CLEAR_PROJECT {
                                String::new()
                            } else {
                                pick
                            };
                            // Cycling a single-option list is a no-op: it
                            // leaves no unsaved trace.
                            if p.project != next_val {
                                p.project = next_val;
                                p.dirty = true;
                                let field = p.field;
                                p.edit_cursor = p.field_len(field);
                            }
                        }
                    }
                    F_RECUR => {
                        // Cycle `"" → daily → weekly → monthly → yearly → ""`.
                        let order = ["", "daily", "weekly", "monthly", "yearly"];
                        if let Some(p) = app.popup.as_mut() {
                            let cur = order
                                .iter()
                                .position(|v| p.recur.trim().to_lowercase() == v.to_lowercase())
                                .unwrap_or(0);
                            let next = if c == ']' {
                                (cur + 1) % order.len()
                            } else {
                                (cur + order.len() - 1) % order.len()
                            };
                            p.recur = order[next].to_string();
                            p.dirty = true;
                            let field = p.field;
                            p.edit_cursor = p.field_len(field);
                        }
                    }
                    _ => {
                        if let Some(p) = app.popup.as_mut() {
                            popup_insert(p, c);
                            p.dirty = true;
                            p.suggest_idx = 0;
                        }
                    }
                }
            } else {
                match field {
                    F_PRIO | F_RECUR => {}
                    _ => {
                        if let Some(p) = app.popup.as_mut() {
                            popup_insert(p, c);
                            p.dirty = true;
                            p.suggest_idx = 0;
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

fn popup_field_mut(p: &mut Popup) -> &mut String {
    match p.field {
        0 => &mut p.desc,
        1 => &mut p.project,
        2 => &mut p.due,
        4 => &mut p.tags,
        5 => &mut p.depends,
        _ => &mut p.desc,
    }
}

/// Byte index of char offset `at`, clamped to the end of `text`.
fn char_byte_index(text: &str, at: usize) -> usize {
    text.char_indices()
        .nth(at)
        .map(|(byte, _)| byte)
        .unwrap_or(text.len())
}

/// Insert `c` at char offset `at`; returns the caret after the insert.
fn insert_char_at(text: &mut String, at: usize, c: char) -> usize {
    let at = at.min(text.chars().count());
    let byte = char_byte_index(text, at);
    text.insert(byte, c);
    at + 1
}

/// Delete the char before `at`; returns the caret after the delete.
fn delete_before(text: &mut String, at: usize) -> usize {
    let at = at.min(text.chars().count());
    if at == 0 {
        return 0;
    }
    let start = char_byte_index(text, at - 1);
    let end = char_byte_index(text, at);
    text.replace_range(start..end, "");
    at - 1
}

/// Delete the char at `at`; returns the clamped caret (unchanged).
fn delete_at(text: &mut String, at: usize) -> usize {
    let at = at.min(text.chars().count());
    if at == text.chars().count() {
        return at;
    }
    let start = char_byte_index(text, at);
    let end = char_byte_index(text, at + 1);
    text.replace_range(start..end, "");
    at
}

/// Move a char caret by `delta`, clamped into `text`.
fn move_caret(text: &str, at: usize, delta: isize) -> usize {
    let len = text.chars().count() as isize;
    (at.min(len as usize) as isize + delta).clamp(0, len) as usize
}

/// Display width of the first `at` chars of `text`.
fn caret_width(text: &str, at: usize) -> usize {
    text_width(&text.chars().take(at).collect::<String>())
}

/// Visible window of a typed input so long text scrolls instead of
/// overflowing its box. Returns the visible slice and the caret's cell
/// offset inside it. The window keeps the caret visible: it shows the head
/// until the caret runs past `max`, then scrolls just enough to pin the
/// caret at the right edge.
fn visible_input(text: &str, caret: usize, max: usize) -> (String, usize) {
    if max == 0 {
        return (String::new(), 0);
    }
    let chars: Vec<char> = text.chars().collect();
    let caret = caret.min(chars.len());
    let widths: Vec<usize> = chars.iter().map(|c| text_width(&c.to_string())).collect();
    let total: usize = widths.iter().sum();
    if total <= max {
        return (text.to_string(), caret_width(text, caret));
    }
    // Advance the start until the caret fits inside the window.
    let mut start = 0;
    while start < caret {
        let w: usize = widths.iter().skip(start).take(caret - start).sum();
        if w < max {
            break;
        }
        start += 1;
    }
    // Extend the end to fill the window.
    let mut used = 0;
    let mut end = start;
    while end < chars.len() {
        if used + widths[end] > max {
            break;
        }
        used += widths[end];
        end += 1;
    }
    let visible: String = chars[start..end].iter().collect();
    let caret_x: usize = widths.iter().skip(start).take(caret - start).sum();
    (visible, caret_x.min(max.saturating_sub(1)))
}

/// Insert a char at the EDIT text cursor (char-based, end-clamped).
fn popup_insert(p: &mut Popup, c: char) {
    p.clamp_cursor();
    let at = p.edit_cursor;
    let next = insert_char_at(popup_field_mut(p), at, c);
    p.edit_cursor = next;
}

/// Delete the char before the EDIT text cursor.
fn popup_backspace(p: &mut Popup) {
    p.clamp_cursor();
    let at = p.edit_cursor;
    let next = delete_before(popup_field_mut(p), at);
    p.edit_cursor = next;
}

/// Delete the char after the EDIT text cursor.
fn popup_delete(p: &mut Popup) {
    p.clamp_cursor();
    let at = p.edit_cursor;
    let next = delete_at(popup_field_mut(p), at);
    p.edit_cursor = next;
}

// ---------------------------------------------------------------------------
// rendering — tab bar, two-line rows, and the split inspector
// ---------------------------------------------------------------------------

fn task_tags_text(task: &Task) -> String {
    task.tags
        .iter()
        .map(|tag| format!("#{tag}"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Urgency label: whole numbers stay bare, fractions keep one decimal.
fn urgency_label(urgency: f32) -> String {
    if (urgency.fract()).abs() < 0.05 {
        format!("{urgency:.0}")
    } else {
        format!("{urgency:.1}")
    }
}

/// Colored due span when a task carries a real due label.
fn due_span(task: &Task) -> Option<Span<'static>> {
    if task.due.is_empty() || task.due == "—" {
        return None;
    }
    let due_color = if is_overdue(task) {
        RED
    } else if is_due_today(task) {
        YELLOW
    } else {
        INK2
    };
    Some(Span::styled(
        task.due.clone(),
        Style::default().fg(due_color),
    ))
}

/// Secondary metadata for a row: id · due · project · tags · recurrence · blocked by.
/// Project names use the deterministic palette; an absent project or tag
/// simply drops its segment instead of leaving a placeholder.
/// Absent due leaves no trace: no dash, no leading separator. Separators sit
/// only between present segments. Blocked tasks show `blocked by <ids>`
/// with the short display IDs of their open blockers.
fn task_meta_spans(task: &Task, app: &App, hide_project: bool) -> Vec<Span<'static>> {
    let mut spans: Vec<Span<'static>> = vec![Span::styled(
        task_id_display(task),
        Style::default().fg(DIM),
    )];
    if let Some(due) = due_span(task) {
        spans.push(Span::styled("  ·  ", Style::default().fg(DIM)));
        spans.push(due);
    }
    if !hide_project && !task.project.is_empty() {
        if !spans.is_empty() {
            spans.push(Span::styled("  ·  ", Style::default().fg(DIM)));
        }
        spans.push(Span::styled(
            task.project.clone(),
            Style::default().fg(app.project_color(&task.project)),
        ));
    }
    let tags = task_tags_text(task);
    if !tags.is_empty() {
        if !spans.is_empty() {
            spans.push(Span::styled("  ·  ", Style::default().fg(DIM)));
        }
        spans.push(Span::styled(tags, Style::default().fg(CYAN)));
    }
    if is_recurring(task) {
        if !spans.is_empty() {
            spans.push(Span::styled("  ·  ", Style::default().fg(DIM)));
        }
        spans.push(Span::styled(
            task.recur.trim().to_string(),
            Style::default().fg(BLUE),
        ));
    }
    let mut blockers: Vec<String> = app
        .blocked_by(&task.uuid)
        .iter()
        .map(|t| task_id_display(t))
        .collect();
    blockers.sort();
    if !blockers.is_empty() {
        if !spans.is_empty() {
            spans.push(Span::styled("  ·  ", Style::default().fg(DIM)));
        }
        spans.push(Span::styled(
            format!("blocked by {}", blockers.join(",")),
            Style::default().fg(RED),
        ));
    }
    spans
}

/// Compact single-row relation: ` ID  desc · due`.
/// Both BLOCKS and BLOCKED BY share it so the two sections stay identical.
/// The ID column uses the shared `rel_id_width` and the description is
/// padded so the due column starts at the same cell on every row.
/// The description shrinks with `…` so due stays visible;
/// the whole row never exceeds `width` and never wraps.
fn rel_row(rel: &Task, width: usize, rel_id_width: usize) -> Line<'static> {
    let due = due_span(rel);
    let due_w = due.as_ref().map(Span::width).unwrap_or(0);
    // Leading blank + right-aligned ID + two spaces before the description.
    let prefix_w = 1 + rel_id_width + 2;
    let gap_w = if due.is_some() { 5 } else { 0 };
    let desc_w = width.saturating_sub(prefix_w + gap_w + due_w).max(1);
    let desc = fit(&rel.desc, desc_w);
    let pad = desc_w.saturating_sub(text_width(&desc));
    let mut spans = vec![
        Span::raw(" "),
        Span::styled(
            format!("{:>width$}", task_id_display(rel), width = rel_id_width),
            Style::default().fg(DIM),
        ),
        Span::raw("  "),
        Span::styled(desc, Style::default().fg(INK2)),
    ];
    if let Some(d) = due {
        if pad > 0 {
            spans.push(Span::raw(" ".repeat(pad)));
        }
        spans.push(Span::styled("  ·  ", Style::default().fg(DIM)));
        spans.push(d);
    }
    Line::from(spans)
}

/// Left accent for the row: the cursor wins, a multi-selected row uses the
/// softer bar, and unselected rows keep the cell blank for alignment.
fn row_bar(cursor: bool, selected: bool) -> Span<'static> {
    if cursor {
        Span::styled("▎", Style::default().fg(CYAN))
    } else if selected {
        Span::styled("▌", Style::default().fg(BLUE))
    } else {
        Span::raw(" ")
    }
}

/// Two-line task row: priority dot, start indicator, description,
/// and right-aligned urgency on the first line; metadata on the second.
fn task_row(
    task: &Task,
    app: &App,
    width: usize,
    cursor: bool,
    selected: bool,
    hide_project: bool,
) -> Vec<Line<'static>> {
    let priority = prio_label(task.prio);
    // Left-hugging prefix: bar · priority dot · start · space.
    // `● desc`: the description starts at a fixed column for every row.
    let blocked = !app.blocked_by(&task.uuid).is_empty();
    let prefix_w = 4;
    let urgency = urgency_label(task.urgency);
    let urgency_w = urgency.chars().count();
    let gap = 2.min(width.saturating_sub(prefix_w + urgency_w));
    let desc_w = width.saturating_sub(prefix_w + urgency_w + gap);
    let desc = fit(&task.desc, desc_w.max(1));
    let desc_width = text_width(&desc);
    let desc_color = if cursor {
        BRIGHT
    } else if blocked {
        DIM
    } else {
        INK
    };
    let mut first = vec![
        row_bar(cursor, selected),
        Span::styled("●", Style::default().fg(prio_color(priority))),
        Span::styled(
            if task.active { "◉" } else { " " },
            Style::default().fg(MINT),
        ),
        Span::raw(" "),
        Span::styled(
            desc,
            Style::default().fg(desc_color).add_modifier(if cursor {
                Modifier::BOLD
            } else {
                Modifier::empty()
            }),
        ),
    ];
    let used = prefix_w + desc_width + urgency_w;
    first.push(Span::raw(" ".repeat(width.saturating_sub(used))));
    first.push(Span::styled(
        urgency,
        Style::default().fg(if cursor { CYAN } else { DIM }),
    ));
    // Second line sits under the description column: the full prefix
    // width of spaces, then metadata with no leading separator.
    let mut second = vec![Span::raw(" ".repeat(prefix_w))];
    second.extend(task_meta_spans(task, app, hide_project));
    vec![Line::from(first), Line::from(second)]
}

/// Dimmed completed row: check + struck-through description.
fn completed_row(task: &Task, width: usize, cursor: bool, selected: bool) -> Line<'static> {
    // bar · check · two spaces, matching the open-row description column.
    let prefix_w = 4;
    let desc_w = width.saturating_sub(prefix_w).max(1);
    Line::from(vec![
        row_bar(cursor, selected),
        Span::styled("✓", Style::default().fg(LINE)),
        Span::raw("  "),
        Span::styled(
            fit(&task.desc, desc_w),
            Style::default()
                .fg(if cursor { INK2 } else { DIM })
                .add_modifier(Modifier::CROSSED_OUT),
        ),
    ])
}

/// Display width of `text` in terminal cells (wide glyphs count twice).
fn text_width(text: &str) -> usize {
    Span::raw(text).width()
}

/// Put the real terminal cursor on a typing caret. `x` is a cell offset
/// from `area.x`; `y` is the absolute row. The caret is clamped inside
/// `area`, so long text clips instead of pushing the cursor past the pane.
fn place_cursor(f: &mut ratatui::Frame, area: Rect, x: usize, y: usize) {
    if area.is_empty() || y < area.y as usize || y >= area.bottom() as usize {
        return;
    }
    let x = (area.x as usize + x).min(area.right() as usize - 1);
    f.set_cursor_position((x as u16, y as u16));
}

/// Minimum width for the side-by-side inspector. Narrower terminals use a
/// centered overlay so the list never collapses.
const SPLIT_MIN_WIDTH: u16 = 100;
/// Width of the inspector pane in the split layout.
const INSPECTOR_WIDTH: u16 = 54;
/// Blank gutter between the list and the inspector.
const INSPECTOR_GAP: u16 = 6;

fn spans_width(spans: &[Span<'_>]) -> usize {
    spans.iter().map(Span::width).sum()
}

/// Fit a row's spans into exactly `max` cells: truncate the tail when too
/// long, pad with spaces when too short. Keeps the add-box `│` borders in a
/// fixed column so long chips can never push the right border off-screen.
fn fit_spans(spans: Vec<Span<'static>>, max: usize) -> Vec<Span<'static>> {
    let total = spans_width(&spans);
    if total == max {
        return spans;
    }
    if total < max {
        let mut out = spans;
        out.push(Span::raw(" ".repeat(max - total)));
        return out;
    }
    let mut out = spans;
    let mut excess = total - max;
    while excess > 0 {
        let Some(last) = out.pop() else {
            break;
        };
        let w = last.width();
        if w <= excess {
            excess -= w;
            continue;
        }
        let keep_w = w - excess;
        let text = last.content.to_string();
        let kept: String = text.chars().take(keep_w).collect();
        let style = last.style;
        out.push(Span::styled(kept, style));
        break;
    }
    out
}

/// A line with `left` at the start and `right` flush against `width`.
fn join_right(left: Vec<Span<'static>>, right: Vec<Span<'static>>, width: usize) -> Line<'static> {
    let mut spans = left;
    let gap = width.saturating_sub(spans_width(&spans) + spans_width(&right));
    spans.push(Span::raw(" ".repeat(gap)));
    spans.extend(right);
    Line::from(spans)
}

fn rule_line(width: usize, color: Color) -> Line<'static> {
    Line::from(Span::styled("─".repeat(width), Style::default().fg(color)))
}

fn render(f: &mut ratatui::Frame, app: &App) {
    let area = f.area();
    f.render_widget(Block::default().style(Style::default().bg(BG)), area);
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .split(area);

    render_header(f, rows[0], app);

    // The inspector is a split pane on normal terminals and a centered
    // overlay on narrow ones, so the list always keeps a usable width.
    let split = app.popup.is_some() && area.width >= SPLIT_MIN_WIDTH;
    if split {
        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(30),
                Constraint::Length(INSPECTOR_GAP),
                Constraint::Length(INSPECTOR_WIDTH),
                Constraint::Length(2),
            ])
            .split(rows[1]);
        render_list(f, body[0], app);
        if let Some(p) = &app.popup {
            render_inspector(f, body[2], p, app);
        }
    } else {
        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(30), Constraint::Length(2)])
            .split(rows[1]);
        render_list(f, body[0], app);
    }

    render_footer(f, rows[2], app);

    if !split {
        if let Some(p) = &app.popup {
            render_popup(f, rows[1], p, app);
        }
    }
    if app.show_help {
        render_help(f, area);
    }
}

struct HeaderTab {
    label: String,
    count: usize,
    active: bool,
}

impl HeaderTab {
    fn width(&self) -> usize {
        text_width(&self.label) + self.count.to_string().chars().count() + 3
    }

    fn spans(&self) -> Vec<Span<'static>> {
        if self.active {
            vec![Span::styled(
                format!(" {} {} ", self.label, self.count),
                Style::default()
                    .fg(CYAN)
                    .bg(TINT)
                    .add_modifier(Modifier::BOLD),
            )]
        } else {
            vec![
                Span::styled(format!(" {} ", self.label), Style::default().fg(INK2)),
                Span::styled(format!("{} ", self.count), Style::default().fg(DIM)),
            ]
        }
    }
}

/// One top bar row plus the hairline beneath it. Inbox, Today, All stay pinned
/// on the left; only project tabs scroll, with the window following the
/// active project and `‹`/`›` marking hidden neighbours. The current stats
/// stay pinned on the right. Search shows a `SEARCH` pill where the wordmark was.
fn render_header(f: &mut ratatui::Frame, area: Rect, app: &App) {
    let width = area.width as usize;

    let mut tabs = vec![
        HeaderTab {
            label: "Inbox".into(),
            count: app.count_for(&View::Inbox),
            active: app.view == View::Inbox,
        },
        HeaderTab {
            label: "Today".into(),
            count: app.count_for(&View::Today),
            active: app.view == View::Today,
        },
        HeaderTab {
            label: "All".into(),
            count: app.count_for(&View::All),
            active: app.view == View::All,
        },
    ];
    for project in &app.projects {
        tabs.push(HeaderTab {
            label: fit(project, 18),
            count: app.count_for(&View::Project(project.clone())),
            active: matches!(&app.view, View::Project(p) if p == project),
        });
    }

    let fixed: &[HeaderTab] = &tabs[..3.min(tabs.len())];
    let projects: &[HeaderTab] = if tabs.len() > 3 { &tabs[3..] } else { &[] };
    let fixed_w: usize = fixed.iter().map(HeaderTab::width).sum();
    let projects_w: usize = projects.iter().map(HeaderTab::width).sum();
    // Reserve the pinned tabs plus the active project (and chevrons) first,
    // so optional stats (`active`, sync) yield instead of crowding them out.
    // `open`/`due` lead the stats list, so they survive the squeeze.
    let active_proj_w = projects
        .iter()
        .find(|t| t.active)
        .map(HeaderTab::width)
        .unwrap_or(0);
    let stats_limit = width
        .saturating_sub(fixed_w)
        .saturating_sub(active_proj_w)
        .saturating_sub(5);
    // Pinned right stats so tabs never push `open`/`due` out.
    let (open, due, active) = app.header_stats();
    let sync = match app.last_sync {
        Some(t) if app.sync_ok => format!("synced {}", t.format("%H:%M")),
        Some(_) => "sync failed".to_string(),
        None => "not synced".to_string(),
    };
    let sync_color = if app.sync_ok { DIM } else { YELLOW };
    let stats: Vec<(String, Color)> = vec![
        (format!("{open} open"), INK),
        (format!("{due} due"), CYAN),
        (format!("{active} active"), DIM),
        (sync, sync_color),
    ];
    let mut right: Vec<Span<'static>> = Vec::new();
    for (text, color) in &stats {
        let sep = if right.is_empty() { "" } else { " · " };
        let candidate = spans_width(&right) + sep.len() + text.chars().count() + 1;
        if candidate > width.min(stats_limit) {
            break;
        }
        if !right.is_empty() {
            right.push(Span::styled(" · ", Style::default().fg(DIM)));
        }
        right.push(Span::styled(text.clone(), Style::default().fg(*color)));
    }
    let right_w = spans_width(&right);

    let mut left: Vec<Span<'static>> = Vec::new();
    if app.view == View::Search {
        let n = app.open_rows().len();
        left.push(Span::styled(
            format!(" SEARCH {n} "),
            Style::default()
                .fg(CYAN)
                .bg(TINT)
                .add_modifier(Modifier::BOLD),
        ));
    }

    let left_used = spans_width(&left);
    let budget = width.saturating_sub(right_w + 1).saturating_sub(left_used);
    // Inbox, Today, All stay pinned; only project tabs scroll.
    if fixed_w + projects_w + 4 <= budget {
        for tab in fixed {
            left.extend(tab.spans());
        }
        for tab in projects {
            left.extend(tab.spans());
        }
    } else {
        let mut proj_budget = budget;
        for tab in fixed {
            if proj_budget < tab.width() {
                break;
            }
            proj_budget -= tab.width();
            left.extend(tab.spans());
        }
        if !projects.is_empty() {
            // Window follows the active project; `‹`/`›` mark hidden ones.
            // A fixed tab (or Search) being active pins the window at start.
            let active_proj = projects.iter().position(|t| t.active).unwrap_or(0);
            let cap = proj_budget.saturating_sub(4);
            let mut start = active_proj;
            let mut end = active_proj + 1;
            let mut used = projects.get(active_proj).map(HeaderTab::width).unwrap_or(0);
            while end < projects.len() && used + projects[end].width() <= cap {
                used += projects[end].width();
                end += 1;
            }
            while start > 0 && used + projects[start - 1].width() <= cap {
                used += projects[start - 1].width();
                start -= 1;
            }
            if start > 0 {
                left.push(Span::styled("‹ ", Style::default().fg(DIM)));
            }
            for tab in &projects[start..end] {
                left.extend(tab.spans());
            }
            if end < projects.len() {
                left.push(Span::styled(" ›", Style::default().fg(DIM)));
            }
        }
    }

    f.render_widget(
        Paragraph::new(join_right(left, right, width)).style(Style::default().bg(BG)),
        Rect::new(area.x, area.y, area.width, 1),
    );
    f.render_widget(
        Paragraph::new(rule_line(width, LINE)).style(Style::default().bg(BG)),
        Rect::new(area.x, area.y + 1, area.width, 1),
    );
}

/// Contextual hints and the cursor position. A toast or a
/// pending action temporarily replaces the key hints so it stays visible.
fn render_footer(f: &mut ratatui::Frame, area: Rect, app: &App) {
    let width = area.width as usize;
    f.render_widget(
        Paragraph::new(rule_line(width, LINE)).style(Style::default().bg(BG)),
        Rect::new(area.x, area.y, area.width, 1),
    );

    let mut spans = vec![Span::raw(" ")];

    let mut hint_color = DIM;
    let position = app
        .cursor_id()
        .and_then(|id| {
            let rows = app.open_rows();
            rows.iter()
                .position(|x| x == &id)
                .map(|i| (i + 1, rows.len()))
        })
        .map(|(i, n)| format!("{i} / {n}"))
        .unwrap_or_default();
    // Section prompt buffer scrolls so long names keep the caret on-screen.
    let section_visible: Option<(String, usize)> = app.section_prompt.as_ref().map(|p| {
        let prefix = section_prompt_prefix(p);
        let suffix = " · Enter save · esc cancel";
        let left_fixed = 1;
        let max_buffer = width
            .saturating_sub(left_fixed + text_width(&prefix) + text_width(suffix) + position.len())
            .max(1);
        let (visible, caret_x) = visible_input(&p.buffer, p.cursor, max_buffer);
        (format!("{prefix}{visible}{suffix}"), caret_x)
    });
    let hint = if let Some(visible_hint) = section_visible.as_ref().map(|(h, _)| h.clone()) {
        visible_hint
    } else if app.confirm_delete.is_some() {
        hint_color = RED;
        "Delete? y yes · n no".to_string()
    } else if !app.selected.is_empty() {
        hint_color = YELLOW;
        format!(
            "{} selected · D delete · m move · t tags · d done",
            app.selected.len()
        )
    } else if !app.toast.is_empty() {
        hint_color = MINT;
        app.toast.clone()
    } else if let Some(p) = &app.popup {
        match p.mode {
            PopupMode::View if p.annotating => {
                "NOTE · type · ←/→ cursor · Enter add · esc cancel".to_string()
            }
            PopupMode::View => {
                if app.popup_list_focus {
                    // Same hotkeys as the main list; only focus/close differ:
                    // `i` hands focus back to the inspector, `esc`/`q` close
                    // it instead of quitting.
                    if width >= 132 {
                        "LIST · j/k move · i inspector · a new · d done · s start · D delete · u undo · / search · ? help · esc close".to_string()
                    } else {
                        "LIST · j/k move · i inspector · a new · d done · s start · D delete · u undo · / search · esc close".to_string()
                    }
                } else {
                    "j/k move · h list · e edit · d done · a note · D del note · Enter/esc close"
                        .to_string()
                }
            }
            PopupMode::Edit if p.field == F_DEPENDS => {
                "j/k/↑/↓/Tab choose · Enter append+save · ⌫ remove · esc discard".to_string()
            }
            PopupMode::Edit => {
                "type · Tab/↑/↓ choose · ←/→ cursor · Enter save · esc discard".to_string()
            }
        }
    } else if app.add_mode {
        match app.add.focus {
            AddFocus::Desc => {
                "ADD desc · type · ←/→ cursor · Enter add · esc done · + more".to_string()
            }
            AddFocus::Tags => {
                "ADD tags · type · ←/→ cursor · ↑/↓/Tab pick · Enter toggle · esc back"
                    .to_string()
            }
            AddFocus::Due => {
                "ADD due · type · ←/→ cursor · ↑/↓/Tab pick · Enter apply · esc back".to_string()
            }
            AddFocus::Project => {
                "ADD project · type · ←/→ cursor · ↑/↓/Tab pick · Enter apply · esc back"
                    .to_string()
            }
            AddFocus::Priority => {
                "ADD priority · j/k/↑/↓/Tab pick · 1/2/3 · Enter apply · esc back".to_string()
            }
            AddFocus::Depends => {
                "ADD depends · j/k/↑/↓/Tab pick · Enter append · esc back".to_string()
            }
            AddFocus::Recur => {
                "ADD recur · j/k/↑/↓/Tab pick · Enter apply · esc back".to_string()
            }
            AddFocus::More => {
                "ADD more · @ project · ! priority · # tags · ~ due · & depends · * recur · esc back"
                    .to_string()
            }
        }
    } else if app.view == View::Search {
        if app.search_input {
            "SEARCH input · type · ←/→ cursor · Tab/↑/↓ move · Enter open · esc back".to_string()
        } else {
            "SEARCH · i input · j/k move · Enter open · esc clear/back".to_string()
        }
    } else if width >= 132 {
        "j/k move · i open · h close · a new · d done · s start · D delete · u undo · / search · ? help · q quit".to_string()
    } else {
        "j/k move · i open · a new · d done · s start · D delete · u undo · / search · ? help · q quit"
            .to_string()
    };
    if app.popup.is_none()
        && app.add_mode
        && add_can_remove(&app.add)
        && !hint.ends_with("⌫ remove")
    {
        spans.push(Span::styled(
            format!("{hint} · ⌫ remove"),
            Style::default().fg(hint_color),
        ));
    } else {
        spans.push(Span::styled(hint, Style::default().fg(hint_color)));
    }

    let right = vec![Span::styled(position.clone(), Style::default().fg(DIM))];
    f.render_widget(
        Paragraph::new(join_right(spans, right, width)).style(Style::default().bg(BG)),
        Rect::new(area.x, area.y + 1, area.width, 1),
    );
    if let Some(p) = &app.section_prompt {
        let visible_caret = section_visible.as_ref().map(|(_, x)| *x).unwrap_or(0);
        let x = 1 + text_width(&section_prompt_prefix(p)) + visible_caret;
        place_cursor(f, area, x, area.y as usize + 1);
    }
}

/// Quick-add field. Collapsed it is a single tinted capture row; focused it
/// expands into description, attribute chips, a contextual input, and the
/// picker list. Description keeps immediate focus; trigger chars jump to a
/// picker, and only the focused picker's hints render.
fn render_add_box(items: &mut Vec<ListItem>, app: &App, width: usize) -> Option<(usize, usize)> {
    // Shared rounded boundary: LINE corners/rules, black fill. Both the
    // collapsed capture row and the expanded input mode live inside it.
    let inner_w = width.saturating_sub(2);
    let corners = |l: &'static str, r: &'static str| {
        Line::from(vec![
            Span::styled(l.to_string(), Style::default().fg(LINE)),
            Span::styled("─".repeat(inner_w), Style::default().fg(LINE)),
            Span::styled(r.to_string(), Style::default().fg(LINE)),
        ])
    };
    if !app.add_mode {
        // Outlined capture box with no fill: the LINE boundary carries
        // the shape, the background stays black like the list.
        items.push(ListItem::new(corners("╭", "╮")).style(Style::default().bg(BG)));
        let left = vec![
            Span::styled("+ ", Style::default().fg(CYAN)),
            Span::styled("Add a task…", Style::default().fg(DIM)),
        ];
        let right = vec![Span::styled("new · a", Style::default().fg(DIM))];
        let gap = inner_w.saturating_sub(spans_width(&left) + spans_width(&right));
        let mut row = vec![Span::styled("│", Style::default().fg(LINE))];
        row.extend(left);
        row.push(Span::raw(" ".repeat(gap)));
        row.extend(right);
        row.push(Span::styled("│", Style::default().fg(LINE)));
        items.push(ListItem::new(Line::from(row)).style(Style::default().bg(BG)));
        items.push(ListItem::new(corners("╰", "╯")).style(Style::default().bg(BG)));
        return None;
    }
    let d = &app.add;
    let on = |f: AddFocus| d.focus == f;
    // Content rows are collected first, then framed in the shared rounded
    // boundary: black fill everywhere, TINT only on the focused row.
    let mut body: Vec<(Vec<Span<'static>>, Style)> = Vec::new();
    // Typed description scrolls horizontally so the caret stays visible
    // instead of pushing the right border off-screen.
    let desc_max = inner_w.saturating_sub(2);
    let (desc_visible, desc_caret_x) = if d.desc.is_empty() && d.focus != AddFocus::Desc {
        ("Describe the task…".to_string(), 0)
    } else {
        visible_input(&d.desc, d.desc_cursor, desc_max.max(1))
    };
    let desc_row = body.len();
    body.push((
        vec![
            Span::styled("+ ", Style::default().fg(CYAN)),
            Span::styled(
                desc_visible,
                Style::default().fg(if on(AddFocus::Desc) { BRIGHT } else { INK }),
            ),
        ],
        Style::default().bg(if on(AddFocus::Desc) { TINT } else { BG }),
    ));
    let mut caret = if d.focus == AddFocus::Desc {
        Some((desc_row, 2 + desc_caret_x))
    } else {
        None
    };
    // Attribute chips: unset stays grey, set values take unique colors so
    // the row itself shows what the task already carries.
    let (tag_chip, tag_color) = if d.tags.is_empty() {
        ("# tags".to_string(), DIM)
    } else {
        (format!("# {}", d.tags.join(", ")), CYAN)
    };
    let (due_chip, due_color) = if d.due.is_empty() {
        ("~ due".to_string(), DIM)
    } else {
        (format!("~ {}", d.due), YELLOW)
    };
    let tag_color = if on(AddFocus::Tags) { CYAN } else { tag_color };
    let due_color = if on(AddFocus::Due) { YELLOW } else { due_color };
    let (recur_chip, recur_color) = if backend::is_recur_cleared(&d.recur) {
        ("* recur".to_string(), DIM)
    } else {
        (format!("* {}", d.recur.trim()), BLUE)
    };
    let recur_color = if on(AddFocus::Recur) {
        BRIGHT
    } else {
        recur_color
    };
    let mut chips = vec![Span::raw("  ".to_string())];
    chips.push(Span::styled(
        "[+ more] ".to_string(),
        Style::default().fg(if on(AddFocus::More) { BRIGHT } else { DIM }),
    ));
    chips.push(Span::styled(
        format!("{tag_chip} "),
        Style::default().fg(tag_color),
    ));
    chips.push(Span::styled(due_chip, Style::default().fg(due_color)));
    if !d.project.is_empty() {
        let chip_color = if on(AddFocus::Project) {
            BRIGHT
        } else {
            app.project_color(&d.project)
        };
        chips.push(Span::styled(
            format!("  @ {}", d.project),
            Style::default().fg(chip_color),
        ));
    }
    if d.prio_set {
        let label = prio_label(d.prio);
        chips.push(Span::styled(
            format!("  ! {label}"),
            Style::default().fg(if on(AddFocus::Priority) {
                BRIGHT
            } else {
                prio_color(label)
            }),
        ));
    }
    if !d.depends.is_empty() {
        chips.push(Span::styled(
            format!("  & {}", d.depends.join(", ")),
            Style::default().fg(if on(AddFocus::Depends) {
                BRIGHT
            } else {
                MAGENTA
            }),
        ));
    }
    chips.push(Span::styled(
        format!(" {recur_chip}"),
        Style::default().fg(recur_color),
    ));
    body.push((chips, Style::default().bg(BG)));
    if d.focus != AddFocus::Desc {
        // Contextual input: grey example text while the filter is empty,
        // vanishing as soon as the user types. The preset list still
        // renders below. Same examples as the inspector editor.
        let (input, example) = if !d.filter.is_empty() {
            (d.filter.clone(), false)
        } else {
            match d.focus {
                AddFocus::Tags => (CTX_EXAMPLE_TAGS.to_string(), true),
                AddFocus::Due => (CTX_EXAMPLE_DUE.to_string(), true),
                AddFocus::Project => (CTX_EXAMPLE_PROJECT.to_string(), true),
                // Fixed-list depends has no text input: no example line,
                // like priority and the + more menu.
                _ => (String::new(), false),
            }
        };
        if !input.is_empty() {
            let row = body.len();
            // Typed filters scroll so long custom tags/dates/projects keep
            // the caret on-screen; grey examples never take a caret.
            let (visible, caret_x) = if example {
                (input.clone(), 0)
            } else {
                visible_input(&d.filter, d.filter_cursor, inner_w.saturating_sub(2).max(1))
            };
            body.push((
                vec![Span::styled(
                    format!("  {visible}"),
                    Style::default().fg(if example { DIM } else { BRIGHT }),
                )],
                Style::default().bg(BG),
            ));
            let x = 2 + if example { 0 } else { caret_x };
            caret = Some((row, x));
        }
        if matches!(
            d.focus,
            AddFocus::Project | AddFocus::Priority | AddFocus::Recur | AddFocus::More
        ) && d.focus != AddFocus::Tags
            && d.focus != AddFocus::Due
        {
            let ctx = match d.focus {
                AddFocus::Project if !d.project.is_empty() => {
                    format!("  now: @ {}", d.project)
                }
                AddFocus::Priority if d.prio_set => {
                    format!("  now: ! {}", prio_label(d.prio))
                }
                AddFocus::Recur if !backend::is_recur_cleared(&d.recur) => {
                    format!("  now: * {}", d.recur.trim())
                }
                _ => String::new(),
            };
            if !ctx.is_empty() {
                body.push((
                    vec![Span::styled(ctx, Style::default().fg(INK2))],
                    Style::default().bg(BG),
                ));
            }
        }
        for (i, s) in add_suggestions(app, d).iter().enumerate() {
            let hot = i == d.picker;
            let mark = match d.focus {
                AddFocus::Tags if d.tags.iter().any(|g| g == s) => "× ",
                _ if hot => "› ",
                _ => "  ",
            };
            body.push((
                vec![
                    Span::raw("  ".to_string()),
                    Span::styled(
                        format!("{mark}{s}"),
                        Style::default().fg(if hot { BRIGHT } else { INK2 }),
                    ),
                ],
                Style::default().bg(if hot { TINT } else { BG }),
            ));
        }
    }
    // Frame the input mode in the shared boundary: top rule, every content
    // row with LINE sides, bottom rule. The terminal caret shifts one cell
    // right for the left border.
    items.push(ListItem::new(corners("╭", "╮")).style(Style::default().bg(BG)));
    let base = items.len();
    for (spans, style) in body {
        let mut row = vec![Span::styled("│", Style::default().fg(LINE))];
        row.extend(fit_spans(spans, inner_w));
        row.push(Span::styled("│", Style::default().fg(LINE)));
        items.push(ListItem::new(Line::from(row)).style(style));
    }
    items.push(ListItem::new(corners("╰", "╯")).style(Style::default().bg(BG)));
    caret.map(|(row, x)| (base + row, x + 1))
}

/// Section groups for a sectioned view: every registry section in order,
/// including empty ones, then any unknown values. Unsectioned tasks are NOT
/// a group here; project views render them as a flat list above these
/// headers. Rows are indices into the flat cursor order.
fn section_groups(app: &App, ids: &[String]) -> Vec<(String, Vec<usize>)> {
    let mut groups: Vec<(String, Vec<usize>)> = Vec::new();
    for section in &app.sections {
        let rows: Vec<usize> = ids
            .iter()
            .enumerate()
            .filter(|(_, id)| {
                app.task(id)
                    .is_some_and(|t| t.section.eq_ignore_ascii_case(section))
            })
            .map(|(i, _)| i)
            .collect();
        groups.push((section.to_uppercase(), rows));
    }
    // Defensive: a value not yet in the registry (direct test fixtures)
    // still renders after the known sections instead of disappearing.
    for (i, id) in ids.iter().enumerate() {
        let Some(t) = app.task(id) else { continue };
        if t.section.trim().is_empty()
            || app
                .sections
                .iter()
                .any(|s| s.eq_ignore_ascii_case(&t.section))
        {
            continue;
        }
        let label = t.section.to_uppercase();
        match groups.iter_mut().find(|(name, _)| name == &label) {
            Some((_, rows)) => rows.push(i),
            None => groups.push((label, vec![i])),
        }
    }
    groups
}

/// Section under the list cursor for section-aware quick-add. Project views
/// only: a section header contributes its registry name (original casing),
/// a task contributes its own section; everywhere else (including the
/// COMPLETED tail) contributes no section.
fn cursor_section(app: &App) -> String {
    if !matches!(app.view, View::Project(_)) {
        return String::new();
    }
    match app.cursor_nav() {
        Some(NavRow::SectionHeader(label)) => app
            .sections
            .iter()
            .find(|s| s.eq_ignore_ascii_case(&label))
            .cloned()
            .unwrap_or(label),
        Some(NavRow::Task(id)) => app.task(&id).map(|t| t.section.clone()).unwrap_or_default(),
        _ => String::new(),
    }
}

fn render_list(f: &mut ratatui::Frame, area: Rect, app: &App) {
    let width = area.width as usize;
    let blank = || ListItem::new(Line::from(vec![Span::raw("")])).style(Style::default().bg(BG));

    // Fixed header: view title, add/search field, and their breathing room.
    // Keeping it outside the scrolling list means the capture box never
    // scrolls away while the task rows move.
    let mut header: Vec<ListItem> = Vec::new();
    header.push(
        ListItem::new(join_right(
            vec![Span::styled(
                app.view.title(),
                Style::default().fg(BRIGHT).add_modifier(Modifier::BOLD),
            )],
            vec![Span::styled("urgency", Style::default().fg(DIM))],
            width,
        ))
        .style(Style::default().bg(BG)),
    );

    let mut header_caret: Option<(usize, usize)> = None;
    if app.view == View::Search {
        header.push(blank());
        let row = header.len();
        if app.search_query.is_empty() {
            header.push(
                ListItem::new(Line::from(vec![Span::styled(
                    if app.search_input {
                        "Type to search…".to_string()
                    } else {
                        "Type to search…  (i input)".to_string()
                    },
                    Style::default().fg(DIM),
                )]))
                .style(Style::default().bg(PANEL)),
            );
            if app.search_input {
                header_caret = Some((row, 0));
            }
        } else if app.search_input {
            let (visible, caret_x) =
                visible_input(&app.search_query, app.search_cursor, width.max(1));
            header.push(
                ListItem::new(Line::from(vec![Span::styled(
                    visible,
                    Style::default().fg(BRIGHT),
                )]))
                .style(Style::default().bg(PANEL)),
            );
            header_caret = Some((row, caret_x));
        } else {
            header.push(
                ListItem::new(Line::from(vec![
                    Span::styled(app.search_query.clone(), Style::default().fg(BRIGHT)),
                    Span::styled("  (i input)".to_string(), Style::default().fg(DIM)),
                ]))
                .style(Style::default().bg(PANEL)),
            );
        }
        header.push(blank());
    } else {
        if let Some((row, x)) = render_add_box(&mut header, app, width) {
            header_caret = Some((row, x));
        }
        header.push(blank());
    }

    let header_h = header.len().min(area.height as usize) as u16;
    let header_area = Rect::new(area.x, area.y, area.width, header_h);
    f.render_widget(List::new(header), header_area);
    if let Some((row, x)) = header_caret {
        place_cursor(f, header_area, x, header_area.y as usize + row);
    }
    let task_area = Rect::new(
        area.x,
        area.y + header_h,
        area.width,
        area.height.saturating_sub(header_h),
    );
    if task_area.is_empty() {
        return;
    }

    let ids = app.open_rows();
    let done = app.completed_rows();
    let nav = app.navigable_rows();
    if (ids.is_empty() && done.is_empty()) || nav.is_empty() {
        let hint = ListItem::new(Line::from(vec![Span::styled(
            app.view.empty_hint(),
            Style::default().fg(DIM),
        )]))
        .style(Style::default().bg(BG));
        f.render_widget(List::new(vec![hint]), task_area);
        return;
    }

    let hide_project = matches!(app.view, View::Project(_));
    let section_count = |label: &str| -> usize {
        ids.iter()
            .filter(|id| {
                let s = app.task(id).map(|t| t.section.clone()).unwrap_or_default();
                if label == "UNSECTIONED" {
                    s.trim().is_empty()
                } else {
                    s.eq_ignore_ascii_case(label)
                }
            })
            .count()
    };
    let mut items: Vec<ListItem> = Vec::new();
    let mut heights: Vec<usize> = Vec::new();
    let mut nav_items = vec![0usize; nav.len()];
    let mut seen_task = false;
    for (nav_idx, row) in nav.iter().enumerate() {
        // While the add box is open the terminal caret owns focus; the list
        // cursor highlight hides so only the add box carries a background.
        let cur = nav_idx == app.cursor && !app.add_mode;
        match row {
            NavRow::SectionHeader(label) => {
                if seen_task {
                    items.push(blank());
                    heights.push(1);
                }
                seen_task = false;
                nav_items[nav_idx] = items.len();
                let n = section_count(label);
                let arrow = if app.is_collapsed(label) {
                    "▸"
                } else {
                    "▾"
                };
                items.push(
                    ListItem::new(Line::from(vec![
                        row_bar(cur, false),
                        Span::styled(
                            format!(" {label}"),
                            Style::default().fg(if cur { BRIGHT } else { INK2 }),
                        ),
                        Span::styled(format!("  {n}"), Style::default().fg(DIM)),
                        Span::styled(format!("  {arrow}"), Style::default().fg(DIM)),
                    ]))
                    .style(Style::default().bg(if cur { TINT } else { BG })),
                );
                heights.push(1);
                items.push(blank());
                heights.push(1);
            }
            NavRow::Task(id) => {
                seen_task = true;
                let Some(t) = app.task(id) else { continue };
                nav_items[nav_idx] = items.len();
                let sel = app.selected.contains(id.as_str());
                let bg = if cur || sel { TINT } else { BG };
                let lines = task_row(t, app, width, cur, sel, hide_project);
                items.push(ListItem::new(lines).style(Style::default().bg(bg)));
                heights.push(2);
            }
            NavRow::CompletedHeader => {
                if seen_task {
                    items.push(blank());
                    heights.push(1);
                }
                seen_task = false;
                nav_items[nav_idx] = items.len();
                items.push(ListItem::new(rule_line(width, LINE)).style(Style::default().bg(BG)));
                heights.push(1);
                let arrow = if app.is_collapsed("COMPLETED") {
                    "▸"
                } else {
                    "▾"
                };
                items.push(
                    ListItem::new(Line::from(vec![
                        row_bar(cur, false),
                        Span::styled(
                            " COMPLETED".to_string(),
                            Style::default().fg(if cur { BRIGHT } else { DIM }),
                        ),
                        Span::styled(format!("  {}", done.len()), Style::default().fg(DIM)),
                        Span::styled(format!("  {arrow}"), Style::default().fg(DIM)),
                    ]))
                    .style(Style::default().bg(if cur { TINT } else { BG })),
                );
                heights.push(1);
            }
            NavRow::CompletedTask(id) => {
                let Some(t) = app.task(id) else { continue };
                nav_items[nav_idx] = items.len();
                let sel = app.selected.contains(id.as_str());
                let bg = if cur || sel { TINT } else { BG };
                items.push(
                    ListItem::new(completed_row(t, width, cur, sel)).style(Style::default().bg(bg)),
                );
                heights.push(1);
            }
        }
    }

    if items.is_empty() {
        return;
    }
    // Keep the cursor item fully visible with a two-line cushion below it.
    // The offset persists between frames so moving up scrolls one row at a
    // time instead of jumping back to the top. Scrolling is item-based, so a
    // two-line task never gets sliced at the top; the fixed header above is
    // unaffected.
    let viewport = task_area.height as usize;
    let cursor_item = nav_items.get(app.cursor).copied().unwrap_or(0);
    let mut offset = app.list_offset.get().min(items.len().saturating_sub(1));
    if cursor_item < offset {
        offset = cursor_item;
    }
    while offset < cursor_item {
        let above: usize = heights[offset..cursor_item].iter().sum();
        if above + heights[cursor_item] + 2 <= viewport {
            break;
        }
        offset += 1;
    }
    app.list_offset.set(offset);
    let mut state = ListState::default().with_offset(offset);
    f.render_stateful_widget(List::new(items), task_area, &mut state);
}

// ---------------------------------------------------------------------------
// popup rendering
// ---------------------------------------------------------------------------

fn popup_title(p: &Popup) -> (String, Color) {
    match p.mode {
        PopupMode::View => ("VIEW".into(), INK2),
        PopupMode::Edit => ("EDIT".into(), MINT),
    }
}

/// Shared frame for the split inspector and the narrow-terminal overlay.
fn render_inspector_frame(f: &mut ratatui::Frame, area: Rect, p: &Popup, app: &App) {
    f.render_widget(Clear, area);
    // The box border brightens while the inspector holds focus and falls
    // back to the hairline color when focus shifts to the task list.
    let border = if app.popup_list_focus { LINE } else { INK2 };
    f.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::default().fg(border))
            .style(Style::default().bg(BG)),
        area,
    );
    render_inspector_body(f, area, p, app);
}

/// Right-hand split pane used at normal terminal widths.
fn render_inspector(f: &mut ratatui::Frame, area: Rect, p: &Popup, app: &App) {
    render_inspector_frame(f, area, p, app);
}

/// Full-screen detail overlay used when the terminal is too narrow to split.
fn render_popup(f: &mut ratatui::Frame, area: Rect, p: &Popup, app: &App) {
    render_inspector_frame(f, area, p, app);
}

/// Task title, attribute rows, annotations, and dependency relations.
/// The title doubles as the editable description field (index 0).
fn render_inspector_body(f: &mut ratatui::Frame, area: Rect, p: &Popup, app: &App) {
    let inner = Rect::new(
        area.x + 2,
        area.y + 1,
        area.width.saturating_sub(4),
        area.height.saturating_sub(2),
    );
    if inner.is_empty() {
        return;
    }
    let width = inner.width as usize;

    // Panel header: TASK + mode, then a hairline with a cyan TASK underline.
    let (mode, mode_color) = popup_title(p);
    let mut head = vec![
        Span::styled(
            "TASK",
            Style::default().fg(BRIGHT).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(mode, Style::default().fg(mode_color)),
    ];
    if p.dirty {
        head.push(Span::styled("  ● unsaved", Style::default().fg(YELLOW)));
    }
    f.render_widget(
        Paragraph::new(Line::from(head)).style(Style::default().bg(BG)),
        Rect::new(inner.x, inner.y, inner.width, 1),
    );
    let underline = width.min(4);
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("─".repeat(underline), Style::default().fg(CYAN)),
            Span::styled("─".repeat(width - underline), Style::default().fg(LINE)),
        ]))
        .style(Style::default().bg(BG)),
        Rect::new(inner.x, inner.y + 1, inner.width, 1),
    );

    // Title row: the description field. A focused title gets a small marker;
    // EDIT places the terminal caret directly on it. Long titles scroll so
    // the caret stays on-screen instead of overflowing the pane.
    let title_y = inner.y + 2;
    let desc_focused = p.field == F_DESC;
    let editing_desc = p.mode == PopupMode::Edit && desc_focused;
    let (title_text, title_example) = if editing_desc && p.desc.is_empty() {
        ("e.g. Call the client back".to_string(), true)
    } else {
        (p.desc.clone(), false)
    };
    let title_max = width.saturating_sub(2).max(1);
    let (title_visible, title_caret_x) = if editing_desc && !title_example {
        visible_input(&p.desc, p.edit_cursor, title_max)
    } else {
        (fit(&title_text, title_max), 0)
    };
    let marker = if desc_focused { "▸ " } else { "  " };
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(marker, Style::default().fg(CYAN)),
            Span::styled(
                title_visible,
                Style::default()
                    .fg(if title_example { DIM } else { BRIGHT })
                    .add_modifier(Modifier::BOLD),
            ),
        ]))
        .style(Style::default().bg(if editing_desc { TINT } else { BG })),
        Rect::new(inner.x, title_y, inner.width, 1),
    );
    if editing_desc && !app.add_mode {
        let x = 2 + title_caret_x;
        place_cursor(f, inner, x, title_y as usize);
    }

    let content_y = inner.y + 4;
    let content_h = inner.bottom().saturating_sub(content_y);
    if content_h == 0 {
        return;
    }
    let content = Rect::new(inner.x, content_y, inner.width, content_h);
    let mut lines: Vec<ListItem> = Vec::new();
    let editing = p.mode == PopupMode::Edit;

    let field_color = |i: usize| -> Color {
        match i {
            F_PROJECT => app.project_color(&p.project),
            F_DUE => {
                let task = app.task(&p.task_id);
                if task.is_some_and(is_overdue) {
                    RED
                } else if task.is_some_and(is_due_today) {
                    YELLOW
                } else {
                    INK2
                }
            }
            F_PRIO => prio_color(prio_label(p.prio)),
            F_TAGS => CYAN,
            F_DEPENDS | F_RECUR => INK2,
            _ => INK,
        }
    };
    let example_for = |field: usize| -> &'static str {
        match field {
            F_PROJECT => CTX_EXAMPLE_PROJECT,
            F_DUE => CTX_EXAMPLE_DUE,
            F_TAGS => CTX_EXAMPLE_TAGS,
            _ => "",
        }
    };
    for i in 1..=F_RECUR {
        let on = p.field == i;
        let focused = on && editing;
        let label = field_label(i);
        let body = popup_field_text(p, i);
        let label_style = Style::default().fg(if on { BLUE } else { INK2 });
        if focused {
            let (text, is_example) = if body.is_empty() {
                (example_for(i).to_string(), true)
            } else {
                (body.clone(), false)
            };
            let row = lines.len();
            // Typed fields scroll so long edits keep the caret on-screen.
            let field_max = width.saturating_sub(12).max(1);
            let (visible_text, visible_caret) = if is_example {
                (text.clone(), 0)
            } else {
                visible_input(&body, p.edit_cursor, field_max)
            };
            lines.push(
                ListItem::new(Line::from(vec![
                    Span::styled(format!(" {label:<10} "), label_style),
                    Span::styled(
                        visible_text,
                        Style::default().fg(if is_example { DIM } else { BRIGHT }),
                    ),
                ]))
                .style(Style::default().bg(TINT)),
            );
            if i != F_PRIO && i != F_DEPENDS && i != F_RECUR && !app.add_mode {
                let x = 12 + visible_caret;
                place_cursor(f, content, x, content.y as usize + row);
            }
        } else {
            let color = if body.is_empty() { DIM } else { field_color(i) };
            lines.push(
                ListItem::new(Line::from(vec![
                    Span::styled(format!(" {label:<10} "), label_style),
                    Span::styled(body, Style::default().fg(color)),
                ]))
                .style(Style::default().bg(BG)),
            );
        }
        if focused {
            let saved = app.task(&p.task_id);
            let now_line = match i {
                F_PROJECT if !saved.map(|t| t.project.as_str()).unwrap_or("").is_empty() => {
                    Some(format!("now: @ {}", saved.unwrap().project))
                }
                F_PRIO => saved.map(|t| format!("now: ! {}", prio_label(t.prio))),
                F_RECUR if saved.is_some_and(|t| !backend::is_recur_cleared(&t.recur)) => {
                    Some(format!("now: * {}", saved.unwrap().recur))
                }
                _ => None,
            };
            if let Some(text) = now_line {
                lines.push(
                    ListItem::new(Line::from(vec![
                        Span::raw(" ".repeat(12)),
                        Span::styled(text, Style::default().fg(INK2)),
                    ]))
                    .style(Style::default().bg(BG)),
                );
            }
            let sugs = popup_suggestions(app, p);
            let visible = CTX_SUGGEST_CAP;
            let start = if sugs.len() <= visible {
                0
            } else {
                p.suggest_idx
                    .min(sugs.len().saturating_sub(1))
                    .saturating_sub(visible - 1)
                    .min(sugs.len() - visible)
            };
            for (offset, s) in sugs.iter().skip(start).take(visible).enumerate() {
                let si = start + offset;
                let hot = si == p.suggest_idx.min(sugs.len().saturating_sub(1));
                lines.push(
                    ListItem::new(Line::from(vec![
                        Span::raw(" ".repeat(12)),
                        Span::styled(
                            format!("{} {s}", if hot { "›" } else { " " }),
                            Style::default().fg(if hot { BRIGHT } else { INK2 }),
                        ),
                    ]))
                    .style(Style::default().bg(if hot { TINT } else { BG })),
                );
            }
        }
    }

    // Read-only block, one blank line below the editable fields.
    lines.push(ListItem::new(Line::from(vec![Span::raw("")])).style(Style::default().bg(BG)));
    let task = app.task(&p.task_id);
    let id_body = task.map(task_id_display).unwrap_or_default();
    let status_body = task.map(|t| t.status.clone()).unwrap_or_default();
    let urgency_body = task
        .map(|t| format!("{:.1}", t.urgency))
        .unwrap_or_default();
    let now = Local::now();
    let start_body = task.map(|t| start_display(t, now)).unwrap_or_default();
    let readonly = [
        ("id", id_body, INK2),
        ("status", status_body, INK),
        ("urgency", urgency_body, ORANGE),
        ("start", start_body, INK),
    ];
    for (label, value, color) in readonly {
        lines.push(
            ListItem::new(Line::from(vec![
                Span::styled(format!(" {label:<10} "), Style::default().fg(DIM)),
                Span::styled(value, Style::default().fg(color)),
            ]))
            .style(Style::default().bg(BG)),
        );
    }

    // Interactive annotation block.
    lines.push(ListItem::new(Line::from(vec![Span::raw("")])).style(Style::default().bg(BG)));
    let live_annotations: Vec<Annotation> = task.map(|t| t.annotations.clone()).unwrap_or_default();
    let annotation_label = if live_annotations.is_empty() {
        " annotation ".to_string()
    } else {
        format!(" annotation ({}) ", live_annotations.len())
    };
    let annotation_hint = if p.annotating {
        "type + Enter add · esc cancel"
    } else {
        "a add · [ ] select · D delete"
    };
    lines.push(
        ListItem::new(join_right(
            vec![Span::styled(annotation_label, Style::default().fg(INK2))],
            vec![Span::styled(annotation_hint, Style::default().fg(DIM))],
            width,
        ))
        .style(Style::default().bg(BG)),
    );
    let selected_idx = p
        .annotation_idx
        .min(live_annotations.len().saturating_sub(1));
    for (idx, note) in live_annotations.iter().enumerate() {
        let selected = idx == selected_idx && !p.annotating;
        let marker = if selected { "›" } else { " " };
        let mut body = note.text.clone();
        if !note.entry.is_empty() {
            if let Some(utc) = parse_task_datetime(note.entry.trim()) {
                let date = utc.with_timezone(&Local).format("%Y-%m-%d").to_string();
                body = format!("{} · {date}", note.text);
            }
        }
        lines.push(
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!(" {marker} "),
                    Style::default().fg(if selected { CYAN } else { DIM }),
                ),
                Span::styled(
                    fit(&body, width.saturating_sub(3).max(1)),
                    Style::default().fg(if selected { BRIGHT } else { INK }),
                ),
            ]))
            .style(Style::default().bg(if selected { TINT } else { BG })),
        );
    }
    if p.annotating {
        let (text, is_example) = if p.annotation_draft.is_empty() {
            ("e.g. call the client back".to_string(), true)
        } else {
            (p.annotation_draft.clone(), false)
        };
        let row = lines.len();
        let note_max = width.saturating_sub(3).max(1);
        let (visible, visible_caret) = if is_example {
            (text.clone(), 0)
        } else {
            visible_input(&p.annotation_draft, p.annotation_cursor, note_max)
        };
        lines.push(
            ListItem::new(Line::from(vec![
                Span::styled(" > ", Style::default().fg(CYAN)),
                Span::styled(
                    visible,
                    Style::default().fg(if is_example { DIM } else { BRIGHT }),
                ),
            ]))
            .style(Style::default().bg(TINT)),
        );
        let x = 3 + visible_caret;
        if !app.add_mode {
            place_cursor(f, content, x, content.y as usize + row);
        }
    }

    // Dependency relations close the panel.
    let blocked_by = app.blocked_by(&p.task_id);
    let blocking = app.blocking(&p.task_id);
    let rel_ids: Vec<String> = blocked_by
        .iter()
        .map(|t| t.uuid.clone())
        .chain(blocking.iter().map(|t| t.uuid.clone()))
        .collect();
    let rel_id_width = rel_ids
        .iter()
        .filter_map(|id| app.task(id))
        .map(|t| task_id_display(t).chars().count())
        .max()
        .unwrap_or(2)
        .clamp(2, 8);
    for (label, color, rows) in [
        ("BLOCKED BY", RED, &blocked_by),
        ("BLOCKS", BLUE, &blocking),
    ] {
        if rows.is_empty() {
            continue;
        }
        lines.push(ListItem::new(Line::from(vec![Span::raw("")])).style(Style::default().bg(BG)));
        lines.push(
            ListItem::new(Line::from(vec![Span::styled(
                format!(" {label}"),
                Style::default().fg(color),
            )]))
            .style(Style::default().bg(BG)),
        );
        for rel in rows {
            lines.push(
                ListItem::new(rel_row(rel, width, rel_id_width)).style(Style::default().bg(BG)),
            );
        }
    }

    f.render_widget(List::new(lines), content);
}

fn render_help(f: &mut ratatui::Frame, area: Rect) {
    let w = area.width.saturating_sub(8).clamp(40, 68);
    let h = 34.min(area.height.saturating_sub(2)).max(12);
    let r = Rect::new(
        area.x + area.width.saturating_sub(w) / 2,
        area.y + area.height.saturating_sub(h) / 2,
        w,
        h,
    );
    f.render_widget(Clear, r);
    let lines = vec![
        "  KEYS",
        "",
        "  List",
        "  j/k        move cursor (headers toggle)",
        "  Enter/i    toggle inspector · h/← close inspector",
        "  →          open inspector (or next tab when no task)",
        "  h/i/l      list ⇄ inspector focus (split open)",
        "  Tab        switch tabs, even with inspector open",
        "  space      select · v range, j/k extends",
        "  a          new task",
        "  d          done / undone · s start/stop",
        "  D          delete (y/n) · u undo",
        "  m          move section · t retag selection",
        "  /          search (hotkey only) · esc returns",
        "  Tab/⇧Tab  switch tabs (no wrap) · 1-3 views · 4+ projects",
        "",
        "  Edit",
        "  e          VIEW → EDIT one field · type at cursor",
        "  Tab/⇧Tab   move hint · ↑/↓ move hint · ←/→ cursor · ⌫/Del edit",
        "  Enter      apply hint + save · esc discard field",
        "  d          done in VIEW",
        "  a          note add in VIEW · D del note · [/] select",
        "  a          add · # tags · ~ due · @ · ! · +",
        "  ↑/↓/Tab    pick list · Enter apply",
        "",
        "  Search",
        "  /          open search over every task",
        "  i input · Tab/↑/↓ move · Enter open · esc clears/returns",
        "",
        "  ?          this help · q / esc quit",
    ];
    f.render_widget(
        Paragraph::new(
            lines
                .into_iter()
                .map(|l| {
                    Line::from(vec![Span::styled(
                        l.to_string(),
                        Style::default().fg(if l.trim().is_empty() {
                            DIM
                        } else if l == "  KEYS" || l == "  List" || l == "  Edit" || l == "  Search"
                        {
                            BRIGHT
                        } else {
                            INK
                        }),
                    )])
                })
                .collect::<Vec<_>>(),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(DIM))
                .style(Style::default().bg(BG)),
        ),
        r,
    );
}

#[cfg(test)]
mod smoke {
    use super::*;
    use ratatui::backend::{Backend, TestBackend};
    use ratatui::layout::Position;

    fn content_of(buf: &ratatui::buffer::Buffer) -> String {
        buf.content()
            .iter()
            .map(|c| c.symbol().to_string())
            .collect()
    }

    fn draw(app: &App) -> String {
        let backend = TestBackend::new(120, 40);
        let mut term = Terminal::new(backend).unwrap();
        term.draw(|f| render(f, app)).unwrap();
        content_of(term.backend().buffer())
    }

    /// Draw with a cursor sentinel so tests can tell a positioned typing
    /// caret from a hidden one. Returns the trimmed lines and the position.
    fn draw_with_cursor(app: &App) -> (Vec<String>, Position) {
        let mut backend = TestBackend::new(120, 40);
        let sentinel = Position::new(119, 39);
        backend.set_cursor_position(sentinel).unwrap();
        let mut term = Terminal::new(backend).unwrap();
        term.draw(|f| render(f, app)).unwrap();
        let buf = term.backend().buffer();
        let w = buf.area.width as usize;
        let lines = buf
            .content()
            .chunks(w)
            .map(|row| {
                row.iter()
                    .map(|c| c.symbol().to_string())
                    .collect::<String>()
                    .trim_end()
                    .to_string()
            })
            .collect();
        let cursor = term.backend_mut().get_cursor_position().unwrap();
        (lines, cursor)
    }

    fn draw_lines(app: &App) -> Vec<String> {
        draw_with_cursor(app).0
    }

    /// The list owns the full width now; assertions only need the line
    /// without its surrounding padding.
    fn main_col(line: &str) -> String {
        line.trim().to_string()
    }

    /// App with an isolated backend: unique temp `TASKDATA` so parallel
    /// tests never touch live data. `uses_backend()` is true here, so
    /// every write goes through the real `task` binary.
    fn backend_app() -> App {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "siap-app-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            n
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let rc = dir.join("taskrc");
        std::fs::write(
            &rc,
            format!("data.location={}\nconfirmation=no\n", dir.display()),
        )
        .unwrap();
        let mut app = App::new();
        app.taskdata_dir = Some(dir);
        app.taskrc_path = Some(rc);
        app.bootstrap_backend();
        app
    }

    /// Create fixture tasks through the backend, then load them.
    fn seed(app: &mut App, descs: &[&str]) {
        for d in descs {
            let add = backend::build_add_args(d, "", "", None, &[], &[], "", "");
            let refs: Vec<&str> = add.iter().map(String::as_str).collect();
            app.run_backend(&refs).expect("seed add succeeds");
        }
        app.refresh_from_backend();
        assert!(
            !app.toast.starts_with("refresh failed"),
            "seed refresh: {}",
            app.toast
        );
    }

    fn uuid_of(app: &App, desc: &str) -> String {
        app.tasks
            .iter()
            .find(|t| t.desc == desc)
            .unwrap_or_else(|| panic!("fixture {desc} present"))
            .uuid
            .clone()
    }

    fn goto(app: &mut App, uuid: &str) {
        let rows = app.navigable_rows();
        app.cursor = rows
            .iter()
            .position(|r| match r {
                NavRow::Task(id) | NavRow::CompletedTask(id) => id == uuid,
                _ => false,
            })
            .unwrap_or_else(|| panic!("uuid {uuid} in navigable rows"));
    }

    #[test]
    fn backend_start_stop_done_hold_cursor() {
        let mut app = backend_app();
        app.view = View::All;
        seed(&mut app, &["alpha op", "beta op"]);
        let alpha = uuid_of(&app, "alpha op");
        let beta = uuid_of(&app, "beta op");
        goto(&mut app, &alpha);
        // `s` starts; cursor stays on the same task by `uuid`.
        app.single_start_stop();
        assert!(app.task(&alpha).unwrap().active, "start marks active");
        assert_eq!(app.cursor_id().as_deref(), Some(alpha.as_str()));
        assert_eq!(app.toast, "toggled start/stop");
        // `s` stops.
        app.single_start_stop();
        assert!(!app.task(&alpha).unwrap().active, "stop clears active");
        assert_eq!(app.cursor_id().as_deref(), Some(alpha.as_str()));
        // `d` completes; the cursor follows into the COMPLETED tail.
        app.single_done();
        assert_eq!(app.task(&alpha).unwrap().status, "completed");
        assert_eq!(
            app.cursor_id().as_deref(),
            Some(alpha.as_str()),
            "cursor follows into COMPLETED"
        );
        assert!(
            app.open_rows().contains(&beta),
            "other open task stays open"
        );
    }

    #[test]
    fn backend_quick_add_writes_no_section() {
        let mut app = backend_app();
        app.view = View::All;
        app.add.desc = "unsectioned add".into();
        app.add.project = "work".into();
        app.quick_add_commit();
        assert_eq!(app.toast, "created 1 task · esc done");
        let pending = app.run_backend(&["+PENDING", "export"]).expect("export");
        let values: Vec<serde_json::Value> = serde_json::from_str(&pending).unwrap();
        let added = values
            .iter()
            .find(|v| v.get("description").and_then(|d| d.as_str()) == Some("unsectioned add"))
            .expect("added task reads back");
        let section = added
            .get("siapsection")
            .and_then(|s| s.as_str())
            .unwrap_or("");
        assert!(section.is_empty(), "quick-add stores no section value");
        assert!(
            !added
                .get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("")
                .contains("siapsection"),
            "section text never leaks into the description"
        );
    }

    #[test]
    fn uda_registration_failure_toasts_without_section_leak() {
        let mut app = backend_app();
        // Pointing TASKDATA at a file makes `task config` fail, so the
        // non-interactive registration fails instead of prompting or hanging.
        let file = app.taskrc_path.clone().expect("isolated taskrc");
        app.taskdata_dir = Some(file);
        app.bootstrap_backend();
        assert!(
            app.toast.starts_with("uda setup failed:"),
            "registration failure toasts: {}",
            app.toast
        );
        // A failed registration never folds section text into a description:
        // the add write fails here, leaving the draft exactly as typed.
        app.view = View::All;
        app.add.desc = "probe".into();
        app.quick_add_commit();
        assert!(app.toast.starts_with("add failed:"));
        assert_eq!(app.add.desc, "probe");
        assert!(
            !app.add.desc.to_lowercase().contains("siapsection"),
            "no section text leaks into the description"
        );
    }

    #[test]
    fn backend_delete_is_single_with_no_cascade() {
        let mut app = backend_app();
        app.view = View::All;
        seed(&mut app, &["lone task", "other task"]);
        let lone = uuid_of(&app, "lone task");
        let other = uuid_of(&app, "other task");
        goto(&mut app, &lone);
        app.request_delete();
        app.confirm_delete_yes();
        assert_eq!(app.toast, "deleted 1 task(s)");
        assert!(app.task(&lone).is_none(), "deleted task leaves");
        assert!(app.task(&other).is_some(), "no cascade to others");
    }

    #[test]
    fn backend_clearing_recurrence_keeps_previous_occurrences() {
        let mut app = backend_app();
        app.view = View::All;
        let add = backend::build_add_args(
            "weekly review",
            "work",
            "today",
            None,
            &[],
            &[],
            "weekly",
            "",
        );
        let refs: Vec<&str> = add.iter().map(String::as_str).collect();
        app.run_backend(&refs).expect("recurring add succeeds");
        app.refresh_from_backend();
        assert!(!app.toast.starts_with("refresh failed"), "{}", app.toast);

        let mut instances: Vec<Task> = app
            .tasks
            .iter()
            .filter(|t| t.desc == "weekly review" && is_open(t))
            .cloned()
            .collect();
        instances.sort_by(|a, b| a.due_raw.cmp(&b.due_raw));
        assert!(
            instances.len() >= 2,
            "taskwarrior pre-generates next occurrences"
        );
        assert!(
            instances
                .iter()
                .all(|t| t.recur == "weekly" && !t.recur_parent.is_empty()),
            "generated occurrences carry recur + parent"
        );
        let keep = instances[0].clone();
        let dropped: Vec<String> = instances[1..].iter().map(|t| t.uuid.clone()).collect();

        app.popup = Some(Popup::open(&app, &keep));
        {
            let p = app.popup.as_mut().unwrap();
            p.field = F_RECUR;
            p.mode = PopupMode::Edit;
            p.recur.clear();
            p.dirty = true;
        }
        app.save_popup();
        assert_eq!(app.toast, "saved");

        let kept = app.task(&keep.uuid).expect("selected occurrence stays");
        assert_eq!(kept.recur, "none");
        assert!(
            kept.recur_parent.is_empty(),
            "selected occurrence detaches from its template"
        );
        for uuid in &dropped {
            assert!(app.task(uuid).is_none(), "other occurrence {uuid} removed");
        }

        let pending = app.run_backend(&["+PENDING", "export"]).expect("export");
        let values: Vec<serde_json::Value> = serde_json::from_str(&pending).unwrap();
        assert_eq!(values.len(), 1, "only the kept occurrence remains");
        assert_eq!(values[0]["uuid"], keep.uuid);
        assert!(values[0].get("recur").is_none(), "no recurrence stored");
        assert!(values[0].get("parent").is_none(), "no template link stored");

        let parents = app
            .run_backend(&["status:recurring", "export"])
            .expect("parent export");
        let parents: Vec<serde_json::Value> = serde_json::from_str(&parents).unwrap();
        assert!(parents.is_empty(), "hidden recurring template is deleted");

        // Scenario B: keep the later occurrence and turn earlier occurrences
        // into one-offs instead of deleting them.
        let add = backend::build_add_args("daily review", "", "today", None, &[], &[], "daily", "");
        let refs: Vec<&str> = add.iter().map(String::as_str).collect();
        app.run_backend(&refs)
            .expect("second recurring add succeeds");
        app.refresh_from_backend();
        let mut later: Vec<Task> = app
            .tasks
            .iter()
            .filter(|t| t.desc == "daily review" && is_open(t))
            .cloned()
            .collect();
        later.sort_by(|a, b| a.due_raw.cmp(&b.due_raw));
        assert!(later.len() >= 2, "two daily occurrences generated");
        let keep_later = later.last().unwrap().clone();
        let earlier = later[0].uuid.clone();
        app.popup = Some(Popup::open(&app, &keep_later));
        {
            let p = app.popup.as_mut().unwrap();
            p.field = F_RECUR;
            p.mode = PopupMode::Edit;
            p.recur.clear();
            p.dirty = true;
        }
        app.save_popup();
        assert_eq!(app.toast, "saved");
        let kept_later = app.task(&keep_later.uuid).expect("later occurrence kept");
        assert_eq!(kept_later.recur, "none");
        assert!(kept_later.recur_parent.is_empty());
        let kept_earlier = app.task(&earlier).expect("earlier occurrence kept");
        assert_eq!(kept_earlier.recur, "none");
        assert!(
            kept_earlier.recur_parent.is_empty(),
            "earlier occurrence detaches too"
        );

        let pending = app.run_backend(&["+PENDING", "export"]).expect("export");
        let values: Vec<serde_json::Value> = serde_json::from_str(&pending).unwrap();
        let daily: Vec<&serde_json::Value> = values
            .iter()
            .filter(|v| v.get("description").and_then(|d| d.as_str()) == Some("daily review"))
            .collect();
        assert_eq!(daily.len(), 2, "earlier and selected occurrences remain");
        for value in daily {
            assert!(value.get("recur").is_none());
            assert!(value.get("parent").is_none());
        }
        let parents = app
            .run_backend(&["status:recurring", "export"])
            .expect("parent export");
        let parents: Vec<serde_json::Value> = serde_json::from_str(&parents).unwrap();
        assert!(parents.is_empty(), "hidden recurring template is deleted");
    }

    #[test]
    fn backend_section_move_persists() {
        let mut app = backend_app();
        // Section moves live in project views; stamp a project first, then
        // enter the view (entering earlier falls back to All with no rows).
        seed(&mut app, &["movable", "intake"]);
        let id = uuid_of(&app, "movable");
        app.run_backend(&[id.as_str(), "modify", "project:work"])
            .expect("project stamps");
        app.refresh_from_backend();
        app.view = View::Project("work".into());
        // The add path seeds no sections; name two so `m` has an order.
        app.manual_sections = vec!["Now".into(), "Next".into()];
        app.sync_sections();
        goto(&mut app, &id);
        // An unsectioned task lands on the first registry section.
        app.move_section_forward();
        assert_eq!(app.toast, "moved to Now");
        assert_eq!(app.task(&id).unwrap().section, "Now");
        assert_eq!(
            app.cursor_id().as_deref(),
            Some(id.as_str()),
            "cursor holds by uuid"
        );
        app.move_section_forward();
        assert_eq!(app.toast, "moved to Next");
        assert_eq!(app.task(&id).unwrap().section, "Next");
        // Inbox still refuses section moves without touching the backend.
        app.view = View::Inbox;
        let inbox_id = uuid_of(&app, "intake");
        goto(&mut app, &inbox_id);
        app.move_section_forward();
        assert_eq!(app.toast, "sections live in project views");
        assert!(app.task(&inbox_id).unwrap().section.is_empty());
    }

    #[test]
    fn backend_depends_blocks_until_open_deps_complete() {
        let mut app = backend_app();
        app.view = View::All;
        seed(&mut app, &["blocked task", "blocker"]);
        let blocked = uuid_of(&app, "blocked task");
        let blocker = uuid_of(&app, "blocker");
        // Native `depends`: the blocked task lists the blocker.
        let add = backend::build_depends_add_args(&blocked, &blocker);
        let refs: Vec<&str> = add.iter().map(String::as_str).collect();
        app.run_backend(&refs).expect("depends links");
        app.refresh_from_backend();
        assert!(app.is_blocked(&blocked), "open dep blocks");
        assert!(
            app.blocked_by(&blocked).iter().any(|t| t.uuid == blocker),
            "blocked_by lists the blocker"
        );
        assert!(
            app.blocking(&blocker).iter().any(|t| t.uuid == blocked),
            "blocking is the reverse index"
        );
        // Completing the blocker lifts the block.
        goto(&mut app, &blocker);
        app.single_done();
        assert!(!app.is_blocked(&blocked), "block lifts when deps complete");
        assert!(
            app.blocked_by(&blocked).is_empty(),
            "completed deps hide from display"
        );
        // Reopening the blocker restores the block (the link survives completion).
        goto(&mut app, &blocker);
        app.single_done();
        app.refresh_from_backend();
        assert!(app.is_blocked(&blocked), "reopened blocker blocks again");
        let rm = backend::build_depends_remove_args(&blocked, &blocker);
        let refs: Vec<&str> = rm.iter().map(String::as_str).collect();
        app.run_backend(&refs).expect("depends unlinks");
        app.refresh_from_backend();
        assert!(!app.is_blocked(&blocked), "remove-one unblocks");
    }

    #[test]
    fn backend_bulk_ops_loop_per_uuid() {
        let mut app = backend_app();
        app.view = View::All;
        seed(
            &mut app,
            &["bulk one", "bulk two", "bulk three", "bulk four"],
        );
        let b1 = uuid_of(&app, "bulk one");
        let b2 = uuid_of(&app, "bulk two");
        let b3 = uuid_of(&app, "bulk three");
        let b4 = uuid_of(&app, "bulk four");
        // Bulk `x` completes each selected task by `uuid`.
        app.selected.insert(b1.clone());
        app.selected.insert(b2.clone());
        let ids: Vec<String> = app.selected.iter().cloned().collect();
        app.complete_ids(&ids);
        assert_eq!(app.toast, "completed 2 task(s)");
        assert!(app.selected.is_empty(), "bulk done clears");
        assert_eq!(app.task(&b1).unwrap().status, "completed");
        assert_eq!(app.task(&b2).unwrap().status, "completed");
        // Bulk `t` toggles ±urgent; cursor holds on the same task.
        goto(&mut app, &b3);
        app.selected.insert(b3.clone());
        app.bulk_toggle_tag();
        assert!(app.task(&b3).unwrap().tags.contains(&"urgent".into()));
        assert_eq!(app.cursor_id().as_deref(), Some(b3.as_str()));
        app.selected.insert(b3.clone());
        app.bulk_toggle_tag();
        assert!(!app.task(&b3).unwrap().tags.contains(&"urgent".into()));
        // Bulk `m` cycles the project per task.
        app.selected.insert(b3.clone());
        app.bulk_move_project();
        assert_eq!(app.task(&b3).unwrap().project, "work");
        assert_eq!(app.cursor_id().as_deref(), Some(b3.as_str()));
        // Bulk `d` deletes every selected task together.
        app.selected.insert(b3.clone());
        app.selected.insert(b4.clone());
        app.request_delete();
        app.confirm_delete_yes();
        assert_eq!(app.toast, "deleted 2 task(s)");
        assert!(
            app.task(&b3).is_none() && app.task(&b4).is_none(),
            "bulk delete removes both"
        );
    }

    #[test]
    fn backend_refresh_picks_up_external_edits() {
        let mut app = backend_app();
        app.view = View::All;
        seed(&mut app, &["first"]);
        let first = uuid_of(&app, "first");
        goto(&mut app, &first);
        // Change made behind the app's back, as `r` would encounter it.
        let add = backend::build_add_args("external edit", "", "", None, &[], &[], "", "");
        let refs: Vec<&str> = add.iter().map(String::as_str).collect();
        app.run_backend(&refs).expect("external add");
        app.refresh_from_backend();
        assert!(
            app.tasks.iter().any(|t| t.desc == "external edit"),
            "refresh shows the external task"
        );
        assert_eq!(
            app.cursor_id().as_deref(),
            Some(first.as_str()),
            "cursor holds by uuid across refresh"
        );
    }

    #[test]
    fn backend_live_mapping_section_fallback_and_refresh_hold() {
        let mut app = backend_app();
        app.view = View::All;
        // One explicit task plus one legacy task with no priority/section.
        let add = backend::build_add_args(
            "mapped live",
            "work",
            "today",
            Some('H'),
            &[String::from("live")],
            &[],
            "",
            "",
        );
        let refs: Vec<&str> = add.iter().map(String::as_str).collect();
        app.run_backend(&refs).expect("add mapped");
        // The add path leaves sections unset; stamp this fixture explicitly.
        app.run_backend(&["+LATEST", "modify", "siapsection:Next"])
            .expect("seed section");
        app.run_backend(&["add", "plain legacy"])
            .expect("add legacy");
        app.refresh_from_backend();
        assert!(
            !app.toast.starts_with("refresh failed"),
            "live refresh: {}",
            app.toast
        );
        let mapped = uuid_of(&app, "mapped live");
        let plain = uuid_of(&app, "plain legacy");

        // Absent priority renders L; urgency reads from the backend.
        let plain_task = app.task(&plain).unwrap().clone();
        assert_eq!(plain_task.prio, 'L');
        assert_eq!(prio_label(plain_task.prio), "L");
        // A missing section stays unsectioned; nothing is written on read.
        assert_eq!(plain_task.section, "");
        let pending = app.run_backend(&["+PENDING", "export"]).expect("export");
        let values: Vec<serde_json::Value> = serde_json::from_str(&pending).unwrap();
        let plain_json = values
            .iter()
            .find(|v| v.get("uuid").and_then(|x| x.as_str()) == Some(plain.as_str()))
            .expect("plain in export");
        assert!(
            plain_json
                .get("siapsection")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .trim()
                .is_empty(),
            "fallback writes nothing"
        );
        let export_urgency = plain_json
            .get("urgency")
            .and_then(|x| x.as_f64())
            .unwrap_or(0.0) as f32;
        assert!(
            (plain_task.urgency - export_urgency).abs() < f32::EPSILON,
            "urgency reads from backend"
        );

        // Explicit mapping survives the round-trip with a friendly due label.
        let mapped_task = app.task(&mapped).unwrap().clone();
        assert_eq!(mapped_task.section, "Next");
        assert_eq!(mapped_task.prio, 'H');
        assert!(
            mapped_task.due.starts_with("today"),
            "live due renders friendly: {}",
            mapped_task.due
        );

        // Priority save refreshes the read-only urgency from the backend.
        let modify =
            backend::build_modify_args(&plain, "plain legacy", "", "", 'H', &[], &[], "Now", "");
        let refs: Vec<&str> = modify.iter().map(String::as_str).collect();
        app.run_backend(&refs).expect("modify priority");
        app.refresh_from_backend();
        let updated = app.task(&plain).unwrap().clone();
        assert_eq!(updated.prio, 'H');
        let pending = app.run_backend(&["+PENDING", "export"]).expect("re-export");
        let values: Vec<serde_json::Value> = serde_json::from_str(&pending).unwrap();
        let updated_json = values
            .iter()
            .find(|v| v.get("uuid").and_then(|x| x.as_str()) == Some(plain.as_str()))
            .expect("plain in re-export");
        let fresh_urgency = updated_json
            .get("urgency")
            .and_then(|x| x.as_f64())
            .unwrap_or(0.0) as f32;
        assert!(
            (updated.urgency - fresh_urgency).abs() < f32::EPSILON,
            "urgency refreshes after save"
        );

        // Native `depends` blocks live with a blocked marker.
        let blocker_add = backend::build_add_args("live blocker", "", "", None, &[], &[], "", "");
        let refs: Vec<&str> = blocker_add.iter().map(String::as_str).collect();
        app.run_backend(&refs).expect("add blocker");
        app.refresh_from_backend();
        let blocker = uuid_of(&app, "live blocker");
        let link = backend::build_depends_add_args(&mapped, &blocker);
        let refs: Vec<&str> = link.iter().map(String::as_str).collect();
        app.run_backend(&refs).expect("link depends");
        app.refresh_from_backend();
        assert!(app.is_blocked(&mapped), "open dep blocks");

        // Refresh holds the cursor by `uuid` and picks up external adds.
        goto(&mut app, &mapped);
        let external = backend::build_add_args("external live", "", "", None, &[], &[], "", "");
        let refs: Vec<&str> = external.iter().map(String::as_str).collect();
        app.run_backend(&refs).expect("external add");
        app.refresh_from_backend();
        assert!(
            app.tasks.iter().any(|t| t.desc == "external live"),
            "refresh shows the external task"
        );
        assert!(
            app.tasks.iter().any(|t| t.uuid == blocker),
            "refresh keeps the depends link"
        );
        assert_eq!(
            app.cursor_id().as_deref(),
            Some(mapped.as_str()),
            "cursor holds by uuid across refresh"
        );
    }

    #[test]
    fn backend_live_tail_cap_and_view_filter() {
        let mut app = backend_app();
        app.view = View::All;
        // Twelve pending tasks with alternating projects for view filtering.
        for i in 0..12 {
            let project = if i % 2 == 0 { "work" } else { "personal" };
            let desc = format!("tail live {i:02}");
            let add = backend::build_add_args(&desc, project, "", None, &[], &[], "", "");
            let refs: Vec<&str> = add.iter().map(String::as_str).collect();
            app.run_backend(&refs).expect("seed add");
        }
        app.refresh_from_backend();
        assert!(
            !app.toast.starts_with("refresh failed"),
            "live refresh: {}",
            app.toast
        );
        assert_eq!(app.open_rows().len(), 12, "twelve live pending tasks");
        // Complete everything in one backend round-trip.
        let ids: Vec<String> = app.tasks.iter().map(|t| t.uuid.clone()).collect();
        app.complete_ids(&ids);
        assert_eq!(app.toast, "completed 12 task(s)");
        assert!(
            !app.toast.starts_with("refresh failed"),
            "completion refreshes: {}",
            app.toast
        );
        // Tail is capped at 10 by `end` desc.
        app.view = View::All;
        let tail = app.completed_rows();
        assert_eq!(tail.len(), 10, "live tail capped at 10");
        let ends: Vec<Option<String>> = tail
            .iter()
            .map(|id| app.task(id).unwrap().end.clone())
            .collect();
        for pair in ends.windows(2) {
            assert!(pair[0] >= pair[1], "tail sorts by end desc");
        }
        // View filter still applies inside the cap.
        app.view = View::Project("work".into());
        let work_tail = app.completed_rows();
        assert!(!work_tail.is_empty(), "project tail keeps matches");
        for id in &work_tail {
            assert_eq!(app.task(id).unwrap().project, "work");
        }
        assert!(work_tail.len() <= 10, "project tail respects cap");
        app.view = View::Project("personal".into());
        let personal_tail = app.completed_rows();
        for id in &personal_tail {
            assert_eq!(app.task(id).unwrap().project, "personal");
        }
        // Deleted never renders in any view.
        app.view = View::All;
        let gone = tail[0].clone();
        let refs = [
            String::from("rc.confirmation=no"),
            gone.clone(),
            String::from("delete"),
        ];
        let args: Vec<&str> = refs.iter().map(String::as_str).collect();
        app.run_backend(&args).expect("delete completed");
        app.refresh_from_backend();
        assert!(!app.completed_rows().contains(&gone));
        assert!(!app.open_rows().contains(&gone));
    }

    #[test]
    fn backend_failure_toast_preserves_state() {
        let mut app = backend_app();
        app.view = View::All;
        seed(&mut app, &["keep me"]);
        let before = app.open_rows();
        // Failed write: unknown `uuid` errors with stderr on stdout-free output.
        app.complete_ids(&["00000000-0000-0000-0000-000000000000".to_string()]);
        assert!(
            app.toast.starts_with("done failed:"),
            "write stderr surfaces: {}",
            app.toast
        );
        assert_eq!(app.open_rows(), before, "failed write keeps the list");
        assert_eq!(app.cursor, 0, "failed write keeps the cursor");
        // Failed refresh: a file as data dir cannot export.
        let file = app.taskdata_dir.as_ref().unwrap().join("taskrc");
        app.taskdata_dir = Some(file);
        app.refresh_from_backend();
        assert!(
            app.toast.starts_with("refresh failed:"),
            "refresh stderr surfaces: {}",
            app.toast
        );
        assert_eq!(app.open_rows(), before, "failed refresh keeps the list");
    }

    #[test]
    fn list_is_primary_only_with_air_and_no_subtask_counts() {
        let mut app = App::new();
        // Sections render in project views; Today/All stay flat.
        app.view = View::Project("work".into());
        // A completed work top so the dimmed tail renders in this view too.
        let short = app.next_id;
        app.next_id += 1;
        app.tasks.push(Task {
            uuid: mock_uuid(short),
            short_id: Some(short),
            desc: "Filed expense report".into(),
            project: "work".into(),
            due: "—".into(),
            due_raw: None,
            prio: 'L',
            tags: vec![],
            section: "Later".into(),
            urgency: 0.0,
            active: false,
            status: "completed".into(),
            depends: Vec::new(),
            recur: "none".into(),
            recur_parent: String::new(),
            start_end: "done".into(),
            start_raw: None,
            end: Some("20261110T120000Z".into()),
            annotations: Vec::new(),
        });
        app.cursor_to_first_task();
        let c = draw(&app);
        assert!(c.contains("NOW") && c.contains("NEXT"), "sections render");
        assert!(c.contains("Add a task"), "quick-add renders");
        assert!(c.contains("COMPLETED"), "completed tail renders");
        assert!(c.contains('●'), "priority dot renders");
        assert!(!c.contains("depends:"), "no inline secondary fields");
        assert!(
            ["11.4", "9.5", "8.2", "7.7"].iter().any(|u| c.contains(u)),
            "urgency numbers render on the right"
        );
        assert!(!c.contains("subtasks"), "no n/m subtask counts in rows");
        assert!(!c.contains("1/2"), "no progress fraction in rows");
        assert!(c.contains("blocked"), "blocked parent still marked");

        // Cursor is a single accent bar and moving it never expands rows.
        assert_eq!(c.matches('▎').count(), 1, "one cursor bar");
        let rows_before = app.open_rows();
        app.move_down();
        app.move_down();
        let rows_after = app.open_rows();
        assert_eq!(
            rows_before.len(),
            rows_after.len(),
            "cursor keeps row count"
        );
        let moved = draw(&app);
        assert_eq!(moved.matches('▎').count(), 1, "cursor bar stays single");
        assert!(!moved.contains("depends:"), "move reveals no extra lines");

        // Sorting is section-then-urgency with precise float order.
        let rows = app.open_rows();
        let mut last_order = 0;
        let mut last_urgency = f32::MAX;
        for id in &rows {
            let t = app.task(id).unwrap();
            let order = app.section_rank(&t.section);
            assert!(order >= last_order, "sections stay grouped");
            if order == last_order {
                assert!(
                    t.urgency <= last_urgency + f32::EPSILON,
                    "urgency descends within {}",
                    t.section
                );
            }
            last_order = order;
            last_urgency = t.urgency;
        }
        // 7.7 must outrank 7.0: float precision, not truncated ints.
        let pos8 = rows.iter().position(|id| *id == mock_uuid(8)).unwrap();
        let pos6 = rows.iter().position(|id| *id == mock_uuid(6)).unwrap();
        assert!(pos8 < pos6, "higher urgency surfaces first");

        // Native `depends` keeps rows flat: blocked tasks stay visible
        // with a `blocked` trailer, no nesting or indentation.
        assert!(
            rows.contains(&mock_uuid(3)),
            "blocked task stays a top-level row"
        );
        assert!(
            rows.contains(&mock_uuid(12)),
            "blocker stays a top-level row"
        );
        let lines = draw_lines(&app);
        let blocked_idx = lines
            .iter()
            .position(|l| l.contains("Reply to Nadia"))
            .unwrap();
        let blocker_idx = lines
            .iter()
            .position(|l| l.contains("Draft cover email"))
            .unwrap();
        assert!(
            lines[blocked_idx + 1].contains("blocked"),
            "blocked task carries its metadata trailer"
        );
        assert!(
            !lines[blocker_idx + 1].contains("blocked"),
            "open blocker carries no trailer"
        );
        assert!(!lines[blocked_idx].contains("1/2"), "no count on rows");

        // Overdue rows carry no trailer: the red due label is the only
        // mention, so the word appears exactly once on the row.
        app.view = View::All;
        let all_lines = draw_lines(&app);
        let overdue_idx = all_lines
            .iter()
            .position(|l| l.contains("Pay domain renewal"))
            .expect("overdue row renders in All");
        assert_eq!(
            all_lines[overdue_idx + 1].matches("overdue").count(),
            1,
            "the due label is the only overdue mention"
        );
    }

    #[test]
    fn views_sort_search_and_bulk() {
        let mut app = App::new();
        let inbox = app.count_for(&View::Inbox);
        let all = app.count_for(&View::All);
        assert!(all > inbox, "All covers more than Inbox");
        app.view = View::All;
        app.cursor_to_first_task();
        assert!(app.open_rows().len() >= 6);
        let start = app.cursor;
        app.move_down();
        app.move_down();
        app.move_down();
        assert_eq!(app.cursor, start + 3);
        app.view = View::Search;
        app.search_input = true;
        app.search_query = "nadia".into();
        app.search_cursor = 5;
        app.cursor = 0;
        app.clamp_cursor();
        assert_eq!(app.open_rows().len(), 1, "search isolates one task");
        let c = draw(&app);
        assert!(c.contains("nadia"), "search query renders");
        assert!(!c.contains("nadia▌"), "no inline cursor glyph");
        assert!(c.contains("SEARCH"), "search hints show in the footer");
        assert!(!c.contains("Add a task"), "Search offers no add box");
        let (_, cursor) = draw_with_cursor(&app);
        // List pane starts at x=0; the query is 5 cells wide with no indent.
        assert_eq!(
            cursor,
            Position::new(5, 4),
            "input mode puts the terminal caret after the query"
        );
        app.search_input = false;
        let c = draw(&app);
        assert!(c.contains("(i input)"), "list mode hints at `i`");
        let (_, cursor) = draw_with_cursor(&app);
        assert_eq!(
            cursor,
            Position::new(119, 39),
            "list mode leaves the terminal cursor hidden"
        );
        // Leaving Search clears the query so results never leak.
        app.switch_view(3);
        assert!(app.view == View::All);
        assert!(app.search_query.is_empty(), "query clears on exit");
        for i in 0..3 {
            let rows = app.open_rows();
            app.selected.insert(rows[i].clone());
        }
        let c = draw(&app);
        assert!(c.contains("3 selected"), "bulk bar shows count");
        app.exit_visual(false);
        assert!(app.selected.is_empty(), "esc clears selection");
    }

    #[test]
    fn single_start_done_delete_confirm_and_edit_hint() {
        let mut app = App::new();
        app.view = View::All;
        app.cursor_to_first_task();
        // `s` toggles start/stop with Today membership.
        let id = app.cursor_id().expect("cursor on a task");
        let was_active = app.task(&id).unwrap().active;
        app.single_start_stop();
        let t = app.task(&id).unwrap();
        assert_eq!(t.active, !was_active);
        assert_eq!(t.status, if !was_active { "active" } else { "pending" });
        let c = draw(&app);
        if !was_active {
            assert!(!c.contains("● active"), "no active text on start");
            assert!(c.contains("◉"), "started task gains the start indicator");
            app.view = View::Today;
            app.cursor_to_first_task();
            assert!(app.open_rows().contains(&id), "active appears in Today");
            app.view = View::All;
            app.cursor_to_first_task();
        }
        // `d` completes into the dimmed tail.
        let id = app.cursor_id().expect("cursor on a task");
        app.single_done();
        assert_eq!(app.task(&id).unwrap().status, "completed");
        assert!(!app.open_rows().contains(&id));
        assert!(app.completed_rows().contains(&id));
        // `D` needs `y`; `n`/esc cancels.
        app.cursor_to_first_task();
        let id = app.cursor_id().expect("cursor on a task");
        app.request_delete();
        assert!(app.confirm_delete.is_some(), "delete asks to confirm");
        app.confirm_delete = None;
        assert!(app.task(&id).unwrap().status != "deleted");
        app.request_delete();
        app.confirm_delete_yes();
        assert_eq!(app.task(&id).unwrap().status, "deleted");
        // `e` in the list does nothing (no inline edit, no hint).
        app.popup = None;
        app.clear_toast();
        let c = draw(&app);
        assert!(!c.contains("select a task with Enter first"));
        assert!(app.popup.is_none(), "e never edits inline");
    }

    #[test]
    fn visual_range_bulk_and_help_match_keymap() {
        let mut app = App::new();
        app.view = View::All;
        app.cursor_to_first_task();
        // `space` toggles one row.
        let id = app.cursor_id().expect("cursor on a task");
        app.toggle_select(id.clone());
        assert!(app.selected.contains(&id));
        app.toggle_select(id.clone());
        assert!(!app.selected.contains(&id));
        // `v` range extends with `j/k` (headers are skipped).
        app.cursor_to_first_task();
        app.enter_visual();
        app.move_down();
        app.move_down();
        assert!(
            app.selected.len() >= 2,
            "VISUAL covers the range (headers skipped)"
        );
        app.exit_visual(false);
        assert!(app.selected.is_empty(), "esc cancels VISUAL");
        // Bulk move/tags/done clear the selection.
        let rows = app.open_rows();
        app.selected.insert(rows[0].clone());
        app.selected.insert(rows[1].clone());
        app.bulk_move_project();
        assert!(app.selected.is_empty(), "bulk move clears");
        let rows = app.open_rows();
        app.selected.insert(rows[0].clone());
        app.bulk_toggle_tag();
        assert!(app.task(&rows[0]).unwrap().tags.contains(&"urgent".into()));
        assert!(app.selected.is_empty(), "bulk retag clears");
        let rows = app.open_rows();
        app.selected.insert(rows[0].clone());
        let doomed = rows[0].clone();
        app.complete_ids(std::slice::from_ref(&doomed));
        assert_eq!(app.task(&doomed).unwrap().status, "completed");
        assert!(app.selected.is_empty(), "bulk done clears");
        // `?` overlay lists exactly the minimal keymap, grouped for reading.
        app.show_help = true;
        let c = draw(&app);
        for key in [
            "List",
            "Edit",
            "Search",
            "j/k",
            "space",
            "a",
            "e",
            "d",
            "s",
            "D",
            "m",
            "t",
            "/",
            "?",
            "q",
            "v",
            "u",
            "i input",
            "esc clears/returns",
            "1-3",
            "done in VIEW",
        ] {
            assert!(c.contains(key), "help lists {key}");
        }
        assert!(!c.contains("FIND"), "the find mode is gone from the keymap");
        assert!(
            !c.contains("open task · toggle section header"),
            "list Enter hint is gone"
        );
        assert!(!c.contains("refresh from backend"), "refresh hint is gone");
    }

    #[test]
    fn list_footer_hints_drop_enter_and_refresh_add_delete_and_slash() {
        let mut app = App::new();
        app.view = View::All;
        let c = draw(&app);
        assert!(c.contains("D delete"), "footer advertises delete");
        assert!(c.contains("/ search"), "footer advertises slash search");
        assert!(
            !c.contains("Enter open/toggle"),
            "list Enter hint is gone from the footer"
        );
        assert!(
            !c.contains("r refresh"),
            "refresh hint is gone from the footer"
        );
    }

    #[test]
    fn slash_shortcut_enters_search_input_mode() {
        let mut app = App::new();
        app.view = View::All;
        // `/` in the list keymap jumps straight to Search.
        app.switch_view(0);
        assert_eq!(app.view, View::Search, "`/` lands in the search view");
        assert!(app.search_input, "search starts in input mode");
    }

    #[test]
    fn view_filters_counts_totals_and_empty_states() {
        let mut app = App::new();

        // Inbox: open tasks with no project, due date ignored.
        app.view = View::Inbox;
        for id in app.open_rows() {
            let t = app.task(&id).unwrap();
            assert!(
                t.status == "pending" || t.status == "active",
                "Inbox shows open tasks only"
            );
            assert!(t.project.is_empty(), "Inbox has no project");
        }
        // Assigned task leaves Inbox: every project task is absent here.
        let inbox_ids = app.open_rows();
        for t in app.tasks.iter().filter(|t| !t.project.is_empty()) {
            assert!(
                !inbox_ids.contains(&t.uuid),
                "project task stays out of Inbox"
            );
        }
        // Active intake qualifies: craft one and check the filter.
        let mut active_intake = Task {
            uuid: mock_uuid(900),
            short_id: Some(900),
            desc: "active intake".into(),
            project: "".into(),
            due: "".into(),
            due_raw: None,
            prio: 'M',
            tags: vec![],
            section: "Now".into(),
            urgency: 6.0,
            active: true,
            status: "active".into(),
            depends: Vec::new(),
            recur: "none".into(),
            recur_parent: String::new(),
            start_end: "now".into(),
            start_raw: None,
            end: None,
            annotations: Vec::new(),
        };
        assert!(
            View::Inbox.matches(&active_intake),
            "active project-less task stays in Inbox"
        );
        // Due date is ignored: a dated project-less task still qualifies.
        active_intake.due = "tomorrow".into();
        assert!(
            View::Inbox.matches(&active_intake),
            "due date keeps task in Inbox"
        );
        active_intake.status = "pending".to_string();
        active_intake.active = false;
        assert!(
            View::Inbox.matches(&active_intake),
            "pending intake belongs in Inbox"
        );

        // Today: overdue, due-today, or active — pending/active only.
        app.view = View::Today;
        let today_ids = app.open_rows();
        for wanted in [mock_uuid(3), mock_uuid(4), mock_uuid(5), mock_uuid(6)] {
            assert!(
                today_ids.contains(&wanted),
                "Today shows overdue/due-today/active task {wanted}"
            );
        }
        assert!(
            !today_ids.contains(&mock_uuid(2)),
            "plain Next task without due stays out of Today"
        );
        for id in &today_ids {
            let t = app.task(id).unwrap();
            assert!(
                t.status == "pending" || t.status == "active",
                "Today limited to pending/active"
            );
        }

        // All covers every pending/active top; project views match exactly.
        app.view = View::All;
        let all_tops = app
            .tasks
            .iter()
            .filter(|t| t.status == "pending" || t.status == "active")
            .count();
        assert_eq!(app.count_for(&View::All), all_tops);
        app.view = View::Project("work".into());
        for id in app.open_rows() {
            // Flat native `depends`: every row is top-level.
            let t = app.task(&id).unwrap();
            assert_eq!(t.project, "work", "project view isolates tasks");
        }
        assert!(
            !app.open_rows().iter().any(|id| {
                let t = app.task(id).unwrap();
                t.project == "personal"
            }),
            "other projects stay out of the work view"
        );

        // Tab counts match each permanent view's filter.
        for (label, view) in [
            ("Search", View::Search),
            ("Inbox", View::Inbox),
            ("Today", View::Today),
            ("All", View::All),
        ] {
            assert_eq!(
                app.count_for(&view),
                app.tasks.iter().filter(|t| view.matches(t)).count(),
                "tab count for {label}"
            );
        }
        app.view = View::Inbox;
        let (open_inbox, _, _) = app.header_stats();
        let inbox_rows = app.open_rows();
        assert_eq!(open_inbox, inbox_rows.len(), "Inbox total is its rows");
        app.view = View::Today;
        let (open_today, today_due, today_active) = app.header_stats();
        assert_eq!(
            open_today,
            app.open_rows().len(),
            "top-line open is current rows"
        );
        assert!(
            today_due > 0 && today_active > 0,
            "Today has due + active totals"
        );
        assert_ne!(
            inbox_rows,
            app.open_rows(),
            "switching views changes the rows"
        );
        let c = draw(&app);
        assert!(c.contains("open"), "top line shows open total");
        assert!(c.contains("due"), "top line keeps due total pinned");
        let header = draw_lines(&app)[0].clone();
        for tab in ["Inbox", "Today", "All", "personal", "reading", "work"] {
            assert!(header.contains(tab), "tab bar lists {tab}");
        }
        assert!(!header.contains("search"), "Search is not a header tab");

        // Empty views explain themselves with no rows.
        app.view = View::Project("zzz-empty".into());
        assert!(app.open_rows().is_empty());
        assert!(app.completed_rows().is_empty());
        let c = draw(&app);
        assert!(
            c.contains(View::Project("zzz-empty".into()).empty_hint()),
            "empty project shows its hint"
        );
        assert!(!c.contains("Nadia"), "empty view shows no task rows");
    }

    #[test]
    fn inspector_splits_on_wide_and_overlays_on_narrow() {
        let mut app = App::new();
        let t = app.task(&mock_uuid(3)).unwrap().clone();
        app.popup = Some(Popup::open(&app, &t));

        let lines = draw_lines(&app);
        assert!(
            lines[2].contains("Inbox") && lines[2].contains('╭'),
            "wide frame keeps list and inspector side by side"
        );
        assert!(
            lines
                .iter()
                .any(|l| l.contains("TASK") && l.contains("VIEW")),
            "split inspector header renders"
        );

        let mut term = Terminal::new(TestBackend::new(90, 40)).unwrap();
        term.draw(|f| render(f, &app)).unwrap();
        let buf = term.backend().buffer();
        let lines: Vec<String> = buf
            .content()
            .chunks(90)
            .map(|row| row.iter().map(|c| c.symbol()).collect::<String>())
            .collect();
        assert!(
            lines[2].trim_start().starts_with('╭'),
            "narrow frame turns the inspector into a full-body overlay"
        );
        assert!(
            !lines[2].contains("Inbox"),
            "overlay covers the list header"
        );
        assert!(
            lines[39].contains("h list"),
            "footer shows inspector hints without a mode chip"
        );
        assert!(!lines[39].contains(" VIEW "), "footer carries no mode chip");
        // With list focus behind the inspector, the footer advertises the
        // full list hotkeys instead of just move/inspector/close.
        app.popup_list_focus = true;
        let mut term = Terminal::new(TestBackend::new(90, 40)).unwrap();
        term.draw(|f| render(f, &app)).unwrap();
        let buf = term.backend().buffer();
        let lines: Vec<String> = buf
            .content()
            .chunks(90)
            .map(|row| row.iter().map(|c| c.symbol()).collect::<String>())
            .collect();
        assert!(
            lines[39].contains("s start"),
            "footer advertises list hotkeys behind the inspector"
        );
    }

    #[test]
    fn project_tabs_are_derived_sorted_and_cycle() {
        let mut app = App::new();
        // Derived from mock tasks, sorted A–Z, no hardcoded leftovers.
        assert_eq!(app.projects, vec!["personal", "reading", "work"]);

        // Header order: Inbox, Today, All, then every project.
        let tabs = app.tab_views();
        assert_eq!(
            tabs,
            vec![
                View::Inbox,
                View::Today,
                View::All,
                View::Project("personal".into()),
                View::Project("reading".into()),
                View::Project("work".into()),
            ]
        );
        app.switch_to_view(View::Inbox);
        let header = draw_lines(&app)[0].clone();
        for tab in ["Inbox", "Today", "All", "personal", "reading", "work"] {
            assert!(header.contains(tab), "header tab {tab} renders");
        }
        assert!(
            !header.contains("search"),
            "Search is not a header tab when inactive"
        );

        // Cycling walks permanent views and every project with no wrap.
        app.cycle_view(1);
        assert_eq!(app.view, View::Today);
        app.cycle_view(1);
        assert_eq!(app.view, View::All);
        app.cycle_view(1);
        assert_eq!(app.view, View::Project("personal".into()));
        app.cycle_view(1);
        assert_eq!(app.view, View::Project("reading".into()));
        app.cycle_view(1);
        assert_eq!(app.view, View::Project("work".into()));
        app.cycle_view(1);
        assert_eq!(app.view, View::Project("work".into()), "no wrap past last");
        app.cycle_view(-1);
        assert_eq!(app.view, View::Project("reading".into()));
        app.switch_to_view(View::Inbox);
        app.cycle_view(-1);
        assert_eq!(app.view, View::Inbox, "no wrap before first");

        // Number keys: 1–3 are permanent views, 4+ pick project tabs.
        app.switch_view(3);
        assert_eq!(app.view, View::All);
        app.switch_view(4);
        assert_eq!(app.view, View::Project("personal".into()));
        app.switch_view(5);
        assert_eq!(app.view, View::Project("reading".into()));

        // Search is hotkey-only: it is not in the cycle and `/` remembers
        // the active project to return to.
        app.switch_to_view(View::Project("work".into()));
        app.open_search();
        assert_eq!(app.view, View::Search);
        assert!(app.search_input, "Search opens in input mode");
        let header = draw_lines(&app)[0].clone();
        assert!(
            header.contains("SEARCH"),
            "Search shows a SEARCH pill in the header while open"
        );
        app.leave_search();
        assert_eq!(app.view, View::Project("work".into()));
        app.open_search();
        app.cycle_view(-1);
        assert_eq!(
            app.view,
            View::Project("work".into()),
            "Tab leaves Search for its origin"
        );

        // Project tab counts match their views.
        for project in &app.projects {
            assert_eq!(
                app.count_for(&View::Project(project.clone())),
                app.tasks
                    .iter()
                    .filter(|t| is_open(t) && &t.project == project)
                    .count()
            );
        }

        // The Search count follows the current query.
        app.open_search();
        app.search_query = "nadia".into();
        assert_eq!(
            app.count_for(&View::Search),
            1,
            "Search count reflects query"
        );
        app.search_query.clear();
        app.search_input = false;
        app.leave_search();

        // A new project appears in sorted position without a restart.
        let short = app.next_id;
        app.next_id += 1;
        let mut extra = app.tasks[2].clone();
        extra.uuid = mock_uuid(short);
        extra.short_id = Some(short);
        extra.desc = "alpha task".into();
        extra.project = "alpha".into();
        app.tasks.push(extra);
        app.sync_projects();
        assert_eq!(app.projects, vec!["alpha", "personal", "reading", "work"]);

        // The empty store keeps the three permanent tabs and drops projects.
        app.tasks.clear();
        app.sync_projects();
        assert!(app.projects.is_empty(), "empty store has no projects");
        let header = draw_lines(&app)[0].clone();
        assert!(header.contains("Inbox") && header.contains("Today") && header.contains("All"));
        assert!(!header.contains("work"), "no stale project tab");

        // A stale project leaves and the active view falls back to All.
        app.tasks = mock_tasks();
        app.sync_projects();
        app.view = View::Project("reading".into());
        app.tasks.retain(|t| t.project != "reading");
        app.sync_projects();
        assert!(
            !app.projects.contains(&"reading".to_string()),
            "stale project leaves the tabs"
        );
        assert!(app.view == View::All, "active project falls back to All");
    }

    #[test]
    fn project_colors_are_deterministic_and_shared() {
        let mut app = App::new();
        // Stability: the same set rebuilds to the same colors.
        let rebuilt = build_project_colors(&app.projects);
        assert_eq!(app.project_colors, rebuilt, "colors are stable");
        // Uniqueness within the palette for the current set.
        let mut seen = HashSet::new();
        for (name, color) in &app.project_colors {
            assert!(
                PROJECT_PALETTE.contains(color),
                "{name} uses a palette accent"
            );
            assert!(seen.insert(*color), "{name} collides in the current set");
        }
        // Collision resolution: two names sharing a preferred slot still
        // land on distinct colors in sorted order.
        let mut colliding: Option<(String, String)> = None;
        let candidates: Vec<String> = (0..50).map(|i| format!("proj-{i}")).collect();
        for (i, a) in candidates.iter().enumerate() {
            for b in candidates.iter().skip(i + 1) {
                if project_preferred_slot(a) == project_preferred_slot(b) {
                    colliding = Some((a.clone(), b.clone()));
                    break;
                }
            }
            if colliding.is_some() {
                break;
            }
        }
        let (a, b) = colliding.expect("a colliding pair exists");
        let mut pair = vec![a.clone(), b.clone()];
        pair.sort();
        let colors = build_project_colors(&pair);
        assert_ne!(
            colors[&pair[0]], colors[&pair[1]],
            "colliding pair resolves to distinct colors"
        );
        assert_eq!(
            colors[&pair[0]],
            PROJECT_PALETTE[project_preferred_slot(&pair[0])],
            "first in sorted order keeps its preferred slot"
        );

        // Row metadata uses the deterministic palette color.
        app.view = View::All;
        let backend = TestBackend::new(120, 40);
        let mut term = Terminal::new(backend).unwrap();
        term.draw(|f| render(f, &app)).unwrap();
        let buf = term.backend().buffer();
        let w = buf.area.width as usize;
        let lines: Vec<String> = buf
            .content()
            .chunks(w)
            .map(|row| row.iter().map(|c| c.symbol()).collect::<String>())
            .collect();
        let row_y = lines
            .iter()
            .position(|l| l.contains("Reply to Nadia"))
            .expect("work row renders");
        let meta = &lines[row_y + 1];
        let rel = meta.find("work").expect("row project renders");
        let row_fg = buf[(rel as u16, (row_y + 1) as u16)].fg;
        assert_eq!(
            row_fg,
            app.project_color("work"),
            "rendered color matches the map"
        );
    }

    #[test]
    fn inbox_renders_flat_by_urgency() {
        let mut app = App::new();
        // A Later task with top urgency outranks Now tasks in Inbox.
        let urgent_id = app.next_id;
        app.next_id += 1;
        app.tasks.push(Task {
            uuid: mock_uuid(urgent_id),
            short_id: Some(urgent_id),
            desc: "urgent intake".into(),
            project: "".into(),
            due: "tomorrow".into(),
            due_raw: None,
            prio: 'H',
            tags: vec![],
            section: "Later".into(),
            urgency: 9.0,
            active: false,
            status: "pending".into(),
            depends: Vec::new(),
            recur: "none".into(),
            recur_parent: String::new(),
            start_end: "not started".into(),
            start_raw: None,
            end: None,
            annotations: Vec::new(),
        });
        app.view = View::Inbox;
        let rows = app.open_rows();
        assert_eq!(
            rows[0],
            mock_uuid(urgent_id),
            "urgency outranks section in Inbox"
        );
        // Flat render: rows with no uppercase section labels.
        let c = draw(&app);
        for label in ["NOW", "NEXT", "LATER"] {
            assert!(!c.contains(label), "no {label} header in Inbox");
        }
        // Native `depends` stays flat in Inbox: both tasks are top rows.
        let first_id = app.next_id;
        app.next_id += 1;
        app.tasks.push(Task {
            uuid: mock_uuid(first_id),
            short_id: Some(first_id),
            desc: "flat first".into(),
            project: "".into(),
            due: "".into(),
            due_raw: None,
            prio: 'M',
            tags: vec![],
            section: "Now".into(),
            urgency: 1.0,
            active: false,
            status: "pending".into(),
            depends: Vec::new(),
            recur: "none".into(),
            recur_parent: String::new(),
            start_end: "not started".into(),
            start_raw: None,
            end: None,
            annotations: Vec::new(),
        });
        let second_id = app.next_id;
        app.next_id += 1;
        app.tasks.push(Task {
            uuid: mock_uuid(second_id),
            short_id: Some(second_id),
            desc: "flat second".into(),
            project: "".into(),
            due: "".into(),
            due_raw: None,
            prio: 'M',
            tags: vec![],
            section: "Now".into(),
            urgency: 1.0,
            active: false,
            status: "pending".into(),
            depends: vec![mock_uuid(first_id)],
            recur: "none".into(),
            recur_parent: String::new(),
            start_end: "not started".into(),
            start_raw: None,
            end: None,
            annotations: Vec::new(),
        });
        let rows = app.open_rows();
        assert!(rows.contains(&mock_uuid(first_id)), "blocker is top-level");
        assert!(
            rows.contains(&mock_uuid(second_id)),
            "blocked task is top-level"
        );
        assert!(app.is_blocked(&mock_uuid(second_id)), "dep blocks");
    }

    #[test]
    fn rows_use_two_lines_without_id_and_blocked_by_ids() {
        let mut app = App::new();
        app.view = View::All;
        let lines = draw_lines(&app);
        let idx = lines
            .iter()
            .position(|l| l.contains("Reply to Nadia"))
            .expect("task row renders");
        let title = lines[idx].trim();
        let meta = lines[idx + 1].trim();

        // Description line: priority dot, start column, urgency — no ID.
        assert!(title.contains('●'), "priority dot renders");
        assert!(!title.contains('3'), "main list drops the ID column");
        assert!(title.contains("11.4"), "urgency renders on the right");
        assert!(title.contains("Reply to Nadia"), "description renders");

        // Metadata line: due, project, tags, blocked-by trailer in order.
        for part in ["today 17:00", "work", "#legal", "blocked by 12"] {
            assert!(meta.contains(part), "metadata shows {part}");
        }
        let due = meta.find("today 17:00").unwrap();
        let project = meta.find("work").unwrap();
        let tag = meta.find("#legal").unwrap();
        let blocked = meta.find("blocked by 12").unwrap();
        assert!(due < project && project < tag && tag < blocked);
        assert!(!meta.contains("● work"), "project name has no dot prefix");
        assert!(
            meta.starts_with("today 17:00"),
            "metadata starts with due, no leading separator"
        );

        // A task with no due date shows no dash and no leading separator:
        // the line starts directly with the first present segment.
        let idx = lines
            .iter()
            .position(|l| l.contains("Read Shape Up"))
            .expect("project-less row renders");
        let meta = lines[idx + 1].trim();
        assert_eq!(meta, "reading", "only the present segment renders, bare");
        assert!(!meta.contains('—'), "absent due leaves no dash");
        assert!(!meta.contains('·'), "no separator without a due date");
    }

    #[test]
    fn main_list_has_no_id_column_but_tail_keeps_uuid_tasks() {
        let mut app = App::new();
        app.view = View::All;
        // Completed tasks export `"id": 0`, so they carry no numeric ID.
        // They still land in the tail without an ID column in the main list.
        app.tasks.push(Task {
            uuid: "abcdefgh-done-uuid".into(),
            short_id: None,
            desc: "old done thing".into(),
            project: "".into(),
            due: "—".into(),
            due_raw: None,
            prio: 'L',
            tags: vec![],
            section: "".into(),
            urgency: 0.5,
            active: false,
            status: "completed".into(),
            depends: Vec::new(),
            recur: "none".into(),
            recur_parent: String::new(),
            start_end: "not started".into(),
            start_raw: None,
            end: Some("20261111T120000Z".into()),
            annotations: Vec::new(),
        });
        assert!(
            app.completed_rows()
                .contains(&"abcdefgh-done-uuid".to_string()),
            "uuid-only task lands in the tail"
        );
        let lines = draw_lines(&app);
        let row = lines
            .iter()
            .find(|l| l.contains("Reply to Nadia"))
            .expect("open row renders");
        // No short numeric ID before the description in the main list.
        assert!(!row.contains(" 3 "), "main list drops the ID column: {row}");
        assert!(row.contains("Reply to Nadia"), "description still renders");
    }

    #[test]
    fn narrow_width_truncates_and_cursor_stays_aligned() {
        let mut app = App::new();
        app.view = View::All;
        // Long values exceed every clamped column (desc 44, project 14,
        // tags 20) so narrow frames must truncate, never panic or overflow.
        let short = app.next_id;
        app.next_id += 1;
        app.tasks.push(Task {
            uuid: mock_uuid(short),
            short_id: Some(short),
            desc: "A very long description that must truncate in narrow frames".into(),
            project: "extraordinarily-long-project".into(),
            due: "today 17:00".into(),
            due_raw: None,
            prio: 'H',
            tags: vec![
                "planning".into(),
                "legal".into(),
                "admin".into(),
                "eng".into(),
                "infra".into(),
            ],
            section: "Now".into(),
            urgency: 9.0,
            active: false,
            status: "pending".into(),
            depends: Vec::new(),
            recur: "none".into(),
            recur_parent: String::new(),
            start_end: "not started".into(),
            start_raw: None,
            end: None,
            annotations: Vec::new(),
        });
        app.sync_projects();
        app.sync_sections();
        let long_id = mock_uuid(short);
        goto(&mut app, &long_id);
        let main_of = |l: &String| -> String { l.trim().to_string() };
        for width in [40u16, 60, 120] {
            let backend = TestBackend::new(width, 40);
            let mut term = Terminal::new(backend).unwrap();
            term.draw(|f| render(f, &app)).unwrap();
            let buf = term.backend().buffer();
            assert_eq!(buf.area.width, width, "frame keeps its width");
            let w = width as usize;
            let lines: Vec<String> = buf
                .content()
                .chunks(w)
                .map(|row| row.iter().map(|c| c.symbol()).collect::<String>())
                .collect();
            // Exactly one cursor marker, on the cursor task's row. The
            // description column stays fixed; truncation never moves it.
            let marked: Vec<&String> = lines.iter().filter(|l| main_of(l).contains('▎')).collect();
            assert_eq!(marked.len(), 1, "one cursor row at width {width}");
            assert!(
                main_of(marked[0]).contains("A very"),
                "cursor stays aligned with its row at width {width}"
            );
            if width >= 60 {
                assert!(
                    main_of(marked[0]).contains("A very"),
                    "description reads at width {width}"
                );
            }
        }
        // Long values truncate with `…` instead of overflowing the frame.
        let backend = TestBackend::new(40, 40);
        let mut term = Terminal::new(backend).unwrap();
        term.draw(|f| render(f, &app)).unwrap();
        let buf = term.backend().buffer();
        let lines: Vec<String> = buf
            .content()
            .chunks(40)
            .map(|row| row.iter().map(|c| c.symbol()).collect::<String>())
            .collect();
        let idx = lines
            .iter()
            .position(|l| l.contains("A very"))
            .expect("long row renders");
        assert!(lines[idx].contains('…'), "long description truncates");
        assert!(
            !lines[idx + 1].contains("extraordinarily-long-project"),
            "long project never overflows the frame"
        );
    }

    #[test]
    fn selection_uses_accent_and_cursor_wins() {
        let mut app = App::new();
        app.view = View::All;
        let selected = app.open_rows()[0].clone();
        let desc = app.task(&selected).unwrap().desc.clone();
        app.selected.insert(selected.clone());
        goto(&mut app, &selected);

        let list_of = |l: &String| -> String { l.trim().to_string() };
        let selected_row = |lines: &[String]| -> String {
            let line = lines
                .iter()
                .find(|l| list_of(l).contains(&desc))
                .expect("selected row renders");
            list_of(line)
        };

        // Cursor and selection on one row: the cursor bar wins.
        let lines = draw_lines(&app);
        let row = selected_row(&lines);
        assert!(row.starts_with('▎'), "cursor bar wins");
        assert!(!row.contains('▌'), "no second marker");
        let backend = TestBackend::new(120, 40);
        let mut term = Terminal::new(backend).unwrap();
        term.draw(|f| render(f, &app)).unwrap();
        let buf = term.backend().buffer();
        let y = lines
            .iter()
            .position(|l| list_of(l).contains(&desc))
            .unwrap();
        assert_eq!(buf[(28, y as u16)].bg, TINT, "coincident row stays tinted");

        // Cursor moves away: the selected row keeps ▌ plus its tint.
        let other = app.open_rows()[1].clone();
        goto(&mut app, &other);
        let lines = draw_lines(&app);
        let row = selected_row(&lines);
        assert!(row.starts_with('▌'), "selection accent renders");
        assert!(!row.contains('▎'), "cursor bar left the row");
        assert_eq!(app.selected.len(), 1, "selection count unchanged");
        let backend = TestBackend::new(120, 40);
        let mut term = Terminal::new(backend).unwrap();
        term.draw(|f| render(f, &app)).unwrap();
        let buf = term.backend().buffer();
        let y = lines
            .iter()
            .position(|l| list_of(l).contains(&desc))
            .unwrap();
        assert_eq!(buf[(28, y as u16)].bg, TINT, "selected row is tinted");
    }

    #[test]
    fn inbox_disables_section_keys() {
        let mut app = App::new();
        app.view = View::Inbox;
        app.cursor = 0;
        let id = app.cursor_id().unwrap();
        let section = app.task(&id).unwrap().section.clone();
        // `m` keeps the hidden section value and explains itself.
        app.move_section_forward();
        assert_eq!(
            app.task(&id).unwrap().section,
            section,
            "section preserved in Inbox"
        );
        assert!(
            app.toast.contains("project views"),
            "hint shown for m in Inbox"
        );
        // `A` / `R` / `X` refuse without opening a prompt or touching sections.
        let sections = app.sections.clone();
        for kind in [
            SectionPromptKind::Add,
            SectionPromptKind::Rename,
            SectionPromptKind::Remove,
        ] {
            app.open_section_prompt(kind);
            assert!(app.section_prompt.is_none(), "{kind:?} refused in Inbox");
            assert_eq!(app.sections, sections, "{kind:?} leaves sections alone");
            assert!(app.toast.contains("project views"));
        }
    }

    #[test]
    fn project_view_hides_project_name_but_popup_keeps_it() {
        let list_col = |l: &String| -> String { l.trim().to_string() };
        // Project views scope to one project, so the row drops the name.
        let mut app = App::new();
        app.view = View::Project("work".into());
        let lines = draw_lines(&app);
        let idx = lines
            .iter()
            .position(|l| list_col(l).contains("Reply to Nadia"))
            .expect("work row renders");
        assert!(
            !list_col(&lines[idx + 1]).contains("work"),
            "project view hides its own project name"
        );
        // Mixed views show the colored project name on the metadata line.
        app.view = View::All;
        let lines = draw_lines(&app);
        let idx = lines
            .iter()
            .position(|l| list_col(l).contains("Reply to Nadia"))
            .expect("All row renders");
        assert!(
            list_col(&lines[idx + 1]).contains("work"),
            "All shows the project name"
        );
        assert!(
            !list_col(&lines[idx + 1]).contains("● work"),
            "no project dot on rows"
        );
        // The Enter popup still shows the project field, without a dot.
        let t = app.task(&mock_uuid(3)).unwrap().clone();
        app.popup = Some(Popup::open(&app, &t));
        let c = draw(&app);
        assert!(c.contains("work"), "popup still shows the project");
        let lines = draw_lines(&app);
        let project_rows: Vec<_> = lines.iter().filter(|l| l.contains("project")).collect();
        assert!(!project_rows.is_empty(), "popup project row renders");
        for l in project_rows {
            assert!(!l.contains("● work"), "popup drops the dot, name only");
        }
    }

    #[test]
    fn quick_add_joins_lines_and_section_moves() {
        let mut app = App::new();
        app.view = View::Inbox;
        let before = app.open_rows().len();
        let tasks_before = app.tasks.len();
        app.add.desc = "hello\nworld\nagain".into();
        app.quick_add_commit();
        assert_eq!(app.open_rows().len(), before + 1);
        assert_eq!(app.tasks.len(), tasks_before + 1, "one Enter is one task");
        assert!(
            app.open_rows()
                .iter()
                .filter_map(|id| app.task(id))
                .any(|t| t.desc == "hello world again"),
            "pasted lines join into one task"
        );
        // View defaults ride along with the single create.
        app.view = View::Project("work".into());
        app.add.desc = "work quick item".into();
        app.quick_add_commit();
        assert_eq!(
            app.tasks
                .iter()
                .find(|t| t.desc == "work quick item")
                .unwrap()
                .project,
            "work",
            "project view supplies the project"
        );
        let first = app.open_rows()[0].clone();
        let sec = app.task(&first).unwrap().section.clone();
        goto(&mut app, &first);
        app.move_section_forward();
        assert_ne!(app.task(&first).unwrap().section, sec);
    }

    #[test]
    fn add_box_triggers_and_unique_hints() {
        use KeyCode as K;
        let mods = KeyModifiers::empty();
        let mut app = App::new();
        app.add_mode = true;
        app.add = AddDraft::fresh();
        // Typing in Desc, then `#` jumps to the tag picker (no memorized key).
        add_key(&mut app, K::Char('h'), mods);
        add_key(&mut app, K::Char('i'), mods);
        assert_eq!(app.add.desc, "hi");
        add_key(&mut app, K::Char('#'), mods);
        assert_eq!(app.add.focus, AddFocus::Tags);
        let c = draw(&app);
        assert!(c.contains("e.g. work, urgent"), "tags show a grey example");
        let sugs = add_suggestions(&app, &app.add);
        assert!(!sugs.is_empty(), "tag picker lists existing tags");
        let c = draw(&app);
        assert!(c.contains("#"), "tag chip renders");
        // j/k types into the filter (arrows/Tab own the list).
        add_key(&mut app, K::Char('j'), mods);
        add_key(&mut app, K::Char('k'), mods);
        assert_eq!(app.add.filter, "jk", "j/k stays typable for tags");
        assert_eq!(app.add.focus, AddFocus::Tags, "j/k never leaves the picker");
        app.add.filter.clear();
        app.add.picker = 0;
        // Arrows/Tab move the picker; Enter toggles into the draft.
        // The shared clear row sits first, so pick a real tag to toggle.
        let sugs = add_suggestions(&app, &app.add);
        assert_eq!(sugs[0], "no tags", "clear row leads the tag list");
        let first = sugs.iter().find(|s| *s != "no tags").cloned().unwrap();
        app.add.picker = sugs.iter().position(|s| s == &first).unwrap();
        add_key(&mut app, K::Down, mods);
        add_key(&mut app, K::Up, mods);
        add_key(&mut app, K::Tab, mods);
        app.add.picker = sugs.iter().position(|s| s == &first).unwrap();
        add_key(&mut app, K::Enter, mods);
        assert!(
            app.add.tags.contains(&first) || !app.add.tags.is_empty(),
            "Enter toggles a tag"
        );
        // The shared clear row empties the draft tags.
        app.add.picker = 0;
        add_key(&mut app, K::Enter, mods);
        assert!(app.add.tags.is_empty(), "no tags clears the draft tags");
        // `@` jumps to the project picker and Enter applies it.
        add_key(&mut app, K::Char('@'), mods);
        assert_eq!(app.add.focus, AddFocus::Project);
        let sugs = add_suggestions(&app, &app.add);
        assert_eq!(sugs[0], "no project", "clear row leads the project list");
        assert!(sugs.iter().any(|s| s == "work"), "project hints list work");
        app.add.picker = sugs.iter().position(|s| s == "work").unwrap();
        add_key(&mut app, K::Enter, mods);
        assert_eq!(app.add.focus, AddFocus::Desc, "apply returns to desc");
        assert!(!app.add.project.is_empty(), "project applied");
        // `!` jumps to the priority picker; 1/2/3 applies directly.
        add_key(&mut app, K::Char('!'), mods);
        assert_eq!(app.add.focus, AddFocus::Priority);
        add_key(&mut app, K::Char('1'), mods);
        assert_eq!(app.add.focus, AddFocus::Desc, "digit applies priority");
        assert!(app.add.prio_set, "priority recorded");
        let c = draw(&app);
        assert!(c.contains("! H"), "set priority chip renders");
        // `~` jumps to due presets with date-only hints.
        add_key(&mut app, K::Char('~'), mods);
        assert_eq!(app.add.focus, AddFocus::Due);
        let c = draw(&app);
        assert!(c.contains("e.g. 2026-11-20"), "due shows an example date");
        let sugs = add_suggestions(&app, &app.add);
        assert!(sugs.contains(&"today".to_string()), "due hints are dates");
        add_key(&mut app, K::Enter, mods);
        assert_eq!(app.add.focus, AddFocus::Desc, "apply returns to desc");
        assert!(!app.add.due.is_empty(), "due applied");
        // Enter in Desc commits but stays in add mode for rapid entry.
        app.add.desc = "triggered task".into();
        app.quick_add_commit();
        assert!(
            app.tasks.iter().any(|t| t.desc == "triggered task"),
            "commit creates the task"
        );
        assert!(app.add_mode, "add stays open after Enter");
        assert!(app.add.desc.is_empty(), "draft clears after commit");
    }

    #[test]
    fn add_box_shift_tab_reaches_both_ends() {
        use KeyCode as K;
        let mut app = App::new();
        app.add_mode = true;
        app.add.focus = AddFocus::Tags;
        let n = add_suggestions(&app, &app.add).len();
        assert!(n > 1, "fixture has several tags");
        // BackTab and shifted Tab move backward with the same clamp.
        add_key(&mut app, K::BackTab, KeyModifiers::empty());
        assert_eq!(app.add.picker, 0, "clamps at the first suggestion");
        add_key(&mut app, K::Tab, KeyModifiers::SHIFT);
        assert_eq!(app.add.picker, 0, "shifted Tab also clamps at the first");
        for _ in 0..n + 2 {
            add_key(&mut app, K::Tab, KeyModifiers::empty());
        }
        assert_eq!(app.add.picker, n - 1, "Tab reaches the last suggestion");
        add_key(&mut app, K::Tab, KeyModifiers::SHIFT);
        assert_eq!(app.add.picker, n - 2, "Shift+Tab steps backward");
        add_key(&mut app, K::BackTab, KeyModifiers::empty());
        assert_eq!(app.add.picker, n - 3, "BackTab steps backward");
        for _ in 0..n + 2 {
            add_key(&mut app, K::BackTab, KeyModifiers::empty());
        }
        assert_eq!(app.add.picker, 0, "backward reaches the first suggestion");
    }

    #[test]
    fn add_box_puts_the_terminal_caret_on_the_typed_field() {
        let mut app = App::new();
        app.add_mode = true;
        app.add = AddDraft::fresh();
        let (lines, cursor) = draw_with_cursor(&app);
        assert!(
            !lines.iter().any(|l| l.contains('▌')),
            "no inline typing-cursor glyph"
        );
        // Main pane x=0, y=2; the add box description text starts after
        // the left border plus the two-cell `+ ` marker.
        assert_eq!(
            cursor,
            Position::new(3, 4),
            "empty description caret starts after the + marker"
        );
        app.add.desc = "buy milk".into();
        app.add.desc_cursor = 8;
        let (_, cursor) = draw_with_cursor(&app);
        assert_eq!(
            cursor,
            Position::new(11, 4),
            "description caret follows the typed text"
        );
        // Tag filter line follows the description and chips; its text is
        // indented two cells. An empty filter shows the grey example, so the
        // caret rests at the example's start.
        app.add.desc.clear();
        app.add.focus = AddFocus::Tags;
        let (_, cursor) = draw_with_cursor(&app);
        assert_eq!(
            cursor,
            Position::new(3, 6),
            "empty tag example caret sits at its start"
        );
        app.add.filter = "urg".into();
        app.add.filter_cursor = 3;
        let (_, cursor) = draw_with_cursor(&app);
        assert_eq!(
            cursor,
            Position::new(6, 6),
            "tag filter caret follows the typed text"
        );
        // Fixed-list pickers have no text input, so no caret.
        app.add.focus = AddFocus::Depends;
        app.add.filter.clear();
        let (_, cursor) = draw_with_cursor(&app);
        assert_eq!(
            cursor,
            Position::new(119, 39),
            "depends picker leaves the terminal cursor hidden"
        );
    }

    #[test]
    fn add_box_caret_moves_and_edits_in_place() {
        use KeyCode as K;
        let mods = KeyModifiers::empty();
        let mut app = App::new();
        app.add_mode = true;
        app.add = AddDraft::fresh();
        for c in "abc".chars() {
            add_key(&mut app, K::Char(c), mods);
        }
        assert_eq!(app.add.desc, "abc");
        assert_eq!(app.add.desc_cursor, 3);
        add_key(&mut app, K::Left, mods);
        add_key(&mut app, K::Left, mods);
        assert_eq!(app.add.desc_cursor, 1, "Left moves one char");
        add_key(&mut app, K::Char('X'), mods);
        assert_eq!(app.add.desc, "aXbc", "typing inserts at the caret");
        assert_eq!(app.add.desc_cursor, 2);
        add_key(&mut app, K::Backspace, mods);
        assert_eq!(app.add.desc, "abc", "Backspace removes before the caret");
        assert_eq!(app.add.desc_cursor, 1);
        add_key(&mut app, K::Delete, mods);
        assert_eq!(app.add.desc, "ac", "Delete removes after the caret");
        assert_eq!(app.add.desc_cursor, 1);
        // The caret clamps at both ends.
        for _ in 0..5 {
            add_key(&mut app, K::Left, mods);
        }
        assert_eq!(app.add.desc_cursor, 0);
        for _ in 0..5 {
            add_key(&mut app, K::Right, mods);
        }
        assert_eq!(app.add.desc_cursor, 2);
        add_key(&mut app, K::Char('Z'), mods);
        assert_eq!(app.add.desc, "acZ");
        let (_, cursor) = draw_with_cursor(&app);
        assert_eq!(
            cursor,
            Position::new(6, 4),
            "terminal caret follows the moved text cursor"
        );

        // Trigger chars stay reserved mid-description and preserve the
        // description caret across the picker.
        add_key(&mut app, K::Left, mods);
        assert_eq!(app.add.desc_cursor, 2);
        add_key(&mut app, K::Char('#'), mods);
        assert_eq!(app.add.focus, AddFocus::Tags);
        assert_eq!(app.add.desc, "acZ", "trigger never types into the desc");
        add_key(&mut app, K::Esc, mods);
        assert_eq!(app.add.focus, AddFocus::Desc);
        assert_eq!(app.add.desc_cursor, 2, "desc caret survives the picker");
    }

    #[test]
    fn add_box_filter_caret_moves_and_edits_in_place() {
        use KeyCode as K;
        let mods = KeyModifiers::empty();
        let mut app = App::new();
        app.add_mode = true;
        app.add.focus = AddFocus::Tags;
        for c in "ab".chars() {
            add_key(&mut app, K::Char(c), mods);
        }
        assert_eq!(app.add.filter, "ab");
        assert_eq!(app.add.filter_cursor, 2);
        add_key(&mut app, K::Left, mods);
        add_key(&mut app, K::Char('X'), mods);
        assert_eq!(app.add.filter, "aXb", "typing inserts at the caret");
        assert_eq!(app.add.filter_cursor, 2);
        add_key(&mut app, K::Backspace, mods);
        assert_eq!(app.add.filter, "ab");
        add_key(&mut app, K::Delete, mods);
        assert_eq!(app.add.filter, "a", "Delete removes after the caret");
        let (_, cursor) = draw_with_cursor(&app);
        // Filter line starts one cell right for the box border; one char sits before the caret.
        assert_eq!(cursor, Position::new(4, 6));

        // Due mirrors caret edits into its chip value.
        app.add.focus = AddFocus::Due;
        app.add.clear_filter();
        for c in "today".chars() {
            add_key(&mut app, K::Char(c), mods);
        }
        assert_eq!(app.add.due, "today");
        add_key(&mut app, K::Left, mods);
        add_key(&mut app, K::Backspace, mods);
        assert_eq!(app.add.filter, "tody");
        assert_eq!(app.add.due, "tody", "due follows the edited filter");
    }

    #[test]
    fn section_prompt_caret_sits_on_the_footer() {
        let mut app = App::new();
        app.view = View::Project("work".into());
        app.open_section_prompt(SectionPromptKind::Add);
        let (_, cursor) = draw_with_cursor(&app);
        // Footer status row is y=39; ` ` + `  ADD SECTION  ` is 16 cells.
        assert_eq!(
            cursor,
            Position::new(16, 39),
            "caret follows the section prompt prefix"
        );
        if let Some(p) = app.section_prompt.as_mut() {
            p.buffer = "Now".into();
            p.cursor = 3;
        }
        let (_, cursor) = draw_with_cursor(&app);
        assert_eq!(
            cursor,
            Position::new(19, 39),
            "caret follows the typed section name"
        );
    }

    #[test]
    fn section_prompt_caret_moves_and_edits_in_place() {
        let mut app = App::new();
        app.view = View::Project("work".into());
        app.open_section_prompt(SectionPromptKind::Add);
        for c in "Now".chars() {
            route_key(&mut app, KeyCode::Char(c));
        }
        route_key(&mut app, KeyCode::Left);
        assert_eq!(app.section_prompt.as_ref().unwrap().cursor, 2);
        route_key(&mut app, KeyCode::Char('X'));
        assert_eq!(app.section_prompt.as_ref().unwrap().buffer, "NoXw");
        route_key(&mut app, KeyCode::Backspace);
        assert_eq!(app.section_prompt.as_ref().unwrap().buffer, "Now");
        assert_eq!(app.section_prompt.as_ref().unwrap().cursor, 2);
        route_key(&mut app, KeyCode::Delete);
        assert_eq!(app.section_prompt.as_ref().unwrap().buffer, "No");
        let (_, cursor) = draw_with_cursor(&app);
        // 16-cell prefix plus the two chars before the caret.
        assert_eq!(cursor, Position::new(18, 39));
    }

    #[test]
    fn add_box_more_enter_opens_chosen_picker() {
        use KeyCode as K;
        let mut app = App::new();
        app.add_mode = true;
        let cases = [
            ("@ project", AddFocus::Project),
            ("! priority", AddFocus::Priority),
            ("# tags", AddFocus::Tags),
            ("~ due", AddFocus::Due),
        ];
        for (label, expected) in cases {
            app.add.focus = AddFocus::More;
            app.add.filter.clear();
            app.add.picker = add_suggestions(&app, &app.add)
                .iter()
                .position(|s| s == label)
                .expect("more item listed");
            add_key(&mut app, K::Enter, KeyModifiers::empty());
            assert_eq!(app.add.focus, expected, "Enter opens {label} and stays");
        }
    }

    #[test]
    fn add_box_backspace_removes_only_on_empty_filter() {
        use KeyCode as K;
        let mods = KeyModifiers::empty();
        let mut app = App::new();
        app.add_mode = true;

        app.add.focus = AddFocus::Project;
        app.add.project = "work".into();
        add_key(&mut app, K::Backspace, mods);
        assert!(app.add.project.is_empty(), "project clears");
        assert_eq!(app.add.focus, AddFocus::Project, "picker stays focused");

        app.add.project = "work".into();
        app.add.filter = "wo".into();
        app.add.filter_cursor = 2;
        add_key(&mut app, K::Backspace, mods);
        assert_eq!(app.add.filter, "w", "non-empty filter edits");
        assert_eq!(app.add.project, "work", "project chip unchanged");

        app.add.focus = AddFocus::Tags;
        app.add.filter.clear();
        app.add.tags = vec!["first".into(), "second".into()];
        add_key(&mut app, K::Backspace, mods);
        assert_eq!(app.add.tags, vec!["first".to_string()], "last tag pops");

        app.add.focus = AddFocus::Due;
        app.add.due = "today".into();
        add_key(&mut app, K::Backspace, mods);
        assert!(app.add.due.is_empty(), "due clears");

        app.add.focus = AddFocus::Priority;
        app.add.prio_set = true;
        add_key(&mut app, K::Backspace, mods);
        assert!(!app.add.prio_set, "priority unsets");

        app.add.focus = AddFocus::More;
        add_key(&mut app, K::Backspace, mods);
        assert_eq!(app.add.focus, AddFocus::More, "More is a no-op");
    }

    #[test]
    fn add_box_footer_hints_removal_only_when_set() {
        let mut app = App::new();
        app.add_mode = true;
        app.add = AddDraft::fresh();
        let c = draw(&app);
        assert!(!c.contains("⌫ remove"), "no hint without a value");

        app.add.focus = AddFocus::Project;
        app.add.project = "work".into();
        let c = draw(&app);
        assert!(c.contains("⌫ remove"), "set project hints removal");

        app.add.project.clear();
        let c = draw(&app);
        assert!(!c.contains("⌫ remove"), "cleared project drops the hint");
    }

    #[test]
    fn section_registry_seeds_from_tasks_and_manual_additions() {
        let mut app = App::new();
        // Mock task values seed the registry; nothing is hardcoded.
        assert_eq!(
            app.sections,
            vec!["Now".to_string(), "Next".to_string(), "Later".to_string()]
        );

        // An empty store has an empty registry: no default sections.
        app.tasks.clear();
        app.sync_sections();
        assert!(app.sections.is_empty(), "empty data has no sections");

        // Custom names sort after Now/Next/Later, alphabetically.
        app.tasks = mock_tasks();
        app.tasks[0].section = "Reading".into();
        app.tasks[1].section = "Alpha".into();
        app.sync_sections();
        assert_eq!(
            app.sections,
            vec![
                "Now".to_string(),
                "Next".to_string(),
                "Later".to_string(),
                "Alpha".to_string(),
                "Reading".to_string()
            ],
            "Now-Next-Later then custom A–Z"
        );

        // A manual name joins even when no task carries it, without duplicates.
        app.manual_sections.push("Someday".into());
        app.manual_sections.push("someday".into());
        app.sync_sections();
        assert!(app.sections.contains(&"Someday".to_string()));
        assert_eq!(
            app.sections
                .iter()
                .filter(|s| s.eq_ignore_ascii_case("someday"))
                .count(),
            1,
            "case-insensitive duplicate is dropped"
        );
        assert_eq!(
            app.sections.last().map(String::as_str),
            Some("Someday"),
            "custom sections sort after the standard names"
        );
    }

    #[test]
    fn sections_render_unsectioned_and_empty_manual_groups() {
        let mut app = App::new();
        // Only project views render sections; Today/All stay flat.
        app.view = View::Project("work".into());
        // Unsectioned tasks render as a flat list with no header on top;
        // sections appear below once added.
        let unsectioned = mock_uuid(3);
        app.tasks
            .iter_mut()
            .find(|t| t.uuid == unsectioned)
            .unwrap()
            .section
            .clear();
        app.sync_sections();
        app.cursor_to_first_task();
        let lines = draw_lines(&app);
        assert!(
            !lines.iter().any(|l| main_col(l).contains("UNSECTIONED")),
            "no UNSECTIONED header"
        );
        let now_pos = lines
            .iter()
            .position(|l| main_col(l).contains("NOW"))
            .expect("NOW header renders");
        let flat_pos = lines
            .iter()
            .position(|l| main_col(l).contains("Reply to Nadia"))
            .expect("unsectioned task renders flat on top");
        assert!(flat_pos < now_pos, "flat unsectioned list leads sections");
        assert_eq!(
            main_col(&lines[now_pos - 1]),
            "",
            "blank line cushions the first header"
        );

        // A manual section with no tasks renders count 0 with the same cushion.
        app.open_section_prompt(SectionPromptKind::Add);
        for ch in "Someday".chars() {
            route_key(&mut app, KeyCode::Char(ch));
        }
        route_key(&mut app, KeyCode::Enter);
        let lines = draw_lines(&app);
        let someday_pos = lines
            .iter()
            .position(|l| main_col(l).contains("SOMEDAY"))
            .expect("empty manual section header renders");
        assert!(
            main_col(&lines[someday_pos]).contains(" 0 "),
            "empty section renders count 0"
        );
        assert_eq!(main_col(&lines[someday_pos - 1]), "", "blank above");
        assert_eq!(main_col(&lines[someday_pos + 1]), "", "blank below");

        // Groups breathe with blank space, never boxes or rules.
        let flat = draw(&app);
        for glyph in ["┌", "┐", "└", "┘", "├", "┤"] {
            assert!(!flat.contains(glyph), "no square row boxes");
        }
    }

    #[test]
    fn section_moves_cycle_and_unsectioned_lands_first() {
        let mut app = App::new();
        // Section moves live in project views; Today/All stay flat.
        app.view = View::Project("work".into());
        let id = mock_uuid(3);
        app.tasks
            .iter_mut()
            .find(|t| t.uuid == id)
            .unwrap()
            .section
            .clear();
        app.sync_sections();
        goto(&mut app, &id);

        // An unsectioned task moves to the first registry section.
        app.move_section_forward();
        assert_eq!(app.task(&id).unwrap().section, "Now");
        assert_eq!(app.toast, "moved to Now");
        assert_eq!(
            app.cursor_id().as_deref(),
            Some(id.as_str()),
            "cursor holds by uuid"
        );

        // Known sections advance through the registry and wrap around.
        for expected in ["Next", "Later", "Now"] {
            app.move_section_forward();
            assert_eq!(app.task(&id).unwrap().section, expected);
            assert_eq!(app.cursor_id().as_deref(), Some(id.as_str()));
        }

        // Inbox refuses the move and preserves the hidden value.
        app.view = View::Inbox;
        app.cursor = 0;
        let inbox_id = app.cursor_id().expect("inbox row");
        let before = app.task(&inbox_id).unwrap().section.clone();
        app.move_section_forward();
        assert_eq!(app.task(&inbox_id).unwrap().section, before);
        assert_eq!(app.toast, "sections live in project views");
    }

    #[test]
    fn section_moves_refused_in_flat_views() {
        let mut app = App::new();
        // Today and All are flat: section moves are refused like Inbox.
        for view in [View::Today, View::All, View::Search] {
            app.view = view;
            app.cursor_to_first_task();
            if app.cursor_id().is_none() {
                continue;
            }
            let id = app.cursor_id().expect("flat row");
            let before = app.task(&id).unwrap().section.clone();
            app.move_section_forward();
            assert_eq!(app.task(&id).unwrap().section, before);
            assert_eq!(app.toast, "sections live in project views");
        }
    }

    #[test]
    fn section_prompts_add_rename_remove_and_block_keys() {
        let mut app = App::new();
        // Prompts live in project views; every flat view refuses A/R/X.
        for view in [View::All, View::Search, View::Today] {
            app.view = view.clone();
            let sections = app.sections.clone();
            for kind in [
                SectionPromptKind::Add,
                SectionPromptKind::Rename,
                SectionPromptKind::Remove,
            ] {
                app.open_section_prompt(kind);
                assert!(
                    app.section_prompt.is_none(),
                    "{kind:?} is refused in {view:?}"
                );
                assert_eq!(app.sections, sections, "{kind:?} leaves the registry alone");
                assert!(app.toast.contains("project views"), "hint explains why");
            }
        }
        app.view = View::Project("work".into());
        let cursor_before = app.cursor;
        let rows_before = app.open_rows();

        // A opens an inline prompt that blocks list keys and quit.
        app.open_section_prompt(SectionPromptKind::Add);
        assert!(app.section_prompt.is_some());
        assert!(route_key(&mut app, KeyCode::Char('j')));
        assert!(route_key(&mut app, KeyCode::Char('q')));
        assert_eq!(app.cursor, cursor_before, "list keys blocked");
        assert_eq!(app.open_rows(), rows_before, "nothing moves behind prompt");
        let c = draw(&app);
        assert!(c.contains("ADD SECTION"), "prompt renders in the footer");
        assert!(c.contains("SECTION"), "prompt names the SECTION action");

        // Esc cancels without adding.
        route_key(&mut app, KeyCode::Esc);
        assert!(app.section_prompt.is_none());
        assert!(!app.sections.iter().any(|s| s == "Someday"));

        // Enter adds the typed name.
        let sections_before = app.sections.len();
        app.open_section_prompt(SectionPromptKind::Add);
        for ch in "Someday".chars() {
            route_key(&mut app, KeyCode::Char(ch));
        }
        route_key(&mut app, KeyCode::Enter);
        assert_eq!(app.toast, "section Someday added");
        assert_eq!(app.sections.len(), sections_before + 1);
        assert!(app.sections.iter().any(|s| s == "Someday"));

        // A duplicate is refused with a hint.
        app.open_section_prompt(SectionPromptKind::Add);
        for ch in "Someday".chars() {
            route_key(&mut app, KeyCode::Char(ch));
        }
        route_key(&mut app, KeyCode::Enter);
        assert_eq!(app.toast, "section Someday already exists");

        // R on an unsectioned task never opens the prompt.
        let unsectioned = mock_uuid(3);
        app.tasks
            .iter_mut()
            .find(|t| t.uuid == unsectioned)
            .unwrap()
            .section
            .clear();
        app.sync_sections();
        goto(&mut app, &unsectioned);
        app.open_section_prompt(SectionPromptKind::Rename);
        assert!(app.section_prompt.is_none());
        assert_eq!(app.toast, "task has no section to rename");

        // R renames the cursor task's section across its tasks.
        let later = app
            .tasks
            .iter()
            .find(|t| t.section == "Later" && t.project == "work" && is_open(t))
            .unwrap()
            .uuid
            .clone();
        let later_count = app.tasks.iter().filter(|t| t.section == "Later").count();
        goto(&mut app, &later);
        app.open_section_prompt(SectionPromptKind::Rename);
        assert!(app.section_prompt.is_some());
        for ch in "Reading".chars() {
            route_key(&mut app, KeyCode::Char(ch));
        }
        let c = draw(&app);
        assert!(c.contains("RENAME SECTION") && c.contains("LATER →"));
        route_key(&mut app, KeyCode::Enter);
        assert_eq!(app.toast, "renamed Later to Reading");
        assert_eq!(
            app.tasks.iter().filter(|t| t.section == "Reading").count(),
            later_count,
            "every task moved with the rename"
        );
        assert!(!app.sections.iter().any(|s| s == "Later"));
        assert!(app.sections.iter().any(|s| s == "Reading"));

        // Renaming to an existing section is refused and changes nothing.
        goto(&mut app, &later);
        app.open_section_prompt(SectionPromptKind::Rename);
        for ch in "Now".chars() {
            route_key(&mut app, KeyCode::Char(ch));
        }
        route_key(&mut app, KeyCode::Enter);
        assert_eq!(app.toast, "section Now already exists");
        assert_eq!(app.task(&later).unwrap().section, "Reading");

        // X refuses a non-empty section and removes an empty one.
        app.open_section_prompt(SectionPromptKind::Remove);
        for ch in "Reading".chars() {
            route_key(&mut app, KeyCode::Char(ch));
        }
        route_key(&mut app, KeyCode::Enter);
        assert_eq!(app.toast, "Reading is not empty");
        assert!(app.sections.iter().any(|s| s == "Reading"));
        app.open_section_prompt(SectionPromptKind::Remove);
        for ch in "Someday".chars() {
            route_key(&mut app, KeyCode::Char(ch));
        }
        route_key(&mut app, KeyCode::Enter);
        assert_eq!(app.toast, "removed empty section Someday");
        assert!(!app.sections.iter().any(|s| s == "Someday"));

        // An unknown name and an empty name both explain themselves.
        app.open_section_prompt(SectionPromptKind::Remove);
        for ch in "Ghost".chars() {
            route_key(&mut app, KeyCode::Char(ch));
        }
        route_key(&mut app, KeyCode::Enter);
        assert_eq!(app.toast, "no section named Ghost");
        app.open_section_prompt(SectionPromptKind::Add);
        route_key(&mut app, KeyCode::Enter);
        assert_eq!(app.toast, "section name required");
    }

    #[test]
    fn popup_is_read_only_until_edit_and_validates() {
        let mut app = App::new();
        let t = app.task(&mock_uuid(3)).unwrap().clone();
        app.popup = Some(Popup::open(&app, &t));
        let c = draw(&app);
        assert!(c.contains("VIEW"), "popup opens read-only");
        assert!(!c.contains("PRIMARY"), "no PRIMARY section");
        assert!(!c.contains("SECONDARY"), "no SECONDARY section");

        // Typing in VIEW must not dirty the draft.
        let mods = KeyModifiers::empty();
        popup_key(&mut app, KeyCode::Char('x'), mods);
        assert!(!app.popup.as_ref().unwrap().dirty);

        // e enters EDIT at once, Enter saves and returns to VIEW.
        popup_key(&mut app, KeyCode::Char('e'), mods);
        assert_eq!(app.popup.as_ref().unwrap().mode, PopupMode::Edit);
        popup_key(&mut app, KeyCode::Char('!'), mods);
        assert!(app.popup.as_ref().unwrap().dirty);
        app.save_popup();
        assert!(app.popup.is_some(), "save stays open for rapid tweaks");
        assert!(!app.popup.as_ref().unwrap().dirty, "save clears dirty");
        assert_eq!(
            app.popup.as_ref().unwrap().mode,
            PopupMode::View,
            "save returns to VIEW"
        );
        assert!(app.task(&mock_uuid(3)).unwrap().desc.ends_with('!'));

        // `i` also enters EDIT straight from a fresh VIEW popup.
        let t = app.task(&mock_uuid(3)).unwrap().clone();
        app.popup = Some(Popup::open(&app, &t));
        popup_key(&mut app, KeyCode::Char('i'), mods);
        assert_eq!(app.popup.as_ref().unwrap().mode, PopupMode::Edit);
        // esc in EDIT discards the whole draft back to VIEW.
        popup_key(&mut app, KeyCode::Char('!'), mods);
        popup_key(&mut app, KeyCode::Esc, mods);
        assert_eq!(
            app.popup.as_ref().unwrap().mode,
            PopupMode::View,
            "esc in EDIT discards to VIEW"
        );

        // j/k moves between fields in VIEW; Tab toggles views instead.
        let t = app.task(&mock_uuid(3)).unwrap().clone();
        app.popup = Some(Popup::open(&app, &t));
        {
            let p = app.popup.as_mut().unwrap();
            p.field = 0;
        }
        for expected in 1..6 {
            popup_key(&mut app, KeyCode::Char('j'), mods);
            assert_eq!(app.popup.as_ref().unwrap().field, expected);
        }
        popup_key(&mut app, KeyCode::Char('k'), mods);
        assert_eq!(app.popup.as_ref().unwrap().field, 4, "k moves back");
        popup_key(&mut app, KeyCode::Up, mods);
        assert_eq!(app.popup.as_ref().unwrap().field, 3, "Up moves back too");
        // Tab cycles views with the inspector open; the popup follows along.
        app.switch_to_view(View::Inbox);
        popup_key(&mut app, KeyCode::Tab, mods);
        assert_eq!(
            app.view,
            View::Today,
            "Tab toggles views over the inspector"
        );
        assert!(app.popup.is_some(), "popup stays open across view toggle");
        popup_key(&mut app, KeyCode::BackTab, mods);
        assert_eq!(app.view, View::Inbox, "Shift+Tab toggles back");
        // The open inspector re-seats on the new view's cursor task instead
        // of showing the previous view's row.
        app.switch_to_view(View::All);
        let first = app.open_rows()[0].clone();
        goto(&mut app, &first);
        open_cursor_row(&mut app);
        let shown = app.popup.as_ref().unwrap().task_id.clone();
        assert_eq!(shown, app.cursor_id().unwrap());
        popup_key(&mut app, KeyCode::Tab, mods);
        assert_ne!(app.view, View::All, "view toggled");
        assert!(app.popup.is_some(), "popup stays open across view toggle");
        assert_eq!(
            app.popup.as_ref().unwrap().task_id,
            app.cursor_id().unwrap(),
            "inspector follows the new view's cursor"
        );
        // An empty view has no cursor task, so the inspector closes.
        app.switch_to_view(View::Project("zzz-empty".into()));
        assert!(app.popup.is_none(), "empty view closes the inspector");
        // Reopen for the editing checks below.
        app.switch_to_view(View::All);
        let t = app.task(&mock_uuid(3)).unwrap().clone();
        app.popup = Some(Popup::open(&app, &t));
        // e enters EDIT; typing edits the focused field at once.
        popup_key(&mut app, KeyCode::Char('e'), mods);
        assert_eq!(app.popup.as_ref().unwrap().mode, PopupMode::Edit);
        {
            let p = app.popup.as_mut().unwrap();
            p.field = F_DESC;
            p.dirty = false;
        }
        let before = app.popup.as_ref().unwrap().field;
        popup_key(&mut app, KeyCode::Char('j'), mods);
        assert!(app.popup.as_ref().unwrap().dirty, "j types in EDIT");
        assert_eq!(
            app.popup.as_ref().unwrap().field,
            before,
            "j never moves in EDIT"
        );
        // Tab in EDIT moves the suggestion highlight, not the field.
        {
            let p = app.popup.as_mut().unwrap();
            p.field = F_PROJECT;
            p.project = String::new();
            p.suggest_idx = 0;
        }
        let first = {
            let p = app.popup.as_ref().unwrap();
            popup_suggestions(&app, p)
                .first()
                .cloned()
                .unwrap_or_default()
        };
        popup_key(&mut app, KeyCode::Tab, mods);
        assert_eq!(
            app.popup.as_ref().unwrap().field,
            F_PROJECT,
            "Tab never leaves the field in EDIT"
        );
        // Highlight moves when suggestions exist.
        if !first.is_empty() {
            assert_eq!(
                app.popup.as_ref().unwrap().suggest_idx,
                1.min(
                    popup_suggestions(&app, app.popup.as_ref().unwrap())
                        .len()
                        .saturating_sub(1)
                )
            );
        }
        popup_key(&mut app, KeyCode::BackTab, mods);
        assert_eq!(
            app.popup.as_ref().unwrap().field,
            F_PROJECT,
            "Shift+Tab stays in the field in EDIT"
        );
        // Left/right move the text cursor inside the focused field.
        {
            let p = app.popup.as_mut().unwrap();
            p.field = F_PROJECT;
            p.project = "work".into();
            p.edit_cursor = 0;
        }
        popup_key(&mut app, KeyCode::Right, mods);
        assert_eq!(app.popup.as_ref().unwrap().edit_cursor, 1);
        popup_key(&mut app, KeyCode::Left, mods);
        assert_eq!(app.popup.as_ref().unwrap().edit_cursor, 0);
        // The terminal caret follows `edit_cursor` on the field row.
        let (_, cursor) = draw_with_cursor(&app);
        assert_eq!(
            cursor,
            Position::new(78, 7),
            "caret sits at the start of the field"
        );
        popup_key(&mut app, KeyCode::Right, mods);
        popup_key(&mut app, KeyCode::Right, mods);
        let (_, cursor) = draw_with_cursor(&app);
        assert_eq!(
            cursor,
            Position::new(80, 7),
            "caret follows two chars into the field"
        );
        // Delete removes the char after the caret.
        popup_key(&mut app, KeyCode::Delete, mods);
        assert_eq!(
            app.popup.as_ref().unwrap().project,
            "wok",
            "Delete removes after the caret"
        );
        assert_eq!(app.popup.as_ref().unwrap().edit_cursor, 2);
        // Read-only block renders without dashes or read-only tags.
        let c = draw(&app);
        assert!(c.contains("urgency"), "urgency row renders");
        assert!(c.contains("start"), "start row renders");
        assert!(!c.contains("read-only"), "no read-only tags");
        assert!(!c.contains("start/end"), "start label drops /end");
        // The read-only block leads with the ID above urgency. The urgency
        // lookup starts below the id row: in split view the list title
        // shares its line with the inspector border and carries the same
        // bare `urgency` column header.
        let lines = draw_lines(&app);
        let id_pos = lines
            .iter()
            .position(|l| l.contains(" id ") && l.contains('3'))
            .expect("id row renders");
        let urgency_pos = lines
            .iter()
            .skip(id_pos + 1)
            .position(|l| l.contains("urgency"))
            .map(|p| p + id_pos + 1)
            .expect("urgency row renders");
        assert!(id_pos < urgency_pos, "id sits above urgency");
        // Without a short id the row falls back to the UUID prefix.
        let mut no_short = t.clone();
        no_short.short_id = None;
        no_short.uuid = "abcdef1234567890".into();
        assert_eq!(task_id_display(&no_short), "abcdef12");

        // Project suggestions list current projects for the focused field.
        {
            let p = app.popup.as_mut().unwrap();
            p.field = F_PROJECT;
            p.project = String::new();
            p.suggest_idx = 0;
        }
        let sugs = {
            let p = app.popup.as_ref().unwrap();
            popup_suggestions(&app, p)
        };
        assert!(sugs.iter().any(|s| s == "work"), "project hints list work");
        // Tab applies the highlighted suggestion (skip `no project` at 0).
        {
            let p = app.popup.as_mut().unwrap();
            p.suggest_idx = sugs.iter().position(|s| s == "work").unwrap_or(1);
        }
        app.apply_popup_suggestion();
        assert!(!app.popup.as_ref().unwrap().project.is_empty());
        // Tag suggestions list existing tags.
        {
            let p = app.popup.as_mut().unwrap();
            p.field = F_TAGS;
            p.tags = String::new();
            p.suggest_idx = 0;
        }
        let sugs = {
            let p = app.popup.as_ref().unwrap();
            popup_suggestions(&app, p)
        };
        assert!(!sugs.is_empty(), "tag hints list existing tags");

        // `[`/`]` cycle the pickers and mark the draft dirty.
        let p = app.popup.as_mut().unwrap();
        p.field = F_PRIO;
        p.prio = 'H';
        p.dirty = false;
        popup_key(&mut app, KeyCode::Char(']'), mods);
        assert_eq!(app.popup.as_ref().unwrap().prio, 'M');
        assert!(app.popup.as_ref().unwrap().dirty);
        let p = app.popup.as_mut().unwrap();
        p.field = F_PROJECT;
        p.project = "personal".into();
        p.dirty = false;
        popup_key(&mut app, KeyCode::Char(']'), mods);
        assert_eq!(app.popup.as_ref().unwrap().project, "reading");

        // Empty description is rejected.
        let t = app.task(&mock_uuid(3)).unwrap().clone();
        let mut p = Popup::open(&app, &t);
        p.mode = PopupMode::Edit;
        p.desc = "   ".into();
        app.popup = Some(p);
        app.save_popup();
        assert!(app.popup.is_some(), "empty description rejected");
        assert_eq!(app.toast, "description required");

        // `esc` in EDIT discards only the focused field back to VIEW.
        let t = app.task(&mock_uuid(3)).unwrap().clone();
        let orig = t.desc.clone();
        app.popup = Some(Popup::open(&app, &t));
        popup_key(&mut app, KeyCode::Char('e'), mods);
        popup_key(&mut app, KeyCode::Char('!'), mods);
        assert!(app.popup.as_ref().unwrap().dirty);
        popup_key(&mut app, KeyCode::Esc, mods);
        let p = app.popup.as_ref().unwrap();
        assert_eq!(p.mode, PopupMode::View, "esc in EDIT discards the field");
        assert_eq!(p.desc, orig, "focused field restores its value");
        assert!(!p.dirty);

        // Per-field discard: an edit in another field survives `esc`.
        let t = app.task(&mock_uuid(3)).unwrap().clone();
        app.popup = Some(Popup::open(&app, &t));
        popup_key(&mut app, KeyCode::Char('e'), mods);
        {
            let p = app.popup.as_mut().unwrap();
            p.field = F_PROJECT;
            p.project = "personal".into();
            p.dirty = true;
        }
        {
            let p = app.popup.as_mut().unwrap();
            p.field = F_DESC;
        }
        popup_key(&mut app, KeyCode::Char('!'), mods);
        popup_key(&mut app, KeyCode::Esc, mods);
        let p = app.popup.as_ref().unwrap();
        assert_eq!(p.desc, t.desc, "the focused field reverts");
        assert_eq!(p.project, "personal", "other unsaved edits survive");
        assert!(p.dirty, "draft stays dirty from the surviving edit");
        assert_eq!(p.mode, PopupMode::View);

        // Plain close from VIEW leaves the list unchanged; Enter and Esc
        // both toggle the inspector closed around the same row.
        popup_key(&mut app, KeyCode::Char('e'), mods);
        popup_key(&mut app, KeyCode::Esc, mods);
        assert_eq!(
            app.popup.as_ref().unwrap().mode,
            PopupMode::View,
            "esc discards EDIT to VIEW"
        );
        popup_key(&mut app, KeyCode::Esc, mods);
        assert!(app.popup.is_none(), "esc closes from VIEW");
        let t = app.task(&mock_uuid(3)).unwrap().clone();
        app.popup = Some(Popup::open(&app, &t));
        popup_key(&mut app, KeyCode::Enter, mods);
        assert!(app.popup.is_none(), "Enter closes from VIEW");

        // `n` no longer creates subtasks; it does nothing in popup VIEW.
        let t = app.task(&mock_uuid(3)).unwrap().clone();
        app.popup = Some(Popup::open(&app, &t));
        let before = app.tasks.len();
        popup_key(&mut app, KeyCode::Char('n'), mods);
        assert_eq!(app.tasks.len(), before, "no subtask on n");

        // `E` toasts the system editor without closing.
        popup_key(&mut app, KeyCode::Char('E'), mods);
        assert_eq!(app.toast, "opened $EDITOR");
        assert!(app.popup.is_some(), "editor keeps popup open");
    }

    #[test]
    fn edit_escape_without_edits_stays_clean() {
        let mods = KeyModifiers::empty();
        let mut app = App::new();
        let t = app.task(&mock_uuid(3)).unwrap().clone();
        app.popup = Some(Popup::open(&app, &t));
        app.clear_toast();
        assert!(!app.popup.as_ref().unwrap().dirty);
        popup_key(&mut app, KeyCode::Char('e'), mods);
        assert_eq!(app.popup.as_ref().unwrap().mode, PopupMode::Edit);
        assert!(
            !app.popup.as_ref().unwrap().dirty,
            "entering EDIT dirties nothing"
        );
        popup_key(&mut app, KeyCode::Esc, mods);
        let p = app.popup.as_ref().unwrap();
        assert_eq!(p.mode, PopupMode::View, "esc returns to VIEW");
        assert!(!p.dirty, "e then esc without edits leaves no unsaved");
        assert!(app.toast.is_empty(), "no discard toast without edits");
    }

    #[test]
    fn save_then_edit_escape_stays_clean() {
        // A save that trims (e.g. a trailing space) must not haunt the next
        // edit with a stale `● unsaved`.
        let mods = KeyModifiers::empty();
        let mut app = App::new();
        let t = app.task(&mock_uuid(3)).unwrap().clone();
        app.popup = Some(Popup::open(&app, &t));
        popup_key(&mut app, KeyCode::Char('e'), mods);
        popup_key(&mut app, KeyCode::Char(' '), mods);
        app.save_popup();
        assert_eq!(app.toast, "saved");
        assert!(!app.popup.as_ref().unwrap().dirty, "save clears dirty");
        popup_key(&mut app, KeyCode::Char('e'), mods);
        popup_key(&mut app, KeyCode::Esc, mods);
        assert!(
            !app.popup.as_ref().unwrap().dirty,
            "no stale unsaved after save"
        );
    }

    #[test]
    fn list_hotkeys_work_behind_inspector() {
        use KeyCode as K;
        let mods = KeyModifiers::empty();
        let mut app = App::new();
        app.view = View::All;
        let first = app.open_rows()[0].clone();
        goto(&mut app, &first);
        open_cursor_row(&mut app);
        assert!(app.popup.is_some(), "Enter opens the inspector");
        popup_key(&mut app, K::Char('h'), mods);
        assert!(app.popup_list_focus, "h focuses the task list");
        // `s` toggles start without closing the inspector.
        let id = app.cursor_id().expect("cursor on a task");
        let was_active = app.task(&id).unwrap().active;
        list_focus_key(&mut app, K::Char('s'), mods);
        assert!(app.popup.is_some(), "s keeps the inspector open");
        assert_eq!(
            app.task(&id).unwrap().active,
            !was_active,
            "s works behind the inspector"
        );
        assert!(app.popup_list_focus, "s keeps list focus");
        // `d` toggles done without closing the inspector.
        list_focus_key(&mut app, K::Char('d'), mods);
        assert!(app.popup.is_some(), "d keeps the inspector open");
        assert_eq!(
            app.task(&id).unwrap().status,
            "completed",
            "d works behind the inspector"
        );
        // `u` undoes without closing the inspector.
        list_focus_key(&mut app, K::Char('u'), mods);
        assert!(app.popup.is_some(), "u keeps the inspector open");
        assert_eq!(
            app.task(&id).unwrap().status,
            "active",
            "u works behind the inspector"
        );
    }

    #[test]
    fn inspector_h_focuses_list_and_preview_follows_cursor() {
        use KeyCode as K;
        let mods = KeyModifiers::empty();
        let mut app = App::new();
        app.view = View::All;
        let first = app.open_rows()[0].clone();
        let second = app.open_rows()[1].clone();
        goto(&mut app, &first);
        open_cursor_row(&mut app);
        assert!(app.popup.is_some(), "Enter opens the inspector");
        assert!(!app.popup_list_focus, "inspector owns focus on open");
        assert_eq!(app.popup.as_ref().unwrap().task_id, first);
        // `h` hands focus to the task list without moving the cursor.
        popup_key(&mut app, K::Char('h'), mods);
        assert!(app.popup_list_focus, "h focuses the task list");
        assert!(app.popup.is_some(), "h leaves the inspector open");
        assert_eq!(app.cursor_id(), Some(first.clone()));
        // Browsing keeps list focus and the preview follows the cursor.
        list_focus_key(&mut app, K::Char('j'), mods);
        assert_eq!(app.cursor_id(), Some(second.clone()));
        assert!(app.popup_list_focus, "browsing keeps list focus");
        assert_eq!(
            app.popup.as_ref().unwrap().task_id,
            second,
            "preview follows the cursor"
        );
        // `i` hands focus back to the inspector on the same task.
        list_focus_key(&mut app, K::Char('i'), mods);
        assert!(!app.popup_list_focus, "i refocuses the inspector");
        assert_eq!(app.popup.as_ref().unwrap().task_id, second);
        // `esc` closes from list focus too.
        app.popup_list_focus = true;
        list_focus_key(&mut app, K::Esc, mods);
        assert!(app.popup.is_none(), "esc closes from list focus");
    }

    #[test]
    fn inspector_border_brightens_with_focus() {
        let mut app = App::new();
        let t = app.task(&mock_uuid(3)).unwrap().clone();
        app.popup = Some(Popup::open(&app, &t));
        let border_fg = |app: &App| -> ratatui::style::Color {
            let backend = TestBackend::new(120, 40);
            let mut term = Terminal::new(backend).unwrap();
            term.draw(|f| render(f, app)).unwrap();
            let buf = term.backend().buffer();
            // The inspector sits right of the list: its top-left corner is
            // the last rounded corner on the row below the top bar.
            buf.content()
                .chunks(120)
                .nth(2)
                .unwrap()
                .iter()
                .rev()
                .find(|c| c.symbol() == "╭")
                .unwrap()
                .fg
        };
        app.popup_list_focus = false;
        assert_eq!(
            border_fg(&app),
            INK2,
            "focused inspector uses the lighter border"
        );
        app.popup_list_focus = true;
        assert_eq!(
            border_fg(&app),
            LINE,
            "list focus falls back to the hairline border"
        );
    }

    #[test]
    fn active_shows_start_indicator_only_and_blocked_stays_flat() {
        let mut app = App::new();
        app.view = View::All;
        let rows = app.open_rows();
        assert!(
            rows.iter().filter_map(|id| app.task(id)).any(|t| t.active),
            "fixture has active tasks"
        );
        assert!(
            rows.iter()
                .filter_map(|id| app.task(id))
                .any(|t| !t.depends.is_empty()),
            "fixture has dependency links"
        );
        assert!(
            app.is_blocked(&mock_uuid(3)),
            "blocked task waits on open deps"
        );

        // Active is exactly one signal — the mint ◉ — with no tint or text.
        // The cursor sits on the first row, so the active row is off-cursor.
        let c = draw(&app);
        assert!(!c.contains("● active"), "no active text renders");
        assert!(c.contains("◉"), "active start indicator renders");
        let lines = draw_lines(&app);
        let active_y = lines
            .iter()
            .position(|l| l.contains("Draft Q4 planning doc"))
            .unwrap();
        let backend = TestBackend::new(120, 40);
        let mut term = Terminal::new(backend).unwrap();
        term.draw(|f| render(f, &app)).unwrap();
        let buf = term.backend().buffer();
        let untinted = (30..60).all(|x| buf[(x as u16, active_y as u16)].bg != TINT);
        assert!(untinted, "off-cursor active row keeps no tint");
        let mint_box = (0..120).any(|x| {
            let cell = &buf[(x as u16, active_y as u16)];
            cell.symbol() == "◉" && cell.fg == MINT
        });
        assert!(mint_box, "active start indicator renders mint");

        // Cursor on the active row keeps › plus tint alongside the mint ◉.
        let active_id = rows
            .iter()
            .find(|id| app.task(id).map(|t| t.active).unwrap_or(false))
            .expect("active task id")
            .clone();
        goto(&mut app, &active_id);
        let lines = draw_lines(&app);
        let cursor_y = lines
            .iter()
            .position(|l| l.contains('▎') && l.contains("◉"))
            .expect("cursor rests on the active row");
        let backend = TestBackend::new(120, 40);
        let mut term = Terminal::new(backend).unwrap();
        term.draw(|f| render(f, &app)).unwrap();
        let buf = term.backend().buffer();
        let cursor_tinted = (30..60).all(|x| buf[(x as u16, cursor_y as u16)].bg == TINT);
        assert!(cursor_tinted, "cursor keeps its highlight on active rows");

        // Completed work rests dimmed at the bottom, below open rows.
        assert!(c.contains("COMPLETED"), "completed tail renders");
        let done = app.completed_rows();
        assert!(!done.is_empty(), "fixture has completed work");
        for id in &done {
            assert!(!rows.contains(id), "completed never in open rows");
        }
        let completed_y = lines
            .iter()
            .position(|l| l.contains("Renew passport"))
            .unwrap();
        let last_open_y = lines
            .iter()
            .rposition(|l| {
                rows.iter()
                    .filter_map(|id| app.task(id))
                    .any(|t| l.contains(&t.desc[..4.min(t.desc.len())]))
            })
            .unwrap();
        assert!(completed_y > last_open_y, "completed sits below open rows");
        let dimmed = (0..100).any(|x| {
            let cell = &buf[(x as u16, completed_y as u16)];
            cell.fg == DIM && cell.modifier.contains(Modifier::CROSSED_OUT)
        });
        assert!(dimmed, "completed tail is dimmed and struck through");

        // Blocking lifts when the last open dep completes, with no counts.
        let c = draw(&app);
        assert!(!c.contains("1/2"), "no progress fraction");
        app.complete_ids(&[mock_uuid(12)]);
        assert!(
            !app.is_blocked(&mock_uuid(3)),
            "task unblocks when deps done"
        );

        // Deleting a blocked task is single-only; the blocker stays open.
        let mut app = App::new();
        app.view = View::All;
        let rows = app.open_rows();
        assert!(rows.contains(&mock_uuid(3)) && rows.contains(&mock_uuid(12)));
        app.confirm_delete = Some(vec![mock_uuid(3)]);
        app.confirm_delete_yes();
        assert_eq!(app.task(&mock_uuid(3)).unwrap().status, "deleted");
        assert_eq!(app.task(&mock_uuid(12)).unwrap().status, "pending");
        assert!(!app.open_rows().contains(&mock_uuid(3)));
        assert!(app.open_rows().contains(&mock_uuid(12)));
    }

    #[test]
    fn theme_palette_space_and_glyphs() {
        let mut app = App::new();
        app.view = View::All;
        let list_col = |l: &String| -> String { l.trim().to_string() };
        // Text glyphs carry state without color; row chrome dropped the
        // checkbox, pending circle, and project dot.
        let c = draw(&app);
        let lines = draw_lines(&app);
        for line in &lines {
            let row = list_col(line);
            for glyph in ["[ ]", "[×]", "○"] {
                assert!(!row.contains(glyph), "row glyph {glyph} is gone");
            }
        }
        for glyph in ["●", "▎", "◉", "✓"] {
            assert!(c.contains(glyph), "glyph {glyph} renders");
        }
        let selected_id = app.open_rows()[0].clone();
        let selected_desc = app.task(&selected_id).unwrap().desc.clone();
        app.selected.insert(selected_id);
        let other = app.open_rows()[1].clone();
        goto(&mut app, &other);
        let lines = draw_lines(&app);
        let selected_row = lines
            .iter()
            .find(|l| list_col(l).contains(&selected_desc))
            .expect("selected row renders");
        assert!(list_col(selected_row).contains('▌'), "selection marker");
        assert!(!list_col(selected_row).contains("[×]"), "no selection box");
        // Blank space groups: gutter is empty, no boxes or borders.
        let backend = TestBackend::new(120, 40);
        let mut term = Terminal::new(backend).unwrap();
        term.draw(|f| render(f, &app)).unwrap();
        let buf = term.backend().buffer();
        let gutter_blank =
            (10..30).all(|y| (118..120).all(|x| buf[(x as u16, y as u16)].symbol() == " "));
        assert!(gutter_blank, "list keeps its right margin");
        for glyph in ["┌", "┐", "└", "┘", "├", "┤"] {
            assert!(!c.contains(glyph), "no square row boxes");
        }
        // Priority dot colors carry priority at a glance.
        let prio_y = lines
            .iter()
            .position(|l| l.contains("Reply to Nadia"))
            .unwrap();
        let p1 = (0..12).any(|x| {
            let cell = &buf[(x as u16, prio_y as u16)];
            cell.symbol() == "●" && cell.fg == RED
        });
        assert!(p1, "H priority dot renders red");
        // Bright cursor row versus dim chrome.
        let cursor_y = lines.iter().position(|l| l.contains('▎')).unwrap();
        let bright_cursor = (0..100).any(|x| {
            let cell = &buf[(x as u16, cursor_y as u16)];
            cell.fg == BRIGHT && cell.modifier.contains(Modifier::BOLD)
        });
        assert!(bright_cursor, "cursor pops bright and bold");
        let hint_app = App::new();
        let hint_lines = draw_lines(&hint_app);
        let hint_y = hint_lines
            .iter()
            .rposition(|l| l.contains("j/k move"))
            .unwrap();
        let mut hint_term = Terminal::new(TestBackend::new(120, 40)).unwrap();
        hint_term.draw(|f| render(f, &hint_app)).unwrap();
        let hint_buf = hint_term.backend().buffer();
        let hint_dim = (0..60).any(|x| hint_buf[(x as u16, hint_y as u16)].fg == DIM);
        assert!(hint_dim, "hints stay faint");
        // Blocked red and active mint keep meaning without color names.
        assert!(c.contains("blocked"), "blocked marker renders");
        let parent_y = lines
            .iter()
            .position(|l| l.contains("Reply to Nadia"))
            .unwrap();
        let blocked_red = (0..120).any(|x| {
            let cell = &buf[(x as u16, (parent_y + 1) as u16)];
            cell.fg == RED
        });
        assert!(blocked_red, "blocked uses red");
    }

    #[test]
    fn completed_tail_capped_and_view_filtered() {
        let mut app = App::new();
        // Seed 12 completed tops with distinct `end` timestamps.
        for i in 0..12 {
            let short = app.next_id;
            app.next_id += 1;
            app.tasks.push(Task {
                uuid: mock_uuid(short),
                short_id: Some(short),
                desc: format!("done {i}"),
                project: if i % 2 == 0 {
                    "work".into()
                } else {
                    "personal".into()
                },
                due: "—".into(),
                due_raw: None,
                prio: 'L',
                tags: vec![],
                section: "Now".into(),
                urgency: 0.0,
                active: false,
                status: "completed".into(),
                depends: Vec::new(),
                recur: "none".into(),
                recur_parent: String::new(),
                start_end: "done".into(),
                start_raw: None,
                end: Some(format!("202611{day:02}T120000Z", day = i + 1)),
                annotations: Vec::new(),
            });
        }
        // Tail is capped at 10 by `end` desc, even with 15 total completed.
        app.view = View::All;
        let tail = app.completed_rows();
        assert_eq!(tail.len(), 10, "completed tail capped at 10");
        // Most recent (`end` max) surfaces first.
        let newest = tail[0].clone();
        assert_eq!(app.task(&newest).unwrap().desc, "done 11");
        // View filter still applies inside the cap.
        app.view = View::Project("work".into());
        let work_tail = app.completed_rows();
        assert!(!work_tail.is_empty(), "project tail keeps matches");
        for id in &work_tail {
            assert_eq!(app.task(id).unwrap().project, "work");
        }
        assert!(work_tail.len() <= 10, "project tail respects cap");
        // Deleted never renders in any view.
        if let Some(t) = app.tasks.iter_mut().find(|t| t.uuid == newest) {
            t.status = "deleted".into();
        }
        app.view = View::All;
        assert!(!app.completed_rows().contains(&newest));
        assert!(!app.open_rows().contains(&newest));
    }

    #[test]
    fn due_labels_derive_from_calendar_day() {
        use chrono::TimeZone;
        let now = Local.with_ymd_and_hms(2026, 11, 12, 12, 0, 0).unwrap();
        // Absent due stays neutral.
        assert_eq!(format_due_label(None, now), "—");
        // Overdue counts calendar days.
        assert_eq!(
            format_due_label(Some("20261110T120000Z"), now),
            "2d overdue"
        );
        // Due-today keeps local time.
        let today = format_due_label(Some("20261112T090000Z"), now);
        assert!(today.starts_with("today"), "due-today label {today}");
        // Tomorrow and weekday stay date-only.
        assert_eq!(format_due_label(Some("20261113T120000Z"), now), "tomorrow");
        let weekday = format_due_label(Some("20261114T120000Z"), now);
        assert_eq!(weekday.len(), 3, "weekday label {weekday}");
        // Beyond a week renders as a date.
        assert_eq!(
            format_due_label(Some("20261201T120000Z"), now),
            "2026-12-01"
        );
    }

    #[test]
    fn due_mapping_handles_kuching_midnight_edge() {
        // 16:00Z is midnight in Kuching (+8) but still the prior day in UTC.
        let kuching = export_date_in_offset("20260711T160000Z", 8 * 3600).unwrap();
        let utc = export_date_in_offset("20260711T160000Z", 0).unwrap();
        assert_eq!(kuching, NaiveDate::from_ymd_opt(2026, 7, 12).unwrap());
        assert_eq!(utc, NaiveDate::from_ymd_opt(2026, 7, 11).unwrap());
    }

    #[test]
    fn export_maps_priority_section_depends_and_readonly_fields() {
        use chrono::TimeZone;
        let now = Local.with_ymd_and_hms(2026, 11, 12, 12, 0, 0).unwrap();
        let json = r#"[
            {"uuid":"parent-1","id":7,"description":"parent","status":"pending","urgency":8.5,"depends":["child-1"],"siap.section":"Next"},
            {"uuid":"child-1","id":8,"description":"child","status":"pending","urgency":3.0},
            {"uuid":"gone-1","id":9,"description":"gone","status":"deleted","urgency":0.0},
            {"uuid":"done-1","id":0,"description":"done","status":"completed","end":"20261111T120000Z","urgency":1.0,"annotations":[{"entry":"20261111T120000Z","description":"shipped"}],"recur":"daily","parent":"template-1"}
        ]"#;
        let tasks = tasks_from_export(json, now);
        // Deleted never survives mapping.
        assert!(!tasks.iter().any(|t| t.uuid == "gone-1"));
        // Absent priority renders L; urgency stays read-only from backend.
        let child = tasks.iter().find(|t| t.uuid == "child-1").unwrap();
        assert_eq!(child.prio, 'L');
        assert_eq!(prio_label(child.prio), "L");
        // A missing section maps to unsectioned, never a written default.
        assert_eq!(child.section, "");
        assert_eq!(
            tasks.iter().find(|t| t.uuid == "parent-1").unwrap().section,
            "Next"
        );
        // Native `depends` stays flat: the blocker lists its dep.
        let blocked = tasks.iter().find(|t| t.uuid == "parent-1").unwrap();
        assert_eq!(blocked.depends, vec!["child-1".to_string()]);
        assert!(tasks
            .iter()
            .find(|t| t.uuid == "child-1")
            .unwrap()
            .depends
            .is_empty());
        // Deferred fields stay visible for read-only rendering.
        let done = tasks.iter().find(|t| t.uuid == "done-1").unwrap();
        assert_eq!(done.recur, "daily");
        assert_eq!(
            done.recur_parent, "template-1",
            "occurrence keeps template id"
        );
        assert!(annotations_text(&done.annotations).contains("shipped"));
        assert_eq!(
            done.annotations[0].entry, "20261111T120000Z",
            "entry timestamp kept"
        );
        assert_eq!(done.short_id, None, "completed id 0 has no short id");
    }

    #[test]
    fn popup_shows_deferred_fields_read_only() {
        use chrono::TimeZone;
        let now = Local.with_ymd_and_hms(2026, 11, 12, 12, 0, 0).unwrap();
        let json = r#"[
            {"uuid":"deferred-1","id":21,"description":"deferred work","status":"pending","urgency":4.0,
             "recur":"daily","annotations":[{"entry":"20261112T120000Z","description":"first note"},{"entry":"20261112T130000Z","description":"second note"}]}
        ]"#;
        let tasks = tasks_from_export(json, now);
        let mut app = App::new();
        app.tasks = tasks;
        app.view = View::All;
        let t = app.task("deferred-1").unwrap().clone();
        app.popup = Some(Popup::open(&app, &t));
        // `recur` is the seventh editable field; `annotation` stays
        // interactive in VIEW with its own keys.
        let c = draw(&app);
        assert!(c.contains("daily"), "recur value shows");
        assert!(c.contains("first note"), "annotations show");
        assert_eq!(Popup::field_count(), 7, "recur is editable");
        // Editing another field and saving preserves recurrence.
        app.popup.as_mut().unwrap().mode = PopupMode::Edit;
        app.popup.as_mut().unwrap().desc = "deferred work!".into();
        app.popup.as_mut().unwrap().due = "2026-11-13".into();
        app.popup.as_mut().unwrap().dirty = true;
        app.save_popup();
        let saved = app.task("deferred-1").unwrap();
        assert_eq!(saved.desc, "deferred work!");
        assert_eq!(saved.recur, "daily", "save preserves recur");
        assert!(
            annotations_text(&saved.annotations).contains("second note"),
            "save preserves notes"
        );
    }

    #[test]
    fn popup_annotation_add_select_delete() {
        use chrono::TimeZone;
        let now = Local.with_ymd_and_hms(2026, 11, 12, 12, 0, 0).unwrap();
        let json = r#"[
            {"uuid":"note-1","id":31,"description":"noted work","status":"pending","urgency":1.0,
             "annotations":[{"entry":"20261112T120000Z","description":"first"}]}
        ]"#;
        let tasks = tasks_from_export(json, now);
        let mut app = App::new();
        app.tasks = tasks;
        app.view = View::All;
        let t = app.task("note-1").unwrap().clone();
        app.popup = Some(Popup::open(&app, &t));

        // Empty drafts refuse with a toast and stay open.
        app.popup.as_mut().unwrap().annotating = true;
        app.annotate_current();
        assert_eq!(app.toast, "annotation required");

        // `a` draft saves a new note and selects the newest entry.
        app.popup.as_mut().unwrap().annotation_draft = "second".into();
        app.popup.as_mut().unwrap().annotation_cursor = 6;
        app.annotate_current();
        assert_eq!(app.toast, "annotated");
        let saved = app.task("note-1").unwrap();
        assert_eq!(saved.annotations.len(), 2);
        assert_eq!(saved.annotations[1].text, "second");
        assert!(!saved.annotations[1].entry.is_empty(), "entry stamped");

        // `[`/`]` selects and `D` removes the selected note.
        app.cycle_annotation(-1);
        assert_eq!(app.popup.as_ref().unwrap().annotation_idx, 0);
        app.cycle_annotation(1);
        assert_eq!(app.popup.as_ref().unwrap().annotation_idx, 1);
        app.denotate_selected();
        assert_eq!(app.toast, "annotation removed");
        let saved = app.task("note-1").unwrap();
        assert_eq!(saved.annotations.len(), 1);
        assert_eq!(saved.annotations[0].text, "first");
        let c = draw(&app);
        assert!(c.contains("first"), "remaining note renders");
    }

    #[test]
    fn sections_collapse_and_cursor_reaches_completed() {
        let mut app = App::new();
        // Headers live in project views; the COMPLETED tail stays
        // collapsible everywhere.
        app.view = View::Project("personal".into());
        app.cursor_to_first_task();
        // Headers are cursor stops; Enter toggles collapse.
        let header_pos = app
            .navigable_rows()
            .iter()
            .position(|r| matches!(r, NavRow::SectionHeader(_)))
            .expect("section header navigable");
        app.cursor = header_pos;
        assert!(app.cursor_id().is_none(), "header has no task");
        let label = match app.cursor_nav() {
            Some(NavRow::SectionHeader(l)) => l,
            _ => panic!("on a section header"),
        };
        app.toggle_collapsed(&label);
        assert!(app.is_collapsed(&label), "header collapses");
        let hidden: Vec<String> = app.open_rows();
        for id in &hidden {
            let t = app.task(id).unwrap();
            if t.section.eq_ignore_ascii_case(&label) {
                assert!(
                    !app.navigable_rows()
                        .iter()
                        .any(|r| r == &NavRow::Task(id.clone())),
                    "collapsed section hides its rows"
                );
                break;
            }
        }
        app.toggle_collapsed(&label);
        assert!(!app.is_collapsed(&label), "header expands again");

        // COMPLETED header collapses and the cursor reaches completed rows.
        let done_before = app.completed_rows().len();
        assert!(!app.completed_rows().is_empty(), "fixture has completed");
        app.toggle_collapsed("COMPLETED");
        assert!(
            !app.navigable_rows()
                .iter()
                .any(|r| matches!(r, NavRow::CompletedTask(_))),
            "collapsed COMPLETED hides its rows"
        );
        app.toggle_collapsed("COMPLETED");
        let completed_id = app.completed_rows()[0].clone();
        goto(&mut app, &completed_id);
        assert_eq!(app.cursor_id().as_deref(), Some(completed_id.as_str()));
        assert_eq!(done_before, app.completed_rows().len());
    }

    #[test]
    fn done_key_toggles_done_and_undone() {
        let mut app = App::new();
        app.view = View::All;
        app.cursor_to_first_task();
        let id = app.cursor_id().expect("cursor on a task");
        app.toggle_ids(std::slice::from_ref(&id));
        assert_eq!(app.task(&id).unwrap().status, "completed");
        goto(&mut app, &id);
        app.toggle_ids(std::slice::from_ref(&id));
        assert_eq!(app.task(&id).unwrap().status, "pending");
        assert!(app.undo_snapshot.is_some(), "toggle snapshots for undo");
    }

    #[test]
    fn undo_restores_last_change() {
        let mut app = App::new();
        app.view = View::All;
        app.cursor_to_first_task();
        let id = app.cursor_id().expect("cursor on a task");
        let before = app.task(&id).unwrap().status.clone();
        app.toggle_ids(std::slice::from_ref(&id));
        assert_ne!(app.task(&id).unwrap().status, before);
        app.undo();
        assert_eq!(app.task(&id).unwrap().status, before, "u restores status");
        app.undo();
        assert_eq!(app.toast, "nothing to undo", "single-level undo");
    }

    #[test]
    fn start_label_shows_relative_age() {
        use chrono::TimeZone;
        let now = Utc
            .with_ymd_and_hms(2026, 11, 12, 12, 0, 0)
            .unwrap()
            .with_timezone(&Local);
        assert_eq!(format_start_ago("", now), "");
        assert_eq!(format_start_ago("20261112T115900Z", now), "1 min ago");
        assert_eq!(format_start_ago("20261112T114500Z", now), "15 min ago");
        assert_eq!(format_start_ago("20261112T110000Z", now), "1 hour ago");
        assert_eq!(format_start_ago("20261112T090000Z", now), "3 hours ago");
        assert_eq!(format_start_ago("20261111T120000Z", now), "1 day ago");
        let t = Task {
            uuid: mock_uuid(901),
            short_id: Some(901),
            desc: "started".into(),
            project: "".into(),
            due: "".into(),
            due_raw: None,
            prio: 'M',
            tags: vec![],
            section: String::new(),
            urgency: 1.0,
            active: true,
            status: "active".into(),
            depends: Vec::new(),
            recur: "none".into(),
            recur_parent: String::new(),
            start_end: "not started".into(),
            start_raw: Some("20261112T110000Z".into()),
            end: None,
            annotations: Vec::new(),
        };
        assert_eq!(start_display(&t, now), "1 hour ago");
    }

    #[test]
    fn add_box_creates_custom_tags() {
        let mut app = App::new();
        app.add_mode = true;
        app.add.focus = crate::AddFocus::Tags;
        app.add.filter = "brand-new-tag".into();
        app.apply_add_suggestion();
        assert!(app.add.tags.contains(&"brand-new-tag".to_string()));
        assert!(app.add.filter.is_empty(), "filter clears after create");
        assert_eq!(app.add.focus, crate::AddFocus::Tags, "tags stay open");
    }

    #[test]
    fn add_box_applies_custom_project_and_due() {
        let mut app = App::new();
        app.add_mode = true;
        // A typed project is used as-is, creating it on commit.
        app.add.focus = AddFocus::Project;
        app.add.filter = "garden".into();
        app.apply_add_suggestion();
        assert_eq!(app.add.project, "garden");
        assert_eq!(app.add.focus, AddFocus::Desc);
        assert!(app.add.filter.is_empty());
        // A typed date passes through instead of needing a preset.
        app.add.focus = AddFocus::Due;
        app.add.filter = "2026-12-01".into();
        app.apply_add_suggestion();
        assert_eq!(app.add.due, "2026-12-01");
        assert_eq!(app.add.focus, AddFocus::Desc);
        app.add.desc = "plant seeds".into();
        app.quick_add_commit();
        let created = app
            .tasks
            .iter()
            .find(|t| t.desc == "plant seeds")
            .expect("custom project task created");
        assert_eq!(created.project, "garden");
        assert_eq!(created.due, "2026-12-01");
        // `no date` clears the due.
        app.add.due = "2026-12-01".into();
        app.add.focus = AddFocus::Due;
        app.add.filter = "no date".into();
        app.apply_add_suggestion();
        assert!(app.add.due.is_empty(), "no date clears the due");
        // The grey example vanishes once the filter has text.
        app.add.filter.clear();
        app.add.focus = AddFocus::Due;
        let c = draw(&app);
        assert!(c.contains("e.g. 2026-11-20"), "empty due shows an example");
        app.add.filter = "2".into();
        let c = draw(&app);
        assert!(
            !c.contains("e.g. 2026-11-20"),
            "typing replaces the example"
        );
    }

    #[test]
    fn overdue_editor_opens_raw_date_not_label() {
        let app = App::new();
        // Legacy mock overdue task carries only the friendly label: the
        // editor opens blank so the label is never edited as a date.
        let t = app.task(&mock_uuid(5)).unwrap().clone();
        assert!(is_overdue(&t), "fixture task is overdue");
        let p = Popup::open(&app, &t);
        assert!(!p.due.contains("overdue"), "editor never shows the label");
        // Backend task with a raw export date opens the calendar date.
        let mut dated = t.clone();
        dated.due_raw = Some("20261110T120000Z".into());
        let p = Popup::open(&app, &dated);
        assert_eq!(p.due, "2026-11-10", "overdue edits the due date");
    }

    #[test]
    fn tag_completion_replaces_last_token() {
        // Completing replaces the in-progress token instead of appending.
        assert_eq!(apply_tag_pick("work, leet", "leetcode"), "work, leetcode");
        assert_eq!(apply_tag_pick("leet", "leetcode"), "leetcode");
        assert_eq!(apply_tag_pick("", "leetcode"), "leetcode");
        assert_eq!(apply_tag_pick("work, ", "urgent"), "work, urgent");
        assert_eq!(tag_last_token("work, lee"), "lee");
        assert_eq!(tag_last_token("work, "), "");

        let mut app = App::new();
        let t = app.task(&mock_uuid(4)).unwrap().clone();
        let mut p = Popup::open(&app, &t);
        p.field = F_TAGS;
        // An empty in-progress token lists tags plus `no tags` first (shared cap).
        p.tags = "legal, ".into();
        let mut unfiltered = vec!["no tags".to_string()];
        unfiltered.extend(all_tags(&app).into_iter().take(CTX_SUGGEST_CAP - 1));
        assert_eq!(
            popup_suggestions(&app, &p),
            unfiltered,
            "empty filter lists all tags"
        );
        // A partial token filters the list.
        p.tags = "legal, plan".into();
        assert_eq!(
            popup_suggestions(&app, &p),
            vec!["planning".to_string()],
            "filter follows the last token"
        );
        // Applying the pick replaces that token.
        app.popup = Some(p);
        app.apply_popup_suggestion();
        assert_eq!(app.popup.as_ref().unwrap().tags, "legal, planning");
    }

    #[test]
    fn popup_done_key_toggles_without_closing() {
        let mut app = App::new();
        let id = mock_uuid(2);
        let t = app.task(&id).unwrap().clone();
        app.popup = Some(Popup::open(&app, &t));
        let mods = KeyModifiers::empty();
        popup_key(&mut app, KeyCode::Char('d'), mods);
        assert!(app.popup.is_some(), "popup stays open on complete");
        assert_eq!(app.task(&id).unwrap().status, "completed");
        assert!(
            !app.popup.as_ref().unwrap().dirty,
            "done is saved, not an unsaved draft"
        );
        popup_key(&mut app, KeyCode::Char('d'), mods);
        assert!(app.popup.is_some(), "popup stays open on reopen");
        assert_eq!(app.task(&id).unwrap().status, "pending");
        assert!(
            !app.popup.as_ref().unwrap().dirty,
            "reopen is saved, not an unsaved draft"
        );
    }

    #[test]
    fn list_scrolls_to_keep_cursor_visible() {
        let mut app = App::new();
        app.view = View::All;
        for i in 0..20u32 {
            let mut task = app.tasks[0].clone();
            task.uuid = mock_uuid(200 + i);
            task.short_id = Some(200 + i);
            task.desc = format!("scroll task {i:02}");
            task.project.clear();
            task.due = "—".into();
            task.due_raw = None;
            task.prio = 'L';
            task.tags.clear();
            task.section.clear();
            task.urgency = 40.0 - i as f32;
            task.active = false;
            task.status = "pending".into();
            task.depends.clear();
            task.recur = "none".into();
            task.start_raw = None;
            task.end = None;
            task.annotations.clear();
            app.tasks.push(task);
        }
        app.sync_projects();
        app.sync_sections();
        let last = mock_uuid(219);
        goto(&mut app, &last);

        let lines = draw_lines(&app);
        let c = lines.join("\n");
        assert!(c.contains("scroll task 19"), "last task scrolls into view");
        assert!(
            c.contains("Add a task"),
            "the capture header stays fixed while tasks scroll"
        );
        assert!(
            !c.contains("scroll task 00"),
            "the top task scrolls out of the viewport"
        );

        // Cursor up a few rows brings the earlier tail back into view.
        for _ in 0..5 {
            app.move_up();
        }
        let c = draw_lines(&app).join("\n");
        assert!(
            c.contains("scroll task 09") || c.contains("scroll task 10"),
            "moving up re-renders earlier tasks"
        );
    }

    #[test]
    fn list_row_n_does_nothing_without_subtasks() {
        let mut app = App::new();
        app.view = View::Project("work".into());
        let blocked = mock_uuid(3);
        goto(&mut app, &blocked);
        let id = app.cursor_id().expect("cursor on the blocked row");
        assert_eq!(id, blocked);
        let before = app.tasks.len();
        // `n` no longer creates subtasks; list count stays stable.
        route_key(&mut app, KeyCode::Char('n'));
        assert_eq!(app.tasks.len(), before, "n adds nothing");
        // A section header yields no task.
        let header = app
            .navigable_rows()
            .iter()
            .position(|r| matches!(r, NavRow::SectionHeader(_)))
            .expect("project view has a header");
        app.cursor = header;
        assert!(app.cursor_id().is_none(), "headers never yield a task");
    }

    #[test]
    fn toast_expires_after_timeout() {
        let mut app = App::new();
        app.set_toast("saved".into());
        app.expire_toast();
        assert_eq!(app.toast, "saved", "a fresh toast stays");
        // A second toast restarts its own timer.
        app.set_toast("completed 1 task(s)".into());
        app.toast_at = Some(Instant::now() - Duration::from_millis(1000));
        app.expire_toast();
        assert_eq!(app.toast, "completed 1 task(s)", "newer toast persists");
        // Once the timer lapses the bar clears.
        app.toast_at = Some(Instant::now() - Duration::from_millis(2600));
        app.expire_toast();
        assert!(app.toast.is_empty(), "stale toast clears");
        assert!(app.toast_at.is_none());
        // A direct assignment is stamped on the first expiry check and then
        // expires like any other toast.
        app.toast = "edits discarded".into();
        app.expire_toast();
        assert_eq!(app.toast, "edits discarded", "first pass stamps it");
        app.toast_at = Some(Instant::now() - Duration::from_millis(2600));
        app.expire_toast();
        assert!(app.toast.is_empty(), "direct toasts still expire");
    }

    #[test]
    fn today_all_and_search_render_flat() {
        let mut app = App::new();
        // Today, All, and Search show no section headers, only the
        // COMPLETED tail, and every top-level row sorts by urgency.
        for view in [View::Today, View::All, View::Search] {
            app.view = view;
            app.cursor_to_first_task();
            assert!(
                !app.navigable_rows()
                    .iter()
                    .any(|r| matches!(r, NavRow::SectionHeader(_))),
                "no section headers in {}",
                app.view.title()
            );
            let urgencies: Vec<f32> = app
                .open_rows()
                .iter()
                .filter_map(|id| app.task(id))
                .map(|t| t.urgency)
                .collect();
            assert!(
                urgencies.windows(2).all(|w| w[0] >= w[1]),
                "{} sorts tops by urgency descending",
                app.view.title()
            );
        }
        // Project views keep their headers.
        app.view = View::Project("work".into());
        assert!(
            app.navigable_rows()
                .iter()
                .any(|r| matches!(r, NavRow::SectionHeader(_))),
            "project views keep sections"
        );
    }

    #[test]
    fn search_view_matches_and_clears_on_exit() {
        let mut app = App::new();
        app.view = View::Search;
        assert!(!app.open_rows().is_empty(), "empty query lists open tasks");
        app.search_query = "nadia".into();
        let rows = app.open_rows();
        assert_eq!(rows.len(), 1);
        assert_eq!(
            app.task(&rows[0]).unwrap().desc,
            "Reply to Nadia about contract"
        );
        // Project and tag text match too.
        app.search_query = "legal".into();
        assert_eq!(app.open_rows().len(), 1, "tag text matches");
        app.search_query = "personal".into();
        assert!(
            app.open_rows()
                .iter()
                .all(|id| app.task(id).unwrap().project == "personal"),
            "project text matches"
        );
        // Leaving the view clears the query.
        app.switch_view(1);
        assert!(app.view == View::Inbox);
        assert!(app.search_query.is_empty(), "query clears on exit");
    }

    #[test]
    fn search_matches_fuzzy_text_and_id() {
        let mut app = App::new();
        app.view = View::Search;
        // Fuzzy description: `drat` hits both `Draft …` rows.
        app.search_query = "drat".into();
        assert!(
            app.open_rows()
                .iter()
                .any(|id| app.task(id).unwrap().desc == "Draft Q4 planning doc"),
            "fuzzy description matches"
        );
        // Short-ID substring: `3` hits task 3 (and 13).
        app.search_query = "3".into();
        assert!(
            app.open_rows()
                .iter()
                .any(|id| app.task(id).unwrap().short_id == Some(3)),
            "ID substring matches"
        );
        // UUID prefix matches.
        let prefix: String = app.tasks[0].uuid.chars().take(4).collect();
        app.search_query = prefix;
        assert!(!app.open_rows().is_empty(), "UUID prefix matches");
    }

    #[test]
    fn search_input_consumes_hotkeys() {
        let mut app = App::new();
        app.view = View::Search;
        app.search_input = true;
        let mods = KeyModifiers::empty();
        // `q`/`l`/`?` type instead of quitting, moving focus, or opening help.
        for c in ['q', 'l', '?'] {
            assert!(
                search_key(&mut app, KeyCode::Char(c), mods),
                "{c} is consumed"
            );
        }
        assert_eq!(app.search_query, "ql?");
        assert_eq!(app.view, View::Search, "view unchanged");
        assert!(!app.show_help, "help never opens while typing");
        // Tab/Shift+Tab and arrows move the results cursor, not the query.
        let query = app.search_query.clone();
        for code in [KeyCode::Tab, KeyCode::BackTab, KeyCode::Down, KeyCode::Up] {
            assert!(search_key(&mut app, code, mods));
        }
        assert_eq!(app.search_query, query, "navigation never edits the query");
        // `esc` leaves input mode but keeps the query.
        assert!(search_key(&mut app, KeyCode::Esc, mods));
        assert!(!app.search_input);
        assert_eq!(app.search_query, query);
        // List mode falls through to the shared list keymap for `?`, but
        // keeps `i` and `esc` for itself.
        assert!(
            !search_key(&mut app, KeyCode::Char('?'), mods),
            "list mode lets shared hotkeys fall through"
        );
        assert!(search_key(&mut app, KeyCode::Char('i'), mods));
        assert!(app.search_input, "i focuses input");
        // `esc` in list mode clears a non-empty query without leaving.
        app.search_input = false;
        assert!(search_key(&mut app, KeyCode::Esc, mods));
        assert!(app.search_query.is_empty(), "esc clears the query");
        assert_eq!(app.view, View::Search);
    }

    #[test]
    fn search_is_hotkey_only_and_returns_to_origin() {
        let mods = KeyModifiers::empty();
        let mut app = App::new();
        app.switch_to_view(View::Project("work".into()));
        app.open_search();
        assert_eq!(app.view, View::Search);
        assert!(app.search_input);
        for c in "nadia".chars() {
            search_key(&mut app, KeyCode::Char(c), mods);
        }
        // First esc leaves input, second clears the query, third returns.
        assert!(search_key(&mut app, KeyCode::Esc, mods));
        assert!(!app.search_input);
        assert_eq!(app.search_query, "nadia");
        assert!(search_key(&mut app, KeyCode::Esc, mods));
        assert!(app.search_query.is_empty());
        assert_eq!(app.view, View::Search);
        assert!(search_key(&mut app, KeyCode::Esc, mods));
        assert_eq!(
            app.view,
            View::Project("work".into()),
            "Search returns to the tab it came from"
        );
        assert_eq!(app.search_origin, None);
    }

    #[test]
    fn header_scrolls_project_tabs_to_the_active_one() {
        let mut app = App::new();
        for i in 0..18u32 {
            let mut task = app.tasks[0].clone();
            task.uuid = mock_uuid(400 + i);
            task.short_id = Some(400 + i);
            task.desc = format!("project tab task {i:02}");
            task.project = format!("project-{i:02}");
            task.urgency = 1.0;
            task.status = "pending".into();
            app.tasks.push(task);
        }
        app.sync_projects();
        app.switch_to_view(View::Project("project-12".into()));
        let backend = TestBackend::new(80, 40);
        let mut term = Terminal::new(backend).unwrap();
        term.draw(|f| render(f, &app)).unwrap();
        let buf = term.backend().buffer();
        let header: String = buf
            .content()
            .chunks(80)
            .next()
            .unwrap()
            .iter()
            .map(|c| c.symbol())
            .collect();
        assert!(
            header.contains("project-12"),
            "active project tab stays visible: {header}"
        );
        for tab in ["Inbox", "Today", "All"] {
            assert!(
                header.contains(tab),
                "fixed tab {tab} stays pinned: {header}"
            );
        }
        assert!(
            header.contains('‹') || header.contains('›'),
            "overflow chevrons show hidden tabs: {header}"
        );
    }

    #[test]
    fn search_caret_moves_and_edits_in_place() {
        use KeyCode as K;
        let mods = KeyModifiers::empty();
        let mut app = App::new();
        app.view = View::Search;
        app.search_input = true;
        for c in "abc".chars() {
            search_key(&mut app, K::Char(c), mods);
        }
        assert_eq!(app.search_query, "abc");
        assert_eq!(app.search_cursor, 3);
        search_key(&mut app, K::Left, mods);
        search_key(&mut app, K::Left, mods);
        assert_eq!(app.search_cursor, 1, "Left moves one char");
        search_key(&mut app, K::Char('X'), mods);
        assert_eq!(app.search_query, "aXbc", "typing inserts at the caret");
        assert_eq!(app.search_cursor, 2);
        let (_, cursor) = draw_with_cursor(&app);
        // List pane x=0; two chars before the caret.
        assert_eq!(cursor, Position::new(2, 4));
        search_key(&mut app, K::Backspace, mods);
        assert_eq!(app.search_query, "abc");
        search_key(&mut app, K::Delete, mods);
        assert_eq!(app.search_query, "ac", "Delete removes after the caret");
        assert_eq!(app.search_cursor, 1);
        // Re-entering input mode parks the caret at the query's end.
        search_key(&mut app, K::Esc, mods);
        assert!(!app.search_input);
        search_key(&mut app, K::Char('i'), mods);
        assert!(app.search_input);
        assert_eq!(app.search_cursor, 2, "i resumes at the query end");
    }

    #[test]
    fn edit_mode_shows_grey_examples_for_empty_typed_fields() {
        let mut app = App::new();
        // Mock 2 is project-less with no due date and no dependency.
        let t = app.task(&mock_uuid(2)).unwrap().clone();
        app.popup = Some(Popup::open(&app, &t));
        let c = draw(&app);
        assert!(!c.contains("e.g."), "VIEW shows no examples");
        let mods = KeyModifiers::empty();
        popup_key(&mut app, KeyCode::Char('e'), mods);
        // Empty typed fields show their grey example while focused in EDIT.
        {
            let p = app.popup.as_mut().unwrap();
            p.field = F_DUE;
        }
        let c = draw(&app);
        assert!(c.contains("e.g. 2026-11-20"), "empty due shows its example");
        {
            let p = app.popup.as_mut().unwrap();
            p.field = F_DEPENDS;
        }
        let c = draw(&app);
        assert!(
            !c.contains("e.g. 12, 14"),
            "fixed-list depends shows no typing example"
        );
        {
            let p = app.popup.as_mut().unwrap();
            p.field = F_PROJECT;
        }
        let c = draw(&app);
        assert!(c.contains("e.g. work"), "empty project shows its example");
        // Non-empty fields never show one.
        {
            let p = app.popup.as_mut().unwrap();
            p.field = F_TAGS;
        }
        let c = draw(&app);
        assert!(!c.contains("e.g. work, urgent"), "set tags show no example");
        // Typing replaces the example.
        {
            let p = app.popup.as_mut().unwrap();
            p.field = F_DUE;
        }
        popup_key(&mut app, KeyCode::Char('2'), mods);
        let c = draw(&app);
        assert!(!c.contains("e.g. 2026-11-20"), "input replaces the example");
        // Picker-only fields never show an example.
        {
            let p = app.popup.as_mut().unwrap();
            p.field = F_PRIO;
        }
        let c = draw(&app);
        assert!(!c.contains("e.g."), "priority shows no example");
    }

    #[test]
    fn editor_contextual_block_mirrors_add_box() {
        let mut app = App::new();
        // Mock 3 is a `work` H-priority task; the draft opens on its values.
        let t = app.task(&mock_uuid(3)).unwrap().clone();
        app.popup = Some(Popup::open(&app, &t));
        let mods = KeyModifiers::empty();
        popup_key(&mut app, KeyCode::Char('e'), mods);
        // Empty project draft: the field row itself carries the grey
        // example (shared with the add box) with the block cursor inline,
        // and the saved value shows as the `now:` reference below it.
        // No duplicated input line repeats the draft.
        {
            let p = app.popup.as_mut().unwrap();
            p.field = F_PROJECT;
            p.project.clear();
        }
        let (lines, cursor) = draw_with_cursor(&app);
        assert!(
            lines.iter().any(|l| l.contains("e.g. work")),
            "empty project echoes its example"
        );
        assert!(
            lines.iter().any(|l| l.contains("now: @ work")),
            "saved project shows as now:"
        );
        assert!(
            !lines.iter().any(|l| l.contains('▌')),
            "no inline typing-cursor glyph"
        );
        assert_eq!(
            cursor,
            Position::new(78, 7),
            "terminal caret sits at the start of the empty project example"
        );
        // Priority is picker-only: no cursor line, but the same `now:` row.
        {
            let p = app.popup.as_mut().unwrap();
            p.field = F_PRIO;
        }
        let (lines, cursor) = draw_with_cursor(&app);
        assert!(
            lines.iter().any(|l| l.contains("now: ! H")),
            "priority echoes its saved value"
        );
        assert_eq!(
            cursor,
            Position::new(119, 39),
            "priority is picker-only: no typing caret"
        );
        // Suggestion data caps match the add box; depends keeps its
        // scrollable list (render window shows the shared count) with the
        // clear row always last.
        {
            let p = app.popup.as_mut().unwrap();
            p.field = F_PROJECT;
            p.project.clear();
        }
        let sugs = popup_suggestions(&app, app.popup.as_ref().unwrap());
        assert!(
            sugs.len() <= CTX_SUGGEST_CAP,
            "project caps at the shared count"
        );
        {
            let p = app.popup.as_mut().unwrap();
            p.field = F_DEPENDS;
            p.depends.clear();
        }
        let sugs = popup_suggestions(&app, app.popup.as_ref().unwrap());
        assert_eq!(
            sugs.last().map(String::as_str),
            Some("no dependencies"),
            "clear row stays last"
        );
        let mut draft = AddDraft::fresh();
        draft.focus = AddFocus::Tags;
        let sugs = add_suggestions(&app, &draft);
        assert!(sugs.len() <= CTX_SUGGEST_CAP, "add box caps match");
        assert_eq!(sugs[0], "no tags", "add box shares the clear row");
    }

    #[test]
    fn depends_dropdown_is_project_scoped_and_enter_adds() {
        let mut app = App::new();
        // Mock 3 is `work` with `depends: 12`; mock 2 is project-less.
        let blocked = mock_uuid(3);
        let t = app.task(&blocked).unwrap().clone();
        app.popup = Some(Popup::open(&app, &t));
        // Focus DEPENDS so suggestions come from the dependency index.
        app.popup.as_mut().unwrap().field = F_DEPENDS;
        // Dropdown candidates are open `work` tasks excluding self.
        let ids: Vec<String> = depends_candidates(&app, &blocked)
            .iter()
            .map(|t| task_id_display(t))
            .collect();
        assert!(!ids.iter().any(|id| id == "3"), "self excluded");
        assert!(
            ids.iter().any(|id| id == "12"),
            "same-project blocker listed"
        );
        assert!(ids.iter().any(|id| id == "4"), "same-project peer listed");
        assert!(
            !ids.iter().any(|id| id == "1" || id == "2"),
            "no-project tasks hidden from project tasks"
        );
        // Suggestions hide already-added IDs and show `ID  desc`.
        let sugs = popup_suggestions(&app, app.popup.as_ref().unwrap());
        assert!(
            sugs.iter().any(|s| s.starts_with("4  ")),
            "suggestion shows ID + description"
        );
        assert!(
            !sugs.iter().any(|s| s.starts_with("12 ")),
            "already-added IDs hide"
        );
        // Enter applies the highlight and saves back to VIEW at once.
        {
            let p = app.popup.as_mut().unwrap();
            p.mode = PopupMode::Edit;
            p.field = F_DEPENDS;
            p.depends.clear();
            p.suggest_idx = 0;
        }
        let expected = {
            let p = app.popup.as_ref().unwrap();
            let fresh = popup_suggestions(&app, p);
            assert!(!fresh.is_empty(), "cleared draft resows candidates");
            depends_suggestion_id(&fresh[0])
        };
        let mods = KeyModifiers::empty();
        popup_key(&mut app, KeyCode::Enter, mods);
        {
            let p = app.popup.as_ref().unwrap();
            assert_eq!(p.mode, PopupMode::View, "Enter saves back to VIEW");
            assert!(
                parse_depends_tokens(&p.depends).contains(&expected),
                "picked ID lands in the draft"
            );
        }
        let saved = app.task(&blocked).unwrap();
        assert_eq!(saved.depends.len(), 1, "one dep saved");
        // Unknown IDs and self-links toast and keep the draft.
        {
            let p = app.popup.as_mut().unwrap();
            p.mode = PopupMode::Edit;
            p.field = F_DEPENDS;
            p.depends = "999".into();
        }
        app.save_popup();
        assert_eq!(app.toast, "unknown id: 999");
        {
            let p = app.popup.as_mut().unwrap();
            p.depends = "3".into();
        }
        app.save_popup();
        assert_eq!(app.toast, "can't depend on itself");
        assert_eq!(app.task(&blocked).unwrap().depends.len(), 1);
    }

    #[test]
    fn depends_no_dependencies_clears_and_saves_at_once() {
        let mut app = App::new();
        // Mock 3 is `work` with `depends: 12`: the draft starts non-empty.
        let blocked = mock_uuid(3);
        assert!(!app.task(&blocked).unwrap().depends.is_empty());
        let t = app.task(&blocked).unwrap().clone();
        app.popup = Some(Popup::open(&app, &t));
        // `no dependencies` shows even from an empty draft, mirroring
        // `no project` / `no tags`.
        {
            let p = app.popup.as_mut().unwrap();
            p.mode = PopupMode::Edit;
            p.field = F_DEPENDS;
            p.depends.clear();
            p.suggest_idx = 0;
        }
        assert!(
            popup_suggestions(&app, app.popup.as_ref().unwrap())
                .iter()
                .any(|s| s == "no dependencies"),
            "empty draft still offers no dependencies"
        );
        {
            // Restore the saved draft: the clear-and-save path needs a
            // non-empty starting value.
            let display = depends_display_for(&app, &t);
            let p = app.popup.as_mut().unwrap();
            p.depends = display;
            assert!(
                !p.depends.trim().is_empty(),
                "draft starts with the saved dep"
            );
        }
        let clear_idx = {
            let p = app.popup.as_ref().unwrap();
            popup_suggestions(&app, p)
                .iter()
                .position(|s| s == "no dependencies")
                .expect("no dependencies offered for a non-empty draft")
        };
        {
            let p = app.popup.as_mut().unwrap();
            p.suggest_idx = clear_idx;
        }
        let mods = KeyModifiers::empty();
        popup_key(&mut app, KeyCode::Enter, mods);
        assert!(
            app.task(&blocked).unwrap().depends.is_empty(),
            "one Enter on no dependencies clears and saves"
        );
        assert_eq!(app.popup.as_ref().unwrap().mode, PopupMode::View);
        assert_eq!(app.toast, "saved");
    }

    #[test]
    fn due_presets_drop_weekend_and_next_week() {
        let presets = due_presets();
        assert_eq!(
            presets,
            vec![
                "today".to_string(),
                "tomorrow".to_string(),
                "no date".to_string()
            ],
            "date list stays minimal"
        );
    }

    #[test]
    fn fixed_lists_use_jk_including_more() {
        use KeyCode as K;
        let mods = KeyModifiers::empty();
        // Add box: j/k moves the fixed priority list instead of typing.
        let mut app = App::new();
        app.add_mode = true;
        app.add.focus = AddFocus::Priority;
        app.add.picker = 0;
        add_key(&mut app, K::Char('j'), mods);
        assert_eq!(app.add.picker, 1, "j moves the priority highlight");
        add_key(&mut app, K::Char('k'), mods);
        assert_eq!(app.add.picker, 0, "k moves it back");
        // The + more menu owns j/k too and lists the depends entry.
        app.add.focus = AddFocus::More;
        app.add.picker = 0;
        add_key(&mut app, K::Char('j'), mods);
        assert_eq!(app.add.picker, 1, "j moves the more highlight");
        let sugs = add_suggestions(&app, &app.add);
        assert!(
            sugs.iter().any(|s| s == "& depends"),
            "+ more opens the depends picker"
        );
        // Popup: j/k moves the fixed priority highlight in EDIT.
        let t = app.task(&mock_uuid(3)).unwrap().clone();
        app.popup = Some(Popup::open(&app, &t));
        {
            let p = app.popup.as_mut().unwrap();
            p.mode = PopupMode::Edit;
            p.field = F_PRIO;
            p.suggest_idx = 0;
        }
        popup_key(&mut app, K::Char('j'), mods);
        assert_eq!(
            app.popup.as_ref().unwrap().suggest_idx,
            1,
            "j moves the popup priority highlight"
        );
        popup_key(&mut app, K::Char('k'), mods);
        assert_eq!(
            app.popup.as_ref().unwrap().suggest_idx,
            0,
            "k moves it back"
        );
    }

    #[test]
    fn depends_enter_appends_highlight_and_typing_is_ignored() {
        let mut app = App::new();
        // Mock 3 (`work`) already waits on 12; the draft starts with it.
        let blocked = mock_uuid(3);
        let t = app.task(&blocked).unwrap().clone();
        app.popup = Some(Popup::open(&app, &t));
        {
            let p = app.popup.as_mut().unwrap();
            p.mode = PopupMode::Edit;
            p.field = F_DEPENDS;
            assert!(
                parse_depends_tokens(&p.depends).contains(&"12".to_string()),
                "draft starts with the saved link"
            );
        }
        // Typing does nothing in the fixed list: the draft keeps only the
        // saved link, and the highlight stays where it was.
        let mods = KeyModifiers::empty();
        popup_key(&mut app, KeyCode::Char('9'), mods);
        popup_key(&mut app, KeyCode::Char('x'), mods);
        assert_eq!(
            parse_depends_tokens(app.popup.as_ref().unwrap().depends.as_str()),
            vec!["12".to_string()],
            "typed keys never touch the draft"
        );
        // Enter appends the highlighted row and saves at once.
        let idx = {
            let p = app.popup.as_ref().unwrap();
            popup_suggestions(&app, p)
                .iter()
                .position(|s| depends_suggestion_id(s) == "4")
                .expect("peer 4 is suggested")
        };
        app.popup.as_mut().unwrap().suggest_idx = idx;
        popup_key(&mut app, KeyCode::Enter, mods);
        let p = app.popup.as_ref().unwrap();
        let tokens = parse_depends_tokens(&p.depends);
        assert!(
            tokens.contains(&"12".to_string()),
            "existing link survives Enter"
        );
        assert!(tokens.contains(&"4".to_string()), "highlight appends");
        assert_eq!(p.mode, PopupMode::View, "Enter saves back to VIEW");
        let saved = app.task(&blocked).unwrap();
        assert_eq!(saved.depends.len(), 2, "both links save");
    }

    #[test]
    fn depends_circular_links_toast_and_keep_draft() {
        let mut app = App::new();
        // Mock 3 already waits on 12: linking 12 back to 3 closes a
        // two-node loop.
        let blocker = mock_uuid(12);
        let t = app.task(&blocker).unwrap().clone();
        app.popup = Some(Popup::open(&app, &t));
        {
            let p = app.popup.as_mut().unwrap();
            p.mode = PopupMode::Edit;
            p.field = F_DEPENDS;
            p.depends = "3".into();
        }
        app.save_popup();
        assert_eq!(app.toast, "circular dependency");
        assert!(
            app.task(&blocker).unwrap().depends.is_empty(),
            "cycle never saves"
        );
        assert_eq!(app.popup.as_ref().unwrap().mode, PopupMode::Edit);

        // Longer chain: 3 -> 12 -> 4, so 4 -> 3 loops back transitively.
        app.tasks
            .iter_mut()
            .find(|t| t.uuid == blocker)
            .unwrap()
            .depends = vec![mock_uuid(4)];
        let peer = mock_uuid(4);
        let t = app.task(&peer).unwrap().clone();
        app.popup = Some(Popup::open(&app, &t));
        {
            let p = app.popup.as_mut().unwrap();
            p.mode = PopupMode::Edit;
            p.field = F_DEPENDS;
            p.depends = "3".into();
        }
        app.save_popup();
        assert_eq!(app.toast, "circular dependency");
        assert!(app.task(&peer).unwrap().depends.is_empty());
    }

    #[test]
    fn popup_depends_backspace_pops_last_link_and_saves() {
        let mut app = App::new();
        // Mock 3 (`work`) already waits on 12; append 4 first so there are
        // two links to strip.
        let blocked = mock_uuid(3);
        let t = app.task(&blocked).unwrap().clone();
        app.popup = Some(Popup::open(&app, &t));
        {
            let p = app.popup.as_mut().unwrap();
            p.mode = PopupMode::Edit;
            p.field = F_DEPENDS;
            p.depends = "12, 4".into();
        }
        app.save_popup();
        assert_eq!(app.task(&blocked).unwrap().depends.len(), 2);
        let mods = KeyModifiers::empty();
        // Backspace pops the last link and saves, staying in EDIT.
        {
            let p = app.popup.as_mut().unwrap();
            p.mode = PopupMode::Edit;
            p.field = F_DEPENDS;
        }
        popup_key(&mut app, KeyCode::Backspace, mods);
        {
            let p = app.popup.as_ref().unwrap();
            assert_eq!(p.mode, PopupMode::Edit, "stays in EDIT for repeats");
            assert_eq!(
                parse_depends_tokens(&p.depends),
                vec!["12".to_string()],
                "last link pops"
            );
        }
        assert_eq!(
            app.task(&blocked).unwrap().depends.len(),
            1,
            "removal saves at once"
        );
        // A second Backspace clears the final link the same way.
        popup_key(&mut app, KeyCode::Backspace, mods);
        assert!(
            app.popup.as_ref().unwrap().depends.trim().is_empty(),
            "draft clears"
        );
        assert!(
            app.task(&blocked).unwrap().depends.is_empty(),
            "empty saves"
        );
        assert_eq!(app.popup.as_ref().unwrap().mode, PopupMode::Edit);
        // Backspace on an empty draft does nothing.
        app.clear_toast();
        popup_key(&mut app, KeyCode::Backspace, mods);
        assert!(app.toast.is_empty(), "no toast on empty draft");
        assert!(app.task(&blocked).unwrap().depends.is_empty());
    }

    #[test]
    fn add_box_depends_fixed_list_appends_highlight() {
        let mut app = App::new();
        app.add_mode = true;
        app.add.focus = AddFocus::Depends;
        app.add.project = "work".into();
        let mods = KeyModifiers::empty();
        // Typing is ignored: the filter stays empty and no link is added.
        add_key(&mut app, KeyCode::Char('9'), mods);
        add_key(&mut app, KeyCode::Char('x'), mods);
        assert!(app.add.filter.is_empty(), "typing never fills the filter");
        assert!(app.add.depends.is_empty(), "typing adds no link");
        // j/k moves the highlight instead of typing.
        let before = app.add.picker;
        add_key(&mut app, KeyCode::Char('j'), mods);
        assert_eq!(app.add.picker, before + 1, "j moves the highlight");
        add_key(&mut app, KeyCode::Char('k'), mods);
        assert_eq!(app.add.picker, before, "k moves it back");
        // Enter appends the highlighted row and stays for multi-entry.
        let idx = add_suggestions(&app, &app.add)
            .iter()
            .position(|s| depends_suggestion_id(s) == "4")
            .expect("peer 4 is suggested");
        app.add.picker = idx;
        add_key(&mut app, KeyCode::Enter, mods);
        assert_eq!(app.add.depends, vec!["4".to_string()]);
        assert!(
            app.add_mode && app.add.focus == AddFocus::Depends,
            "stays in the picker for multi-entry"
        );
        // Suggestions offer project peers plus the clear row.
        let sugs = add_suggestions(&app, &app.add);
        assert!(
            sugs.iter().any(|s| s.starts_with("12  ")),
            "project peer listed"
        );
        assert!(
            !sugs.iter().any(|s| s.starts_with("4  ")),
            "added links hide"
        );
        assert_eq!(sugs.last().map(String::as_str), Some("no dependencies"));
        // Backspace on the empty filter pops the last link.
        add_key(&mut app, KeyCode::Backspace, mods);
        assert!(app.add.depends.is_empty(), "Backspace pops the link");
        // Re-add, then commit carries the link.
        app.add.picker = add_suggestions(&app, &app.add)
            .iter()
            .position(|s| depends_suggestion_id(s) == "4")
            .expect("peer 4 is suggested");
        add_key(&mut app, KeyCode::Enter, mods);
        app.add.desc = "depends add".into();
        app.quick_add_commit();
        let created = app
            .tasks
            .iter()
            .find(|t| t.desc == "depends add")
            .expect("depends task created");
        assert_eq!(created.depends.len(), 1, "one link stored");
        assert!(app.add.depends.is_empty(), "draft clears after commit");
        app.add.desc = "   ".into();
        app.quick_add_commit();
        assert_eq!(app.toast, "description required");
    }

    #[test]
    fn recur_edit_save_and_list_badge() {
        let mut app = App::new();
        app.view = View::All;
        // Popup offers the fixed presets plus an explicit clear.
        let id = mock_uuid(4);
        let t = app.task(&id).unwrap().clone();
        app.popup = Some(Popup::open(&app, &t));
        app.popup.as_mut().unwrap().field = F_RECUR;
        let sugs = popup_suggestions(&app, app.popup.as_ref().unwrap());
        assert_eq!(
            sugs,
            vec!["daily", "weekly", "monthly", "yearly", "no recurrence"]
        );
        // Recurrence without a due date is refused.
        {
            let p = app.popup.as_mut().unwrap();
            p.mode = PopupMode::Edit;
            p.recur = "weekly".into();
            p.due.clear();
            p.dirty = true;
        }
        app.save_popup();
        assert_eq!(app.toast, "recurrence needs a due date");
        assert_eq!(
            app.popup.as_ref().unwrap().mode,
            PopupMode::Edit,
            "stays in EDIT on validation"
        );
        // Setting both saves and shows the list badge.
        {
            let p = app.popup.as_mut().unwrap();
            p.due = "tomorrow".into();
            p.dirty = true;
        }
        app.save_popup();
        assert_eq!(app.toast, "saved");
        assert_eq!(app.task(&id).unwrap().recur, "weekly");
        app.popup = None;
        let rows = draw_lines(&app);
        let idx = rows
            .iter()
            .position(|l| l.contains("Draft Q4 planning doc"))
            .unwrap_or_else(|| panic!("edited row missing: {rows:#?}"));
        assert!(
            rows[idx + 1].contains("weekly"),
            "metadata shows the recurrence badge: {}",
            rows[idx + 1]
        );
        // Clearing back to none drops the badge.
        let t = app.task(&id).unwrap().clone();
        app.popup = Some(Popup::open(&app, &t));
        {
            let p = app.popup.as_mut().unwrap();
            p.field = F_RECUR;
            p.mode = PopupMode::Edit;
            p.recur.clear();
            p.dirty = true;
        }
        app.save_popup();
        assert_eq!(app.task(&id).unwrap().recur, "none");
        app.popup = None;
        let rows = draw_lines(&app);
        let idx = rows
            .iter()
            .position(|l| l.contains("Draft Q4 planning doc"))
            .expect("cleared row renders");
        assert!(
            !rows[idx + 1].contains("weekly"),
            "badge clears with recurrence"
        );
    }

    #[test]
    fn clearing_recurrence_on_future_task_keeps_earlier_occurrences() {
        let mut app = App::new();
        app.view = View::All;
        let keep = mock_uuid(4);
        let previous = mock_uuid(5);
        let finished = mock_uuid(6);
        let later = mock_uuid(7);
        let template = "template-1".to_string();
        for (uuid, due) in [
            (&keep, "20261120T000000Z"),
            (&previous, "20261119T000000Z"),
            (&finished, "20261118T000000Z"),
            (&later, "20261121T000000Z"),
        ] {
            let t = app.tasks.iter_mut().find(|t| &t.uuid == uuid).unwrap();
            t.recur = "weekly".into();
            t.recur_parent = template.clone();
            t.due_raw = Some(due.into());
        }
        app.tasks
            .iter_mut()
            .find(|t| t.uuid == finished)
            .unwrap()
            .status = "completed".into();
        app.selected.insert(previous.clone());
        app.selected.insert(later.clone());

        let t = app.task(&keep).unwrap().clone();
        app.popup = Some(Popup::open(&app, &t));
        {
            let p = app.popup.as_mut().unwrap();
            p.field = F_RECUR;
            p.mode = PopupMode::Edit;
            p.recur.clear();
            p.dirty = true;
        }
        app.save_popup();

        assert_eq!(app.toast, "saved");
        let kept = app.task(&keep).expect("selected future occurrence stays");
        assert_eq!(kept.recur, "none");
        assert!(kept.recur_parent.is_empty());
        let earlier = app.task(&previous).expect("earlier occurrence stays");
        assert_eq!(
            earlier.recur, "none",
            "earlier occurrence becomes a one-off"
        );
        assert!(
            earlier.recur_parent.is_empty(),
            "earlier occurrence detaches from the template"
        );
        assert!(
            app.selected.contains(&previous),
            "kept earlier occurrence stays selected"
        );
        assert!(app.task(&later).is_none(), "later occurrence is removed");
        assert!(
            !app.selected.contains(&later),
            "removed later occurrence leaves the selection"
        );
        let history = app
            .task(&finished)
            .expect("completed occurrence history survives");
        assert_eq!(history.recur, "weekly", "completed history is untouched");
        assert_eq!(history.recur_parent, template);
    }

    #[test]
    fn main_list_shows_id_next_to_due_while_relations_keep_it() {
        let mut app = App::new();
        app.view = View::All;
        let lines = draw_lines(&app);
        let main_col = |l: &String| -> String { l.trim().to_string() };
        // First line carries the description with no ID column.
        let pos = lines
            .iter()
            .position(|l| main_col(l).contains("Reply to Nadia"))
            .expect("blocked row renders");
        let row = main_col(&lines[pos]);
        assert!(!row.contains('3'), "first line drops the ID column");
        assert!(row.contains("Reply to Nadia"), "desc renders");
        // Second line carries `id · due` metadata.
        let meta = main_col(&lines[pos + 1]);
        assert!(
            meta.contains('3') && meta.contains("today"),
            "metadata shows ID next to due: {meta}"
        );
        // The BLOCKED BY / BLOCKS footer in the popup reuses the same rows.
        let t = app.task(&mock_uuid(3)).unwrap().clone();
        app.popup = Some(Popup::open(&app, &t));
        let c = draw(&app);
        assert!(c.contains("BLOCKED BY"), "relation section renders");
        let lines = draw_lines(&app);
        let blocked_pos = lines
            .iter()
            .position(|l| l.contains("BLOCKED BY"))
            .expect("blocked-by label renders");
        let rel = lines[blocked_pos..]
            .iter()
            .find(|l| l.contains("Draft cover email"))
            .expect("blocker relation renders");
        assert!(rel.contains("12"), "relation rows carry the task ID too");
        // Compact single-row layout: ID, description, and due share one line.
        assert!(
            rel.contains("Draft cover email") && rel.contains("12"),
            "single-row relation keeps ID and desc together: {rel}"
        );
        assert!(rel.contains("today"), "relation keeps due: {rel}");
        assert!(
            !rel.contains("work"),
            "relation drops project, keeping only due: {rel}"
        );
    }
}
