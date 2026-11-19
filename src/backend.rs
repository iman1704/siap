//! taskwarrior CLI backend skeleton.
//!
//! Reads and writes live tasks through the `task` subprocess. All commands
//! inherit the process environment so `TASKDATA`/`TASKRC` overrides keep
//! tests off live data.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Resolve the task data directory without touching the environment.
///
/// `taskdata` is the `TASKDATA` override when present; otherwise fall back
/// to the conventional `~/.task` location. Pure for unit testing.
pub fn resolve_data_dir(taskdata: Option<String>) -> PathBuf {
    match taskdata {
        Some(dir) if !dir.is_empty() => PathBuf::from(dir),
        _ => {
            let home = std::env::var("HOME").unwrap_or_else(|_| String::from("~"));
            PathBuf::from(home).join(".task")
        }
    }
}

/// Current task data directory honoring `TASKDATA`.
pub fn data_dir() -> PathBuf {
    resolve_data_dir(std::env::var("TASKDATA").ok())
}

fn task_command(args: &[&str]) -> Command {
    let mut cmd = Command::new("task");
    cmd.args(args);
    cmd
}

/// Run a `task` write command, capturing stderr for toasts.
///
/// Returns `Ok(stdout)` on success, `Err(stderr)` on failure. Stderr is
/// preferred; when empty, stdout or the status code is used so failures
/// always explain themselves.
///
/// Uses per-command `TASKDATA`/`TASKRC` overrides when given so tests can
/// isolate without mutating process env; otherwise inherits the process
/// environment (production live data or caller-set `TASKDATA`).
pub fn run_write_with_env(
    args: &[&str],
    taskdata: Option<&Path>,
    taskrc: Option<&Path>,
) -> Result<String, String> {
    let mut cmd = task_command(args);
    if let Some(dir) = taskdata {
        cmd.env("TASKDATA", dir);
    }
    if let Some(rc) = taskrc {
        cmd.env("TASKRC", rc);
    }
    let output = cmd.output().map_err(|e| e.to_string())?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    if output.status.success() {
        Ok(stdout)
    } else if !stderr.trim().is_empty() {
        Err(stderr.trim().to_string())
    } else if !stdout.trim().is_empty() {
        Err(stdout.trim().to_string())
    } else {
        Err(format!("task {} failed: {}", args.join(" "), output.status))
    }
}

pub fn run_write(args: &[&str]) -> Result<String, String> {
    run_write_with_env(args, None, None)
}

/// Args for the open-rows export (`+PENDING`). Pure for testing without live data.
pub fn pending_args() -> Vec<String> {
    vec!["+PENDING".into(), "export".into()]
}

/// Args for the completed-tail export (`+COMPLETED`). Pure for testing.
pub fn completed_args() -> Vec<String> {
    vec!["+COMPLETED".into(), "export".into()]
}

/// Export open tasks (`+PENDING`) as JSON. `deleted` is never fetched.
pub fn export_pending() -> Result<String, String> {
    let args = pending_args();
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    run_write(&refs)
}

/// Export completed tasks (`+COMPLETED`) as JSON for the capped tail.
pub fn export_completed() -> Result<String, String> {
    let args = completed_args();
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    run_write(&refs)
}

/// Args to register the section UDA as a string, bypassing any prompt.
pub fn section_uda_args() -> Vec<String> {
    vec![
        "rc.confirmation=off".to_string(),
        "config".to_string(),
        "uda.siapsection.type".to_string(),
        "string".to_string(),
    ]
}

/// Register the section UDA as a string type.
///
/// The concrete name is `siapsection` (no dot): taskwarrior 3.5 treats a
/// dotted `siap.section` attribute as description text, so the dotted
/// spec example is unusable as a live attribute. Reads still accept the
/// dotted keys as fixtures. Registration is non-interactive so launch
/// never waits on a confirmation prompt.
pub fn ensure_section_uda() -> Result<String, String> {
    let args = section_uda_args();
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    run_write(&refs)
}

