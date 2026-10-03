use serde_json::Value;
use std::{
    ffi::{OsStr, OsString},
    fs,
    os::unix::{
        ffi::OsStringExt,
        fs::{PermissionsExt, symlink},
    },
    path::Path,
    process::{Command, Output},
};

fn run(args: &[&OsStr], state: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_foldr"))
        .arg("--state-dir")
        .arg(state)
        .arg("--json")
        .args(args)
        .output()
        .unwrap()
}

fn data(output: &Output) -> Value {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["schema_version"], 1);
    json["data"].clone()
}

#[test]
fn inspection_preserves_non_utf8_paths_and_escapes_human_text() {
    let temp = tempfile::tempdir().unwrap();
    let name = OsString::from_vec(if cfg!(target_os = "linux") {
        b"folder-\xff\n\x1b".to_vec()
    } else {
        b"folder-\n\x1b".to_vec()
    });
    let folder = fs::canonicalize(temp.path()).unwrap().join(name);
    fs::create_dir(&folder).unwrap();
    let state = fs::canonicalize(temp.path()).unwrap().join("state");
    let inspected = data(&run(&["inspect".as_ref(), folder.as_os_str()], &state));
    let bytes: Vec<u8> = serde_json::from_value(inspected["path"]["bytes"].clone()).unwrap();
    use std::os::unix::ffi::OsStrExt;
    assert_eq!(bytes, folder.as_os_str().as_bytes());
    let output = Command::new(env!("CARGO_BIN_EXE_foldr"))
        .arg("inspect")
        .arg(&folder)
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(!output.stdout.contains(&0x1b));
    assert!(!state.exists());
}

#[test]
fn dry_run_does_not_write_and_undo_detects_intervening_changes() {
    let temp = tempfile::tempdir().unwrap();
    let folder = fs::canonicalize(temp.path()).unwrap().join("folder");
    fs::create_dir(&folder).unwrap();
    let state = fs::canonicalize(temp.path()).unwrap().join("state");
    data(&run(
        &[
            "note".as_ref(),
            "set".as_ref(),
            folder.as_os_str(),
            "hello".as_ref(),
            "--dry-run".as_ref(),
        ],
        &state,
    ));
    assert!(!state.exists());
    assert!(
        data(&run(
            &["note".as_ref(), "get".as_ref(), folder.as_os_str()],
            &state
        ))["value"]
            .is_null()
    );
    let first = data(&run(
        &[
            "note".as_ref(),
            "set".as_ref(),
            folder.as_os_str(),
            "hello".as_ref(),
        ],
        &state,
    ));
    data(&run(
        &[
            "note".as_ref(),
            "set".as_ref(),
            folder.as_os_str(),
            "changed".as_ref(),
        ],
        &state,
    ));
    let output = run(
        &[
            "undo".as_ref(),
            "apply".as_ref(),
            first["id"].as_str().unwrap().as_ref(),
        ],
        &state,
    );
    assert_eq!(output.status.code(), Some(4));
    assert!(output.stdout.is_empty());
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stderr).unwrap()["error"]["code"],
        4
    );
    assert_eq!(
        data(&run(
            &["note".as_ref(), "get".as_ref(), folder.as_os_str()],
            &state
        ))["text"],
        "changed"
    );
}

