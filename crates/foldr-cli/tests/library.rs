use serde_json::Value;
use std::{
    ffi::OsStr,
    fs,
    os::unix::fs::symlink,
    path::{Path, PathBuf},
    process::{Command, Output},
};

fn run(args: &[&OsStr], library: &Path, state: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_foldr"))
        .args(["--json", "--preset-dir"])
        .arg(library)
        .arg("--state-dir")
        .arg(state)
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
    assert!(output.stderr.is_empty());
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["schema_version"], 1);
    value["data"].clone()
}
fn setup() -> (tempfile::TempDir, PathBuf, PathBuf, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = fs::canonicalize(temp.path()).unwrap();
    let library = root.join("library");
    let state = root.join("state");
    (temp, root, library, state)
}

#[test]
fn installed_presets_preserve_partial_binary_removal_semantics_and_explicit_sources() {
    let (_temp, root, library, state) = setup();
    let source = root.join("source.toml");
    fs::write(&source, "schema_version=1\nname='embedded-name'\n[settings]\nremove_note=true\nremove_metadata=['gone']\n[settings.metadata]\nblob=[0,255,10]").unwrap();
    assert_eq!(
        data(&run(
            &["preset".as_ref(), "list".as_ref()],
            &library,
            &state
        )),
        serde_json::json!([])
    );
    assert!(!library.exists());
    data(&run(
        &[
            "preset".as_ref(),
            "install".as_ref(),
            source.as_os_str(),
            "--name".as_ref(),
            "z-last".as_ref(),
        ],
        &library,
        &state,
    ));
    data(&run(
        &[
            "preset".as_ref(),
            "install".as_ref(),
            source.as_os_str(),
            "--name".as_ref(),
            "a-first".as_ref(),
        ],
        &library,
        &state,
    ));
    let entries = data(&run(
        &["preset".as_ref(), "list".as_ref()],
        &library,
        &state,
    ));
    assert_eq!(entries[0]["name"], "a-first");
    assert_eq!(entries[1]["name"], "z-last");
    let original = data(&run(
        &["preset".as_ref(), "show".as_ref(), source.as_os_str()],
        &library,
        &state,
    ));
    assert_eq!(entries[0]["preset"], original);
    // Every named-source positional is a folder, including a folder ending in .toml.
    let target = root.join("folder.toml");
    fs::create_dir(&target).unwrap();
    let named = data(&run(
        &[
            "preset".as_ref(),
            "apply".as_ref(),
            "--name".as_ref(),
            "a-first".as_ref(),
            target.as_os_str(),
            "--dry-run".as_ref(),
        ],
        &library,
        &state,
    ));
    let file = data(&run(
        &[
            "preset".as_ref(),
            "apply".as_ref(),
            source.as_os_str(),
            target.as_os_str(),
            "--dry-run".as_ref(),
        ],
        &library,
        &state,
    ));
    assert_eq!(named, file);
    assert!(!state.exists());
}

