use serde_json::Value;
use std::{
    ffi::OsStr,
    fs,
    path::Path,
    process::{Command, Output},
};
fn run(args: &[&OsStr], state: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_foldr"))
        .arg("--json")
        .arg("--state-dir")
        .arg(state)
        .args(args)
        .output()
        .unwrap()
}
fn envelope(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap()
}

#[cfg(target_os = "macos")]
#[test]
fn finder_tags_cli_preserves_native_colors_foreign_metadata_and_byte_exact_undo() {
    use foldr_core::platform::{self, tags};
    use std::{
        fs::File,
        os::unix::fs::{MetadataExt, symlink},
    };
    let temp = tempfile::tempdir().unwrap();
    let root = fs::canonicalize(temp.path()).unwrap();
    let folder = root.join("folder");
    fs::create_dir(&folder).unwrap();
    let file = File::open(&folder).unwrap();
    let original=b"<?xml version='1.0'?><plist version='1.0'><array><string>Blue\n4</string><string>Plain</string></array></plist>";
    platform::write_xattr(&file, tags::TAG_XATTR, Some(original)).unwrap();
    let mut finder_info = [0u8; 32];
    finder_info[31] = 23;
    platform::write_xattr(&file, tags::FINDER_INFO, Some(&finder_info)).unwrap();
    platform::write_xattr(&file, b"com.example.foreign", Some(b"keep")).unwrap();
    let state = root.join("state");
    let before = fs::metadata(&folder).unwrap();
    let listed = run(
        &["tags".as_ref(), "list".as_ref(), folder.as_os_str()],
        &state,
    );
    assert!(listed.status.success());
    assert_eq!(envelope(&listed)["data"]["tags"]["value"][0]["color"], 4);
    let dry = run(
        &[
            "tags".as_ref(),
            "add".as_ref(),
            folder.as_os_str(),
            "New".as_ref(),
            "--dry-run".as_ref(),
        ],
        &state,
    );
    assert!(
        dry.status.success(),
        "{}",
        String::from_utf8_lossy(&dry.stderr)
    );
    assert_eq!(envelope(&dry)["data"]["schema_version"], 2);
    assert_eq!(
        envelope(&dry)["data"]["changes"][0]["field"]["kind"],
        "finder_tags"
    );
    assert_eq!(
        tags::read_finder_tags_raw(&file).unwrap().unwrap(),
        original
    );
    assert!(!state.exists());
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
    let add = run(
        &[
            "tags".as_ref(),
            "add".as_ref(),
            folder.as_os_str(),
            "New".as_ref(),
        ],
        &state,
    );
    assert!(
        add.status.success(),
        "{}",
        String::from_utf8_lossy(&add.stderr)
    );
    let add = envelope(&add)["data"].clone();
    assert_eq!(add["schema_version"], 2);
    let added_raw = tags::read_finder_tags_raw(&file).unwrap().unwrap();
    let native = tags::read_finder_tags(&file).unwrap();
    assert_eq!(
        native
            .iter()
            .map(|tag| (&*tag.name, tag.color))
            .collect::<Vec<_>>(),
        vec![("Blue", Some(4)), ("Plain", None), ("New", Some(0))]
    );
    assert_eq!(
        platform::read_xattr(&file, tags::FINDER_INFO)
            .unwrap()
            .unwrap(),
        finder_info
    );
    assert_eq!(
        platform::read_xattr(&file, b"com.example.foreign")
            .unwrap()
            .unwrap(),
        b"keep"
    );
    let source = root.join("tags.toml");
    let saved = run(
        &[
            "preset".as_ref(),
            "save".as_ref(),
            folder.as_os_str(),
            "--output".as_ref(),
            source.as_os_str(),
            "--fields".as_ref(),
            "finder_tags".as_ref(),
        ],
        &state,
    );
    assert!(
        saved.status.success(),
        "{}",
        String::from_utf8_lossy(&saved.stderr)
    );
    assert!(
        fs::read_to_string(&source)
            .unwrap()
            .contains("schema_version = 2")
    );
    let check = run(
        &[
            "preset".as_ref(),
            "check".as_ref(),
            source.as_os_str(),
            folder.as_os_str(),
        ],
        &state,
    );
    assert!(check.status.success());
    let remove = run(
        &[
            "tags".as_ref(),
            "remove".as_ref(),
            folder.as_os_str(),
            "Blue".as_ref(),
        ],
        &state,
    );
    assert!(remove.status.success());
    assert_eq!(
        tags::read_finder_tags(&file)
            .unwrap()
            .iter()
            .map(|tag| &*tag.name)
            .collect::<Vec<_>>(),
        vec!["Plain", "New"]
    );
    let id = envelope(&remove)["data"]["id"].as_str().unwrap().to_owned();
    assert!(
        run(&["undo".as_ref(), "apply".as_ref(), id.as_ref()], &state)
            .status
            .success()
    );
    assert_eq!(
        tags::read_finder_tags_raw(&file).unwrap().unwrap(),
        added_raw
    );
    let id = add["id"].as_str().unwrap();
    assert!(
        run(&["undo".as_ref(), "apply".as_ref(), id.as_ref()], &state)
            .status
            .success()
    );
    assert_eq!(
        tags::read_finder_tags_raw(&file).unwrap().unwrap(),
        original
    );
    let alias = root.join("alias");
    symlink(&folder, &alias).unwrap();
    assert_eq!(
        run(
            &[
                "tags".as_ref(),
                "add".as_ref(),
                alias.as_os_str(),
                "New".as_ref(),
                "--dry-run".as_ref()
            ],
            &state
        )
        .status
        .code(),
        Some(2)
    );
    assert!(
        run(
            &[
                "tags".as_ref(),
                "add".as_ref(),
                alias.as_os_str(),
                "New".as_ref(),
                "--follow-symlink".as_ref(),
                "--dry-run".as_ref()
            ],
            &state
        )
        .status
        .success()
    );
    let removed_all = run(
        &[
            "tags".as_ref(),
            "remove".as_ref(),
            folder.as_os_str(),
            "Blue".as_ref(),
            "Plain".as_ref(),
        ],
        &state,
    );
    assert!(removed_all.status.success());
    assert_eq!(tags::read_finder_tags_raw(&file).unwrap(), None);
    assert_eq!(
        platform::read_xattr(&file, tags::FINDER_INFO)
            .unwrap()
            .unwrap(),
        finder_info
    );
}