/// True when a recur draft means "no recurrence" (empty, `none`,
/// or the explicit clear row). Shared by add and popup saves.
pub fn is_recur_cleared(recur: &str) -> bool {
    let r = recur.trim();
    r.is_empty() || r.eq_ignore_ascii_case("none") || r.eq_ignore_ascii_case("no recurrence")
}

/// Args for `task add` from quick-add fields. Pure for unit testing.
///
/// Skips empty project/due, skips priority when `None` (absent renders
/// P3), adds `+tag` per tag, stamps `siapsection:` when the draft carries a
/// `siapsection:` when the draft carries a
/// section (from `a` on a section context). `depends` carries
/// blocking UUIDs as one `depends:u1,u2` attribute so the create stays
/// a single write. `recur` is skipped when cleared, otherwise sent as
/// one `recur:<value>` attribute (taskwarrior requires a due date).
#[allow(clippy::too_many_arguments)] // add fields travel together; grouping adds noise.
pub fn build_add_args(
    desc: &str,
    project: &str,
    due: &str,
    prio: Option<char>,
    tags: &[String],
    depends: &[String],
    recur: &str,
    section: &str,
) -> Vec<String> {
    let mut args = vec!["add".to_string(), desc.to_string()];
    if !project.trim().is_empty() {
        args.push(format!("project:{}", project.trim()));
    }
    if !due.trim().is_empty() {
        args.push(format!("due:{}", due.trim()));
    }
    if let Some(p) = prio {
        let v = match p {
            'H' => "H",
            'M' => "M",
            _ => "L",
        };
        args.push(format!("priority:{v}"));
    }
    for t in tags {
        let tag = t.trim();
        if !tag.is_empty() {
            args.push(format!("+{tag}"));
        }
    }
    let deps: Vec<String> = depends
        .iter()
        .map(|d| d.trim().to_string())
        .filter(|d| !d.is_empty())
        .collect();
    if !deps.is_empty() {
        args.push(format!("depends:{}", deps.join(",")));
    }
    if !is_recur_cleared(recur) {
        args.push(format!("recur:{}", recur.trim()));
    }
    if !section.trim().is_empty() {
        args.push(format!("siapsection:{}", section.trim()));
    }
    args
}

/// Args for `task <uuid> modify` from popup fields. Pure for testing.
///
/// Always sets description/project/due/priority/recur/section so clears
/// persist; tags are diffed (`+added`/`-removed`) from old to new.
#[allow(clippy::too_many_arguments)] // popup fields travel together; grouping adds noise.
pub fn build_modify_args(
    uuid: &str,
    desc: &str,
    project: &str,
    due: &str,
    prio: char,
    old_tags: &[String],
    new_tags: &[String],
    section: &str,
    recur: &str,
) -> Vec<String> {
    let mut args = vec![uuid.to_string(), "modify".to_string()];
    args.push(format!("description:{desc}"));
    // Empty project/due clears the field (`project:`/`due:`).
    args.push(format!("project:{}", project.trim()));
    let due_arg = due.trim();
    if due_arg.is_empty() || due_arg == "—" {
        args.push("due:".to_string());
    } else {
        args.push(format!("due:{due_arg}"));
    }
    let prio_v = match prio {
        'H' => "H",
        'M' => "M",
        _ => "L",
    };
    args.push(format!("priority:{prio_v}"));
    // Empty/`none` clears the field (`recur:`); otherwise set it.
    if is_recur_cleared(recur) {
        args.push("recur:".to_string());
    } else {
        args.push(format!("recur:{}", recur.trim()));
    }
    let old: std::collections::HashSet<&str> = old_tags
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    let new: std::collections::HashSet<&str> = new_tags
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    for tag in new.difference(&old) {
        args.push(format!("+{tag}"));
    }
    for tag in old.difference(&new) {
        args.push(format!("-{tag}"));
    }
    if !section.trim().is_empty() {
        args.push(format!("siapsection:{}", section.trim()));
    }
    args
}