#[test]
fn binary_metadata_survives_and_removal_preserves_other_keys() {
    let temp = tempfile::tempdir().unwrap();
    let folder = fs::canonicalize(temp.path()).unwrap().join("folder");
    fs::create_dir(&folder).unwrap();
    let state = fs::canonicalize(temp.path()).unwrap().join("state");
    for (key, value) in [("binary", "00ff1b"), ("keep", "aabb")] {
        data(&run(
            &[
                "attr".as_ref(),
                "set".as_ref(),
                folder.as_os_str(),
                key.as_ref(),
                value.as_ref(),
                "--hex".as_ref(),
            ],
            &state,
        ));
    }
    assert_eq!(
        data(&run(
            &[
                "attr".as_ref(),
                "get".as_ref(),
                folder.as_os_str(),
                "binary".as_ref()
            ],
            &state
        ))["value"],
        serde_json::json!([0, 255, 27])
    );
    data(&run(
        &[
            "attr".as_ref(),
            "remove".as_ref(),
            folder.as_os_str(),
            "binary".as_ref(),
        ],
        &state,
    ));
    assert_eq!(
        data(&run(
            &[
                "attr".as_ref(),
                "get".as_ref(),
                folder.as_os_str(),
                "keep".as_ref()
            ],
            &state
        ))["value"],
        serde_json::json!([170, 187])
    );
}

#[test]
fn symlink_mutation_requires_explicit_follow_and_mode_changes_do_not_recurse() {
    let temp = tempfile::tempdir().unwrap();
    let folder = fs::canonicalize(temp.path()).unwrap().join("folder");
    fs::create_dir(&folder).unwrap();
    fs::set_permissions(&folder, fs::Permissions::from_mode(0o755)).unwrap();
    let child = folder.join("child");
    fs::create_dir(&child).unwrap();
    fs::set_permissions(&child, fs::Permissions::from_mode(0o755)).unwrap();
    let link = fs::canonicalize(temp.path()).unwrap().join("link");
    symlink(&folder, &link).unwrap();
    let state = fs::canonicalize(temp.path()).unwrap().join("state");
    let rejected = run(
        &[
            "permissions".as_ref(),
            "set".as_ref(),
            link.as_os_str(),
            "0700".as_ref(),
        ],
        &state,
    );
    assert!(!rejected.status.success());
    assert!(!state.exists());
    let record = data(&run(
        &[
            "permissions".as_ref(),
            "set".as_ref(),
            link.as_os_str(),
            "0700".as_ref(),
            "--follow-symlink".as_ref(),
        ],
        &state,
    ));
    assert_eq!(
        fs::metadata(&folder).unwrap().permissions().mode() & 0o7777,
        0o700
    );
    assert_eq!(
        fs::metadata(&child).unwrap().permissions().mode() & 0o7777,
        0o755
    );
    data(&run(
        &[
            "undo".as_ref(),
            "apply".as_ref(),
            record["id"].as_str().unwrap().as_ref(),
        ],
        &state,
    ));
    assert_eq!(
        fs::metadata(&folder).unwrap().permissions().mode() & 0o7777,
        0o755
    );
}

#[test]
fn malformed_inputs_produce_json_only_on_stderr() {
    let temp = tempfile::tempdir().unwrap();
    let state = fs::canonicalize(temp.path()).unwrap().join("state");
    for args in [
        vec!["missing".as_ref()],
        vec![
            "permissions".as_ref(),
            "set".as_ref(),
            temp.path().as_os_str(),
            "888".as_ref(),
        ],
    ] {
        let output = run(&args, &state);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert_eq!(
            serde_json::from_slice::<Value>(&output.stderr).unwrap()["error"]["code"],
            2
        );
    }
    assert!(!state.exists());
}

#[test]
fn broken_pipe_is_a_clean_exit() {
    use std::process::Stdio;
    let mut child = Command::new(env!("CARGO_BIN_EXE_foldr"))
        .args(["--json", "inspect", "."])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    drop(child.stdout.take());
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
}

