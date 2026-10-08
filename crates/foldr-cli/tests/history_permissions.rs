use serde_json::Value;
use std::{
    ffi::OsStr,
    fs,
    os::unix::{
        ffi::OsStringExt,
        fs::{MetadataExt, PermissionsExt, symlink},
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
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["schema_version"], 1);
    value["data"].clone()
}
#[test]
fn filtered_history_preserves_legacy_records_handles_renames_and_hides_contents_in_human_output() {
    let temp = tempfile::tempdir().unwrap();
    let root = fs::canonicalize(temp.path()).unwrap();
    let folder = root.join(std::ffi::OsString::from_vec(if cfg!(target_os = "linux") {
        b"folder-\xff\n".to_vec()
    } else {
        b"folder-\n".to_vec()
    }));
    fs::create_dir(&folder).unwrap();
    let state = root.join("state");
    assert_eq!(
        data(&run(
            &[
                "undo".as_ref(),
                "history".as_ref(),
                "--path".as_ref(),
                folder.as_os_str(),
                "--limit".as_ref(),
                "1".as_ref()
            ],
            &state
        )),
        serde_json::json!([])
    );
    assert!(!state.exists());
    let changed = data(&run(
        &[
            "note".as_ref(),
            "set".as_ref(),
            folder.as_os_str(),
            "sensitive-secret".as_ref(),
        ],
        &state,
    ));
    let id = changed["id"].as_str().unwrap();
    let recorded = fs::read(state.join(format!("{id}.json"))).unwrap();
    let legacy = data(&run(&["undo".as_ref(), "history".as_ref()], &state));
    assert_eq!(legacy[0], changed);
    assert_eq!(
        data(&run(
            &["undo".as_ref(), "show".as_ref(), id.as_ref()],
            &state
        )),
        changed
    );
    let renamed = root.join("renamed");
    fs::rename(&folder, &renamed).unwrap();
    fs::create_dir(&folder).unwrap();
    assert_eq!(
        data(&run(
            &[
                "undo".as_ref(),
                "history".as_ref(),
                "--path".as_ref(),
                folder.as_os_str()
            ],
            &state
        )),
        serde_json::json!([])
    );
    let alias = root.join("alias");
    symlink(&renamed, &alias).unwrap();
    let filtered = data(&run(
        &[
            "undo".as_ref(),
            "history".as_ref(),
            "--path".as_ref(),
            alias.as_os_str(),
            "--limit".as_ref(),
            "1".as_ref(),
        ],
        &state,
    ));
    assert_eq!(filtered[0]["record"], changed);
    assert_eq!(filtered[0]["status"], "complete");
    assert_eq!(filtered[0]["renamed"], true);
    assert_eq!(
        fs::read(state.join(format!("{id}.json"))).unwrap(),
        recorded
    );
    for args in [
        vec!["undo", "history"],
        vec!["undo", "history", "--limit", "1"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_foldr"))
            .arg("--state-dir")
            .arg(&state)
            .args(args)
            .output()
            .unwrap();
        assert!(output.status.success());
        let human = String::from_utf8(output.stdout).unwrap();
        assert!(!human.contains("sensitive-secret"));
        assert!(human.contains("complete"));
        assert!(human.contains(&format!("foldr undo show {id}")));
    }
    assert_eq!(
        run(
            &[
                "undo".as_ref(),
                "history".as_ref(),
                "--limit".as_ref(),
                "0".as_ref()
            ],
            &state
        )
        .status
        .code(),
        Some(2)
    );
    fs::write(state.join("broken.json"), "not-json").unwrap();
    let output = run(
        &[
            "undo".as_ref(),
            "history".as_ref(),
            "--limit".as_ref(),
            "1".as_ref(),
        ],
        &state,
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("broken.json"));
}
#[test]
fn permissions_explanation_is_versioned_read_only_and_escapes_controls() {
    let temp = tempfile::tempdir().unwrap();
    let root = fs::canonicalize(temp.path()).unwrap();
    let folder = root.join("folder-\n\x1b");
    fs::create_dir(&folder).unwrap();
    fs::set_permissions(&folder, fs::Permissions::from_mode(0o2750)).unwrap();
    let state = root.join("state");
    let before = fs::metadata(&folder).unwrap();
    let explanation = data(&run(
        &[
            "permissions".as_ref(),
            "explain".as_ref(),
            folder.as_os_str(),
        ],
        &state,
    ));
    assert_eq!(explanation["schema_version"], 1);
    assert_eq!(explanation["setgid"], before.mode() & 0o2000 != 0);
    assert_eq!(explanation["mode_classes"][0]["search"], true);
    assert_eq!(explanation["mode_classes"][2]["modify_entries"], false);
    assert!(!explanation["limitations"].as_array().unwrap().is_empty());
    let output = Command::new(env!("CARGO_BIN_EXE_foldr"))
        .args(["permissions", "explain"])
        .arg(&folder)
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(!output.stdout.contains(&0x1b));
    let after = fs::metadata(&folder).unwrap();
    assert_eq!(
        (
            before.mode(),
            before.mtime(),
            before.mtime_nsec(),
            before.ctime(),
            before.ctime_nsec()
        ),
        (
            after.mode(),
            after.mtime(),
            after.mtime_nsec(),
            after.ctime(),
            after.ctime_nsec()
        )
    );
    assert!(!state.exists());
}