/// Args for `task <uuid> start`. Pure for unit testing.
pub fn build_start_args(uuid: &str) -> Vec<String> {
    vec![uuid.to_string(), "start".to_string()]
}

/// Args for `task <uuid> stop`. Pure for unit testing.
pub fn build_stop_args(uuid: &str) -> Vec<String> {
    vec![uuid.to_string(), "stop".to_string()]
}

/// Args for `task <uuid> done`. Pure for unit testing.
pub fn build_done_args(uuid: &str) -> Vec<String> {
    vec![uuid.to_string(), "done".to_string()]
}

/// Args for `task <uuid> modify status:pending` (undone). Pure for testing.
pub fn build_undone_args(uuid: &str) -> Vec<String> {
    vec![
        uuid.to_string(),
        "modify".to_string(),
        "status:pending".to_string(),
    ]
}

/// Args for `task rc.confirmation=no undo`. Pure for testing.
pub fn build_undo_args() -> Vec<String> {
    vec!["rc.confirmation=no".to_string(), "undo".to_string()]
}

/// Args for `task <uuid> delete` with the confirmation bypass first.
/// The UI already confirms via its own `y/n` modal, so the subprocess
/// must never prompt (it has no stdin under `output()`).
pub fn build_delete_args(uuid: &str) -> Vec<String> {
    vec![
        "rc.confirmation=no".to_string(),
        uuid.to_string(),
        "delete".to_string(),
    ]
}

/// Args deleting a recurring template or one of its pending occurrences
/// without cascading to the other occurrences. `recurrence.confirmation=no`
/// answers "no" to taskwarrior's "delete all pending recurrences?" prompt;
/// the caller deletes the remaining occurrences explicitly.
pub fn build_recur_delete_args(uuid: &str) -> Vec<String> {
    vec![
        "rc.confirmation=no".to_string(),
        "rc.recurrence.confirmation=no".to_string(),
        uuid.to_string(),
        "delete".to_string(),
    ]
}

/// Args removing `recur` and `parent` from one generated occurrence while
/// keeping every other attribute. Taskwarrior refuses `modify recur:` on a
/// recurring task, but its `edit` path accepts the same change, so a
/// non-interactive `sed` editor strips the two fields from the edit file.
pub fn build_detach_recur_args(uuid: &str) -> Vec<String> {
    vec![
        "rc.verbose=nothing".to_string(),
        "rc.editor=sed -i.bak -e '/^[[:space:]]*Recur:/d' -e '/^[[:space:]]*Parent:/d'".to_string(),
        uuid.to_string(),
        "edit".to_string(),
    ]
}

/// Args for `task <uuid> modify siapsection:<section>`. Pure for testing.
pub fn build_section_args(uuid: &str, section: &str) -> Vec<String> {
    vec![
        uuid.to_string(),
        "modify".to_string(),
        format!("siapsection:{}", section.trim()),
    ]
}

/// Args for `task <uuid> modify project:<project>`. Pure for testing.
/// An empty project clears the field, matching the popup save rule.
pub fn build_project_args(uuid: &str, project: &str) -> Vec<String> {
    vec![
        uuid.to_string(),
        "modify".to_string(),
        format!("project:{}", project.trim()),
    ]
}

/// Args for `task <uuid> modify` with tag diffs only. Pure for testing.
pub fn build_tag_args(uuid: &str, add: &[String], remove: &[String]) -> Vec<String> {
    let mut args = vec![uuid.to_string(), "modify".to_string()];
    for tag in add {
        let tag = tag.trim();
        if !tag.is_empty() {
            args.push(format!("+{tag}"));
        }
    }
    for tag in remove {
        let tag = tag.trim();
        if !tag.is_empty() {
            args.push(format!("-{tag}"));
        }
    }
    args
}