#[cfg(target_os = "macos")]
#[test]
fn malformed_and_legacy_native_tags_are_not_reported_compliant_or_changed() {
    use foldr_core::platform::{self, tags};
    use std::fs::File;
    let temp = tempfile::tempdir().unwrap();
    let root = fs::canonicalize(temp.path()).unwrap();
    let folder = root.join("folder");
    fs::create_dir(&folder).unwrap();
    let file = File::open(&folder).unwrap();
    let state = root.join("state");
    let source = root.join("tags.toml");
    fs::write(
        &source,
        "schema_version=2\nplatform='macos'\n[settings]\nfinder_tags=[]",
    )
    .unwrap();
    platform::write_xattr(&file, tags::TAG_XATTR, Some(b"broken-plist")).unwrap();
    let check = run(
        &[
            "preset".as_ref(),
            "check".as_ref(),
            source.as_os_str(),
            folder.as_os_str(),
        ],
        &state,
    );
    assert_eq!(check.status.code(), Some(1));
    assert_eq!(
        envelope(&check)["data"]["targets"][0]["status"],
        "indeterminate"
    );
    assert_eq!(
        run(
            &[
                "tags".as_ref(),
                "add".as_ref(),
                folder.as_os_str(),
                "New".as_ref()
            ],
            &state
        )
        .status
        .code(),
        Some(2)
    );
    assert_eq!(
        tags::read_finder_tags_raw(&file).unwrap().unwrap(),
        b"broken-plist"
    );
    platform::write_xattr(&file, tags::TAG_XATTR, None).unwrap();
    let mut info = [0u8; 32];
    info[9] = 2;
    platform::write_xattr(&file, tags::FINDER_INFO, Some(&info)).unwrap();
    let listed = run(
        &["tags".as_ref(), "list".as_ref(), folder.as_os_str()],
        &state,
    );
    assert_eq!(listed.status.code(), Some(1));
    assert_eq!(envelope(&listed)["data"]["tags"]["state"], "unknown");
    assert_eq!(
        run(
            &[
                "tags".as_ref(),
                "add".as_ref(),
                folder.as_os_str(),
                "New".as_ref()
            ],
            &state
        )
        .status
        .code(),
        Some(2)
    );
    assert_eq!(
        platform::read_xattr(&file, tags::FINDER_INFO)
            .unwrap()
            .unwrap(),
        info
    );
    assert!(!state.exists());
}

#[cfg(target_os = "linux")]
#[test]
fn linux_native_tags_are_explicitly_unsupported_without_writes() {
    let temp = tempfile::tempdir().unwrap();
    let root = fs::canonicalize(temp.path()).unwrap();
    let folder = root.join("folder");
    fs::create_dir(&folder).unwrap();
    let state = root.join("state");
    for args in [
        vec!["tags".as_ref(), "list".as_ref(), folder.as_os_str()],
        vec![
            "tags".as_ref(),
            "add".as_ref(),
            folder.as_os_str(),
            "New".as_ref(),
        ],
        vec![
            "tags".as_ref(),
            "remove".as_ref(),
            folder.as_os_str(),
            "Old".as_ref(),
            "--dry-run".as_ref(),
        ],
    ] {
        let output = run(&args, &state);
        assert_eq!(output.status.code(), Some(3));
    }
    let listed = run(
        &["tags".as_ref(), "list".as_ref(), folder.as_os_str()],
        &state,
    );
    assert_eq!(envelope(&listed)["data"]["tags"]["state"], "unsupported");
    assert!(!state.exists());
}

#[test]
fn generated_documents_include_new_grammar_and_library_option() {
    for args in [
        vec!["--help"],
        vec!["preset", "apply", "--help"],
        vec!["preset", "check", "--help"],
        vec!["tags", "add", "--help"],
        vec!["undo", "history", "--help"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_foldr"))
            .args(&args)
            .output()
            .unwrap();
        assert!(output.status.success());
        let help = String::from_utf8(output.stdout).unwrap();
        assert!(help.contains("--preset-dir"));
    }
    let completion = Command::new(env!("CARGO_BIN_EXE_foldr"))
        .args(["completions", "bash"])
        .output()
        .unwrap();
    assert!(completion.status.success());
    let text = String::from_utf8(completion.stdout).unwrap();
    for word in ["tags", "check", "install", "preset-dir", "limit"] {
        assert!(text.contains(word));
    }
    let man = Command::new(env!("CARGO_BIN_EXE_foldr"))
        .arg("man")
        .output()
        .unwrap();
    assert!(man.status.success());
    assert!(
        String::from_utf8(man.stdout)
            .unwrap()
            .replace("\\-", "-")
            .contains("preset-dir")
    );
}