#[test]
fn corrupt_duplicate_symlink_and_nonregular_sources_are_refused_without_overwrite() {
    let (_temp, root, library, state) = setup();
    let source = root.join("preset.toml");
    fs::write(&source, "schema_version=1\n[settings]\nmode=448").unwrap();
    for name in ["../escape", "a/b", "a\\b", ".", "a.toml"] {
        let output = run(
            &[
                "preset".as_ref(),
                "install".as_ref(),
                source.as_os_str(),
                "--name".as_ref(),
                name.as_ref(),
            ],
            &library,
            &state,
        );
        assert_eq!(output.status.code(), Some(2));
    }
    assert!(!library.exists());
    let corrupt = root.join("corrupt.toml");
    fs::write(&corrupt, "schema_version=1\n[settings]\nwat=true").unwrap();
    assert_eq!(
        run(
            &[
                "preset".as_ref(),
                "install".as_ref(),
                corrupt.as_os_str(),
                "--name".as_ref(),
                "broken".as_ref()
            ],
            &library,
            &state
        )
        .status
        .code(),
        Some(2)
    );
    assert!(!library.exists());
    data(&run(
        &[
            "preset".as_ref(),
            "install".as_ref(),
            source.as_os_str(),
            "--name".as_ref(),
            "private".as_ref(),
        ],
        &library,
        &state,
    ));
    let saved = fs::read(library.join("private.toml")).unwrap();
    fs::write(&source, "schema_version=1\n[settings]\nmode=493").unwrap();
    assert_eq!(
        run(
            &[
                "preset".as_ref(),
                "install".as_ref(),
                source.as_os_str(),
                "--name".as_ref(),
                "private".as_ref()
            ],
            &library,
            &state
        )
        .status
        .code(),
        Some(4)
    );
    assert_eq!(fs::read(library.join("private.toml")).unwrap(), saved);
    let alias = root.join("alias");
    symlink(&library, &alias).unwrap();
    assert_eq!(
        run(&["preset".as_ref(), "list".as_ref()], &alias, &state)
            .status
            .code(),
        Some(2)
    );
    assert_eq!(
        run(
            &[
                "preset".as_ref(),
                "install".as_ref(),
                source.as_os_str(),
                "--name".as_ref(),
                "new".as_ref()
            ],
            &alias.join("child"),
            &state
        )
        .status
        .code(),
        Some(2)
    );
    assert!(!library.join("child").exists());
    symlink(&source, library.join("linked.toml")).unwrap();
    assert_eq!(
        run(&["preset".as_ref(), "list".as_ref()], &library, &state)
            .status
            .code(),
        Some(2)
    );
    let folder = root.join("folder");
    fs::create_dir(&folder).unwrap();
    assert_eq!(
        run(
            &[
                "preset".as_ref(),
                "apply".as_ref(),
                "--name".as_ref(),
                "linked".as_ref(),
                folder.as_os_str(),
                "--dry-run".as_ref()
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
            &[
                "preset".as_ref(),
                "install".as_ref(),
                folder.as_os_str(),
                "--name".as_ref(),
                "dir".as_ref()
            ],
            &library,
            &state
        )
        .status
        .code(),
        Some(2)
    );
    assert!(!state.exists());
}

#[test]
fn exclusive_install_has_one_winner_even_with_concurrent_processes() {
    let (_temp, root, library, state) = setup();
    let source = root.join("source.toml");
    fs::write(&source, "schema_version=1\n[settings]\nmode=448").unwrap();
    let spawn = || {
        Command::new(env!("CARGO_BIN_EXE_foldr"))
            .arg("--preset-dir")
            .arg(&library)
            .args(["preset", "install"])
            .arg(&source)
            .args(["--name", "same"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap()
    };
    let mut first = spawn();
    let mut second = spawn();
    let mut statuses = [
        first.wait().unwrap().code().unwrap(),
        second.wait().unwrap().code().unwrap(),
    ];
    statuses.sort();
    assert_eq!(statuses, [0, 4]);
    assert_eq!(
        data(&run(
            &["preset".as_ref(), "list".as_ref()],
            &library,
            &state
        ))
        .as_array()
        .unwrap()
        .len(),
        1
    );
}

#[test]
fn platform_defaults_and_explicit_override_do_not_search_current_directory() {
    let (_temp, root, library, state) = setup();
    let home = root.join("home");
    fs::create_dir(&home).unwrap();
    let config = root.join("config");
    fs::create_dir(&config).unwrap();
    let source = root.join("source.toml");
    fs::write(&source, "schema_version=1\n[settings]\nmode=448").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_foldr"))
        .env("HOME", &home)
        .env("XDG_CONFIG_HOME", &config)
        .args(["--json", "preset", "install"])
        .arg(&source)
        .args(["--name", "default"])
        .output()
        .unwrap();
    data(&output);
    let default = if cfg!(target_os = "macos") {
        home.join("Library/Application Support/foldr/presets")
    } else {
        config.join("foldr/presets")
    };
    assert!(default.join("default.toml").exists());
    let output = Command::new(env!("CARGO_BIN_EXE_foldr"))
        .env("HOME", &home)
        .env("XDG_CONFIG_HOME", &config)
        .arg("--json")
        .arg("--preset-dir")
        .arg(&library)
        .args(["preset", "list"])
        .output()
        .unwrap();
    assert_eq!(data(&output), serde_json::json!([]));
    assert!(!library.exists());
    assert!(!state.exists());
}

#[test]
fn library_does_not_apply_automatically_and_preserves_platform_preflight() {
    let (_temp, root, library, state) = setup();
    let source = root.join("source.toml");
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
    let folder = root.join("folder");
    fs::create_dir(&folder).unwrap();
    let before = fs::metadata(&folder).unwrap().permissions();
    data(&run(
        &[
            "preset".as_ref(),
            "install".as_ref(),
            source.as_os_str(),
            "--name".as_ref(),
            "foreign".as_ref(),
        ],
        &library,
        &state,
    ));
    data(&run(
        &["inspect".as_ref(), folder.as_os_str()],
        &library,
        &state,
    ));
    let output = run(
        &[
            "preset".as_ref(),
            "apply".as_ref(),
            "--name".as_ref(),
            "foreign".as_ref(),
            folder.as_os_str(),
        ],
        &library,
        &state,
    );
    assert_eq!(output.status.code(), Some(3));
    use std::os::unix::fs::PermissionsExt;
    assert_eq!(
        fs::metadata(&folder).unwrap().permissions().mode(),
        before.mode()
    );
    assert!(!state.exists());
    fs::write(library.join("bad.toml"), "not a preset").unwrap();
    let output = run(&["preset".as_ref(), "list".as_ref()], &library, &state);
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("bad.toml"));
}