/// Args for native blocking: `task <task> modify depends:<dep>`.
/// Add-if-missing, so other dependencies stay untouched. Pure for testing.
pub fn build_depends_add_args(task_uuid: &str, dep_uuid: &str) -> Vec<String> {
    vec![
        task_uuid.to_string(),
        "modify".to_string(),
        format!("depends:{dep_uuid}"),
    ]
}

/// Args for `task <uuid> annotate <text>`. Pure for testing.
pub fn build_annotate_args(uuid: &str, text: &str) -> Vec<String> {
    vec![uuid.to_string(), "annotate".to_string(), text.to_string()]
}

/// Args for `task <uuid> denotate <text>`. Pure for testing.
/// Taskwarrior matches the annotation by its exact description text.
pub fn build_denotate_args(uuid: &str, text: &str) -> Vec<String> {
    vec![uuid.to_string(), "denotate".to_string(), text.to_string()]
}

/// Args removing one link: `task <task> modify depends:-<dep>`.
/// Removes only that link. Pure for testing.
pub fn build_depends_remove_args(task_uuid: &str, dep_uuid: &str) -> Vec<String> {
    vec![
        task_uuid.to_string(),
        "modify".to_string(),
        format!("depends:-{dep_uuid}"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_taskdata_override_without_env() {
        assert_eq!(
            resolve_data_dir(Some(String::from("/tmp/siap-fixture-data"))),
            PathBuf::from("/tmp/siap-fixture-data")
        );
        let fallback = resolve_data_dir(None);
        assert!(fallback.to_string_lossy().ends_with(".task"));
    }

    #[test]
    fn parses_export_fixture_without_live_data() {
        // Fixture JSON, never the live `task` binary.
        let fixture = r#"[
            {"uuid":"mock-uuid-003","description":"parent","status":"pending","entry":"20260101T000000Z"},
            {"uuid":"mock-uuid-012","description":"child","status":"pending","depends":"mock-uuid-003"}
        ]"#;
        let value: serde_json::Value = serde_json::from_str(fixture).unwrap();
        let rows = value.as_array().unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0]["uuid"], "mock-uuid-003");
        assert_eq!(rows[1]["depends"], "mock-uuid-003");
    }

    #[test]
    fn two_exports_cover_open_and_tail_without_deleted() {
        // Open rows come from `+PENDING`; tail from `+COMPLETED`.
        // `deleted` is never fetched by either export.
        assert_eq!(
            pending_args(),
            vec!["+PENDING".to_string(), "export".to_string()]
        );
        assert_eq!(
            completed_args(),
            vec!["+COMPLETED".to_string(), "export".to_string()]
        );
        for args in [pending_args(), completed_args()] {
            assert!(
                !args.iter().any(|a| a == "deleted"),
                "deleted never fetched"
            );
        }
    }

    #[test]
    fn run_write_captures_stderr_with_isolated_taskdata() {
        // Pin to temp dirs so no live data is touched. All backend tests
        // share this fixed path so parallel sets cannot race.
        let dir = "/tmp/siap-backend-test-isolation";
        std::fs::create_dir_all(dir).unwrap();
        // SAFETY: tests in this process do not rely on live TASKDATA/TASKRC;
        // the fixed temp value keeps every backend test isolated together.
        unsafe {
            std::env::set_var("TASKDATA", dir);
            std::env::set_var("TASKRC", format!("{dir}/taskrc"));
        }
        assert_eq!(data_dir(), PathBuf::from(dir));
        // Invalid subcommand must fail with an explanation, never Ok.
        let err = run_write(&["invalid-command-xyz"]).expect_err("bad command fails");
        assert!(!err.trim().is_empty(), "stderr captured for toast");
    }

    #[test]
    fn section_uda_args_register_non_interactively() {
        let raw = section_uda_args();
        let args: Vec<&str> = raw.iter().map(String::as_str).collect();
        assert_eq!(
            args,
            vec![
                "rc.confirmation=off",
                "config",
                "uda.siapsection.type",
                "string"
            ],
            "bootstrap never waits on a confirmation prompt"
        );
    }

    #[test]
    fn add_args_skip_empty_and_omit_section() {
        let args = build_add_args("hello", "", "", None, &[], &[], "", "");
        assert_eq!(args, vec!["add", "hello"]);
        let args = build_add_args(
            "work item",
            "work",
            "today",
            Some('H'),
            &[String::from("urgent")],
            &[],
            "weekly",
            "",
        );
        assert!(args.contains(&"project:work".to_string()));
        assert!(args.contains(&"due:today".to_string()));
        assert!(args.contains(&"priority:H".to_string()));
        assert!(args.contains(&"+urgent".to_string()));
        assert!(args.contains(&"recur:weekly".to_string()));
        assert!(
            !args.iter().any(|a| a.starts_with("siapsection:")),
            "empty section stamps nothing"
        );
        let args = build_add_args("sectioned", "work", "", None, &[], &[], "", "Next");
        assert!(args.contains(&"siapsection:Next".to_string()));
        // Blocking links ride along as one comma-separated attribute.
        let args = build_add_args(
            "blocked add",
            "",
            "",
            None,
            &[],
            &[String::from("uuid-1"), String::from("uuid-2")],
            "",
            "",
        );
        assert!(args.contains(&"depends:uuid-1,uuid-2".to_string()));
        // Cleared recurrence never writes an attribute.
        let args = build_add_args("plain", "", "", None, &[], &[], "none", "");
        assert!(
            !args.iter().any(|a| a.starts_with("recur:")),
            "none skips recur"
        );
    }

    #[test]
    fn modify_args_clear_empty_and_diff_tags() {
        let args = build_modify_args(
            "uuid-1",
            "new desc",
            "",
            "",
            'M',
            &[String::from("old")],
            &[String::from("new")],
            "Next",
            "",
        );
        assert_eq!(args[0], "uuid-1");
        assert_eq!(args[1], "modify");
        assert!(args.contains(&"description:new desc".to_string()));
        assert!(args.contains(&"project:".to_string()), "empty clears");
        assert!(args.contains(&"due:".to_string()), "empty clears");
        assert!(args.contains(&"+new".to_string()));
        assert!(args.contains(&"-old".to_string()));
        assert!(args.contains(&"siapsection:Next".to_string()));
    }

    #[test]
    fn op_args_use_uuid_keyed_commands() {
        assert_eq!(build_start_args("u1"), vec!["u1", "start"]);
        assert_eq!(build_stop_args("u1"), vec!["u1", "stop"]);
        assert_eq!(build_done_args("u1"), vec!["u1", "done"]);
        assert_eq!(
            build_undone_args("u1"),
            vec!["u1", "modify", "status:pending"]
        );
        assert_eq!(build_undo_args(), vec!["rc.confirmation=no", "undo"]);
        // Delete bypasses the interactive confirm; the UI confirms first.
        assert_eq!(
            build_delete_args("u1"),
            vec!["rc.confirmation=no", "u1", "delete"]
        );
        assert_eq!(
            build_section_args("u1", "Next"),
            vec!["u1", "modify", "siapsection:Next"]
        );
        assert_eq!(
            build_project_args("u1", "work"),
            vec!["u1", "modify", "project:work"]
        );
        assert_eq!(
            build_tag_args("u1", &["urgent".to_string()], &["old".to_string()]),
            vec!["u1", "modify", "+urgent", "-old"]
        );
        // Depends links preserve surrounding values by construction.
        assert_eq!(
            build_depends_add_args("parent", "child"),
            vec!["parent", "modify", "depends:child"]
        );
        assert_eq!(
            build_depends_remove_args("parent", "child"),
            vec!["parent", "modify", "depends:-child"]
        );
        // Annotations add/remove by exact description text.
        assert_eq!(
            build_annotate_args("u1", "call back"),
            vec!["u1", "annotate", "call back"]
        );
        assert_eq!(
            build_denotate_args("u1", "call back"),
            vec!["u1", "denotate", "call back"]
        );
    }

    #[test]
    fn recur_delete_and_detach_args_are_non_interactive() {
        // Deleting one recurring task must never answer "yes" to the
        // "delete all pending recurrences?" prompt.
        assert_eq!(
            build_recur_delete_args("u1"),
            vec![
                "rc.confirmation=no",
                "rc.recurrence.confirmation=no",
                "u1",
                "delete"
            ]
        );
        // Detach rides the edit path because modify refuses `recur:`.
        let detach = build_detach_recur_args("u1");
        assert_eq!(detach[0], "rc.verbose=nothing");
        assert!(detach[1].starts_with("rc.editor=sed"));
        assert!(detach[1].contains("*Recur:"));
        assert!(detach[1].contains("*Parent:"));
        assert_eq!(detach[2], "u1");
        assert_eq!(detach[3], "edit");
    }

    #[test]
    fn round_trip_add_then_modify_reads_back_fixture() {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "siap-rt-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            n
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let rc = dir.join("taskrc");
        // Taskwarrior refuses a missing rc file, so seed a minimal one.
        std::fs::write(
            &rc,
            format!("data.location={}\nconfirmation=no\n", dir.display()),
        )
        .unwrap();
        let run = |args: &[&str]| run_write_with_env(args, Some(&dir), Some(&rc));

        // Bootstrap the section UDA without user action or a prompt.
        run(&[
            "rc.confirmation=off",
            "config",
            "uda.siapsection.type",
            "string",
        ])
        .expect("uda registers");

        // Quick-add a fixture task, then read it back from the pending export.
        let add = build_add_args(
            "round-trip fixture",
            "work",
            "today",
            Some('M'),
            &[String::from("rt")],
            &[],
            "",
            "",
        );
        let refs: Vec<&str> = add.iter().map(String::as_str).collect();
        run(&refs).expect("add succeeds");
        let pending = run(&["+PENDING", "export"]).expect("export succeeds");
        let values: Vec<serde_json::Value> =
            serde_json::from_str(&pending).expect("valid export json");
        let found = values
            .iter()
            .find(|v| v.get("description").and_then(|d| d.as_str()) == Some("round-trip fixture"))
            .expect("fixture reads back")
            .clone();
        let uuid = found
            .get("uuid")
            .and_then(|u| u.as_str())
            .unwrap()
            .to_string();
        assert!(!uuid.is_empty(), "backend assigns a uuid");

        // Popup save via `task <uuid> modify`, then re-export and confirm.
        let old_tags = vec![String::from("rt")];
        let new_tags = vec![String::from("rt"), String::from("saved")];
        let modify = build_modify_args(
            &uuid,
            "round-trip edited",
            "work",
            "today",
            'H',
            &old_tags,
            &new_tags,
            "Next",
            "",
        );
        let refs: Vec<&str> = modify.iter().map(String::as_str).collect();
        run(&refs).expect("modify succeeds");
        let pending = run(&["+PENDING", "export"]).expect("re-export succeeds");
        let values: Vec<serde_json::Value> =
            serde_json::from_str(&pending).expect("valid re-export json");
        let edited = values
            .iter()
            .find(|v| v.get("uuid").and_then(|u| u.as_str()) == Some(uuid.as_str()))
            .expect("edited task reads back");
        assert_eq!(
            edited.get("description").and_then(|d| d.as_str()),
            Some("round-trip edited")
        );
        assert_eq!(
            edited.get("siapsection").and_then(|s| s.as_str()),
            Some("Next"),
            "section persists through modify"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