#[test]
fn presets_preserve_omitted_fields_and_report_independent_batch_failures() {
    let temp = tempfile::tempdir().unwrap();
    let root = fs::canonicalize(temp.path()).unwrap();
    let state = root.join("state");
    let source = root.join("source");
    let target = root.join("target");
    let missing = root.join("missing");
    let preset = root.join("project.toml");
    fs::create_dir(&source).unwrap();
    fs::create_dir(&target).unwrap();
    fs::set_permissions(&target, fs::Permissions::from_mode(0o750)).unwrap();
    data(&run(
        &[
            "note".as_ref(),
            "set".as_ref(),
            source.as_os_str(),
            "project".as_ref(),
        ],
        &state,
    ));
    data(&run(
        &[
            "attr".as_ref(),
            "set".as_ref(),
            target.as_os_str(),
            "keep".as_ref(),
            "present".as_ref(),
        ],
        &state,
    ));
    data(&run(
        &[
            "preset".as_ref(),
            "save".as_ref(),
            source.as_os_str(),
            "--output".as_ref(),
            preset.as_os_str(),
            "--fields".as_ref(),
            "note".as_ref(),
        ],
        &state,
    ));
    let history_before = data(&run(&["undo".as_ref(), "history".as_ref()], &state))
        .as_array()
        .unwrap()
        .len();
    data(&run(
        &[
            "preset".as_ref(),
            "apply".as_ref(),
            preset.as_os_str(),
            target.as_os_str(),
            source.as_os_str(),
            "--dry-run".as_ref(),
        ],
        &state,
    ));
    assert_eq!(
        data(&run(&["undo".as_ref(), "history".as_ref()], &state))
            .as_array()
            .unwrap()
            .len(),
        history_before
    );
    assert!(
        data(&run(
            &["note".as_ref(), "get".as_ref(), target.as_os_str()],
            &state
        ))["value"]
            .is_null()
    );
    let output = run(
        &[
            "preset".as_ref(),
            "apply".as_ref(),
            preset.as_os_str(),
            target.as_os_str(),
            missing.as_os_str(),
        ],
        &state,
    );
    assert_eq!(output.status.code(), Some(5));
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    let targets = result["data"]["targets"].as_array().unwrap();
    assert!(targets[0]["record"]["completed"].as_bool().unwrap());
    assert!(targets[1]["error"].is_string());
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stderr).unwrap()["error"]["code"],
        5
    );
    assert_eq!(
        data(&run(
            &["note".as_ref(), "get".as_ref(), target.as_os_str()],
            &state
        ))["text"],
        "project"
    );
    assert_eq!(
        data(&run(
            &[
                "attr".as_ref(),
                "get".as_ref(),
                target.as_os_str(),
                "keep".as_ref()
            ],
            &state
        ))["text"],
        "present"
    );
    assert_eq!(
        fs::metadata(&target).unwrap().permissions().mode() & 0o7777,
        0o750
    );
    let exported = root.join("export.toml");
    let imported = root.join("import.toml");
    data(&run(
        &[
            "preset".as_ref(),
            "export".as_ref(),
            preset.as_os_str(),
            "--output".as_ref(),
            exported.as_os_str(),
        ],
        &state,
    ));
    data(&run(
        &[
            "preset".as_ref(),
            "import".as_ref(),
            exported.as_os_str(),
            "--output".as_ref(),
            imported.as_os_str(),
        ],
        &state,
    ));
    assert_eq!(
        fs::read_to_string(preset).unwrap(),
        fs::read_to_string(imported).unwrap()
    );
}

#[test]
fn completions_and_man_are_real_documents_and_reject_json() {
    let temp = tempfile::tempdir().unwrap();
    let state = temp.path().join("state");
    for args in [vec!["completions", "bash"], vec!["man"]] {
        let output = Command::new(env!("CARGO_BIN_EXE_foldr"))
            .args(&args)
            .output()
            .unwrap();
        assert!(output.status.success());
        assert!(output.stdout.len() > 100);
        assert!(output.stderr.is_empty());
        let args = args.iter().map(OsStr::new).collect::<Vec<_>>();
        let rejected = run(&args, &state);
        assert_eq!(rejected.status.code(), Some(2));
        assert!(rejected.stdout.is_empty());
        assert!(serde_json::from_slice::<Value>(&rejected.stderr).is_ok());
    }
}

