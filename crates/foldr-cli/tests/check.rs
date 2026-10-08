use serde_json::Value;
use std::{
    ffi::OsStr,
    fs,
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::Path,
    process::{Command, Output},
};
fn run(args: &[&OsStr], library: &Path, state: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_foldr"))
        .arg("--json")
        .arg("--preset-dir")
        .arg(library)
        .arg("--state-dir")
        .arg(state)
        .args(args)
        .output()
        .unwrap()
}
fn data(output: &Output, code: i32) -> Value {
    assert_eq!(
        output.status.code(),
        Some(code),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    if matches!(code, 0 | 6) {
        assert!(output.stderr.is_empty());
    }
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["command"], "preset.check");
    value["data"].clone()
}
fn stamps(path: &Path) -> (u32, i64, i64, i64, i64, i64, i64) {
    let m = fs::metadata(path).unwrap();
    (
        m.mode(),
        m.atime(),
        m.atime_nsec(),
        m.mtime(),
        m.mtime_nsec(),
        m.ctime(),
        m.ctime_nsec(),
    )
}
#[test]
fn partial_binary_removal_checks_match_named_and_file_sources_without_writes() {
    let temp = tempfile::tempdir().unwrap();
    let root = fs::canonicalize(temp.path()).unwrap();
    let folder = root.join("folder.toml");
    fs::create_dir(&folder).unwrap();
    let child = folder.join("child");
    fs::create_dir(&child).unwrap();
    fs::set_permissions(&child, fs::Permissions::from_mode(0o700)).unwrap();
    let setup = root.join("setup-state");
    let plan = foldr_core::plan(
        &folder,
        &foldr_core::ChangeRequest {
            metadata: [
                ("blob".into(), Some(vec![0, 255, 10])),
                ("unrelated".into(), Some(b"untouched".to_vec())),
            ]
            .into(),
            ..Default::default()
        },
        false,
    )
    .unwrap();
    assert!(foldr_core::apply(&plan, &setup).unwrap().succeeded());
    let source = root.join("source.toml");
    fs::write(&source,"schema_version=1\n[settings]\nremove_note=true\nremove_metadata=['gone']\n[settings.metadata]\nblob=[0,255,10]").unwrap();
    let library = root.join("library");
    let state = root.join("no-state");
    let installed = run(
        &[
            "preset".as_ref(),
            "install".as_ref(),
            source.as_os_str(),
            "--name".as_ref(),
            "binary".as_ref(),
        ],
        &library,
        &state,
    );
    assert!(installed.status.success());
    let snapshot = foldr_core::inspect(&folder).unwrap();
    let before = stamps(&folder);
    let child_before = stamps(&child);
    let source_before = fs::read(&source).unwrap();
    let file_args = [
        "preset".as_ref(),
        "check".as_ref(),
        source.as_os_str(),
        folder.as_os_str(),
    ];
    let file = data(&run(&file_args, &library, &state), 0);
    let named = data(
        &run(
            &[
                "preset".as_ref(),
                "check".as_ref(),
                "--name".as_ref(),
                "binary".as_ref(),
                folder.as_os_str(),
            ],
            &library,
            &state,
        ),
        0,
    );
    assert_eq!(file, named);
    assert_eq!(file["targets"][0]["status"], "compliant");
    assert_eq!(file["targets"][0]["differences"], serde_json::json!([]));
    assert_eq!(file, data(&run(&file_args, &library, &state), 0));
    assert_eq!(stamps(&folder), before);
    assert_eq!(stamps(&child), child_before);
    assert_eq!(foldr_core::inspect(&folder).unwrap(), snapshot);
    assert_eq!(fs::read(&source).unwrap(), source_before);
    assert!(!state.exists());
    fs::write(
        &source,
        "schema_version=1\n[settings]\nremove_metadata=['blob']",
    )
    .unwrap();
    let drift = data(&run(&file_args, &library, &state), 6);
    assert_eq!(drift["targets"][0]["status"], "drift");
    assert_eq!(
        drift["targets"][0]["differences"][0]["left"]["value"],
        serde_json::json!([0, 255, 10])
    );
    assert_eq!(
        drift["targets"][0]["differences"][0]["right"]["state"],
        "missing"
    );
    assert_eq!(stamps(&folder), before);
}
#[test]
fn mixed_targets_keep_order_and_operational_failures_outrank_drift() {
    let temp = tempfile::tempdir().unwrap();
    let root = fs::canonicalize(temp.path()).unwrap();
    let matched = root.join("match");
    let drifted = root.join("drift-\n\x1b");
    let absent = root.join("absent");
    fs::create_dir(&matched).unwrap();
    fs::create_dir(&drifted).unwrap();
    fs::set_permissions(&matched, fs::Permissions::from_mode(0o700)).unwrap();
    fs::set_permissions(&drifted, fs::Permissions::from_mode(0o750)).unwrap();
    let source = root.join("source.toml");
    fs::write(&source, "schema_version=1\n[settings]\nmode=448").unwrap();
    let library = root.join("no-library");
    let state = root.join("no-state");
    let mixed = data(
        &run(
            &[
                "preset".as_ref(),
                "check".as_ref(),
                source.as_os_str(),
                matched.as_os_str(),
                drifted.as_os_str(),
            ],
            &library,
            &state,
        ),
        6,
    );
    assert_eq!(mixed["targets"][0]["status"], "compliant");
    assert_eq!(mixed["targets"][1]["status"], "drift");
    let errors = data(
        &run(
            &[
                "preset".as_ref(),
                "check".as_ref(),
                source.as_os_str(),
                drifted.as_os_str(),
                absent.as_os_str(),
                matched.as_os_str(),
            ],
            &library,
            &state,
        ),
        1,
    );
    assert_eq!(errors["targets"][0]["status"], "drift");
    assert_eq!(errors["targets"][1]["status"], "indeterminate");
    assert_eq!(errors["targets"][1]["error"]["code"], 1);
    assert_eq!(errors["targets"][2]["status"], "compliant");
    let human = Command::new(env!("CARGO_BIN_EXE_foldr"))
        .args(["preset", "check"])
        .arg(&source)
        .arg(&drifted)
        .output()
        .unwrap();
    assert_eq!(human.status.code(), Some(6));
    assert!(human.stderr.is_empty());
    assert!(!human.stdout.contains(&0x1b));
    assert!(String::from_utf8_lossy(&human.stdout).contains("drift"));
    assert!(!library.exists());
    assert!(!state.exists());
}
#[test]
fn malformed_and_unsupported_inputs_are_distinct_from_drift_and_no_source_fallback_occurs() {
    let temp = tempfile::tempdir().unwrap();
    let root = fs::canonicalize(temp.path()).unwrap();
    let folder = root.join("folder");
    fs::create_dir(&folder).unwrap();
    let source = root.join("source.toml");
    let library = root.join("library");
    let state = root.join("state");
    fs::write(&source, "schema_version=1\n[settings]\nunknown=true").unwrap();
    let invalid = run(
        &[
            "preset".as_ref(),
            "check".as_ref(),
            source.as_os_str(),
            folder.as_os_str(),
        ],
        &library,
        &state,
    );
    assert_eq!(invalid.status.code(), Some(2));
    assert!(invalid.stdout.is_empty());
    fs::write(&source, "schema_version=42\n[settings]").unwrap();
    assert_eq!(
        run(
            &[
                "preset".as_ref(),
                "check".as_ref(),
                source.as_os_str(),
                folder.as_os_str()
            ],
            &library,
            &state
        )
        .status
        .code(),
        Some(3)
    );
    let other = if cfg!(target_os = "macos") {
        "linux"
    } else {
        "macos"
    };
    fs::write(
        &source,
        format!("schema_version=1\nplatform='{other}'\n[settings]\nmode=448"),
    )
    .unwrap();
    let unsupported = data(
        &run(
            &[
                "preset".as_ref(),
                "check".as_ref(),
                source.as_os_str(),
                folder.as_os_str(),
            ],
            &library,
            &state,
        ),
        3,
    );
    assert_eq!(unsupported["targets"][0]["status"], "unsupported");
    fs::write(&source, "schema_version=1\n[settings]").unwrap();
    assert_eq!(
        run(
            &[
                "preset".as_ref(),
                "check".as_ref(),
                "--name".as_ref(),
                "source".as_ref(),
                folder.as_os_str()
            ],
            &library,
            &state
        )
        .status
        .code(),
        Some(2)
    );
    assert_eq!(
        run(
            &["preset".as_ref(), "check".as_ref(), source.as_os_str()],
            &library,
            &state
        )
        .status
        .code(),
        Some(2)
    );
    assert!(!library.exists());
    assert!(!state.exists());
}

#[test]
fn released_v010_binary_and_removal_presets_are_readable_through_cli() {
    let temp = tempfile::tempdir().unwrap();
    let root = fs::canonicalize(temp.path()).unwrap();
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../foldr-core/tests/fixtures/v0.1.0/preset.toml");
    // Fixture names are discovered from the released-tag capture, not reconstructed schemas.
    let fixtures = fixture.parent().unwrap();
    for entry in fs::read_dir(fixtures).unwrap() {
        let file = entry.unwrap().path();
        if file
            .extension()
            .is_some_and(|extension| extension == "toml")
        {
            let output = run(
                &["preset".as_ref(), "show".as_ref(), file.as_os_str()],
                &root.join("library"),
                &root.join("state"),
            );
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            let value: Value = serde_json::from_slice(&output.stdout).unwrap();
            assert_eq!(value["schema_version"], 1);
            assert_eq!(value["data"]["schema_version"], 1);
        }
    }
}