#[test]
fn comparisons_are_read_only_and_work_with_snapshots_and_presets() {
    let temp = tempfile::tempdir().unwrap();
    let root = fs::canonicalize(temp.path()).unwrap();
    let left = root.join("left");
    let right = root.join("right");
    let state = root.join("state");
    fs::create_dir(&left).unwrap();
    fs::create_dir(&right).unwrap();
    fs::set_permissions(&left, fs::Permissions::from_mode(0o750)).unwrap();
    fs::set_permissions(&right, fs::Permissions::from_mode(0o700)).unwrap();
    let left_json = root.join("left.json");
    let right_json = root.join("right.json");
    fs::write(
        &left_json,
        run(&["inspect".as_ref(), left.as_os_str()], &state).stdout,
    )
    .unwrap();
    fs::write(
        &right_json,
        run(&["inspect".as_ref(), right.as_os_str()], &state).stdout,
    )
    .unwrap();
    let live = data(&run(
        &["diff".as_ref(), left.as_os_str(), right.as_os_str()],
        &state,
    ));
    let recorded = data(&run(
        &[
            "diff".as_ref(),
            left_json.as_os_str(),
            right_json.as_os_str(),
            "--snapshot".as_ref(),
        ],
        &state,
    ));
    assert_eq!(live["differences"], recorded["differences"]);
    assert!(
        live["differences"]
            .as_array()
            .unwrap()
            .iter()
            .any(|difference| difference["field"] == "mode")
    );
    let preset = root.join("permissions.toml");
    data(&run(
        &[
            "preset".as_ref(),
            "save".as_ref(),
            right.as_os_str(),
            "--output".as_ref(),
            preset.as_os_str(),
            "--fields".as_ref(),
            "mode".as_ref(),
        ],
        &state,
    ));
    let drift = data(&run(
        &[
            "diff".as_ref(),
            left.as_os_str(),
            preset.as_os_str(),
            "--preset".as_ref(),
        ],
        &state,
    ));
    assert_eq!(drift["differences"].as_array().unwrap().len(), 1);
    assert_eq!(drift["differences"][0]["field"], "mode");
    assert!(!state.exists());
    assert_eq!(
        fs::metadata(&left).unwrap().permissions().mode() & 0o7777,
        0o750
    );
}

#[test]
fn native_hidden_flag_is_recoverable_or_explicitly_unsupported() {
    let temp = tempfile::tempdir().unwrap();
    let root = fs::canonicalize(temp.path()).unwrap();
    let folder = root.join("folder");
    let state = root.join("state");
    fs::create_dir(&folder).unwrap();
    let before = data(&run(&["inspect".as_ref(), folder.as_os_str()], &state));
    let output = run(
        &[
            "flags".as_ref(),
            "set".as_ref(),
            folder.as_os_str(),
            "--hidden".as_ref(),
            "true".as_ref(),
            "--dry-run".as_ref(),
        ],
        &state,
    );
    if cfg!(target_os = "linux") {
        assert_eq!(output.status.code(), Some(3));
        assert!(output.stdout.is_empty());
        assert!(!state.exists());
        return;
    }
    data(&output);
    assert!(!state.exists());
    let record = data(&run(
        &[
            "flags".as_ref(),
            "set".as_ref(),
            folder.as_os_str(),
            "--hidden".as_ref(),
            "true".as_ref(),
        ],
        &state,
    ));
    let hidden = data(&run(&["inspect".as_ref(), folder.as_os_str()], &state));
    assert_ne!(before["flags"], hidden["flags"]);
    data(&run(
        &[
            "undo".as_ref(),
            "apply".as_ref(),
            record["id"].as_str().unwrap().as_ref(),
        ],
        &state,
    ));
    assert_eq!(
        data(&run(&["inspect".as_ref(), folder.as_os_str()], &state))["flags"],
        before["flags"]
    );
}
