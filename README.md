# foldr

A Rust CLI for inspecting and configuring folders on macOS and Linux.

See the properties behind a folder's name: permissions, extended attributes,
native flags, ACL summaries, and filesystem capabilities. Edit notes, binary
metadata, supported flags, and basic permissions; save partial presets; preview
changes; and recover recorded edits.

## Try it

Install from a checkout with Rust 1.85 or newer:

```sh
cargo install --path crates/foldr-cli --locked
foldr --help
```

Create a disposable folder for the walkthrough. Resolve its path because macOS
commonly makes `/tmp` and `/var` symbolic links:

```sh
foldr_demo_dir=$(mktemp -d)
foldr_demo_dir=$(cd "$foldr_demo_dir" && pwd -P)

foldr inspect "$foldr_demo_dir"
foldr doctor "$foldr_demo_dir"
foldr note set "$foldr_demo_dir" "An inbox for incoming documents" --dry-run
foldr note set "$foldr_demo_dir" "An inbox for incoming documents"
foldr note get "$foldr_demo_dir"
foldr attr set "$foldr_demo_dir" category inbox
foldr permissions set "$foldr_demo_dir" 0700 --dry-run
```

Save selected fields and preview a preset on another folder:

```sh
foldr preset save "$foldr_demo_dir" --output inbox.toml --fields note,attrs
foldr preset show inbox.toml
foldr preset apply inbox.toml /absolute/path/to/another-folder --dry-run
```

Review recorded edits before undoing one:

```sh
foldr undo history
foldr undo show CHANGE_ID
foldr undo apply CHANGE_ID --dry-run
foldr undo apply CHANGE_ID
```

## Commands

| Command | Purpose |
| --- | --- |
| `inspect PATH` | Read identity, modes, filesystem, ACLs, flags, attributes, and capabilities |
| `doctor PATH` | Explain support and limitations for this folder |
| `note get/set/remove PATH` | Read and edit the folder note |
| `attr get/set/remove PATH KEY` | Read and edit metadata in foldr's namespace; `set --hex` accepts binary values |
| `flags set PATH` | Set native flags using `--hidden true/false` or `--immutable true/false` |
| `permissions set PATH OCTAL` | Change the selected directory's mode |
| `preset save/show/apply/import/export` | Work with versioned, human-editable TOML presets |
| `undo history/show/apply` | Inspect recovery records and restore recorded fields |
| `diff LEFT RIGHT` | Compare folders, JSON snapshots (`--snapshot`), or a folder and preset (`--preset`) |
| `completions SHELL` | Generate shell completions |
| `man` | Generate the manual page |

Use `foldr COMMAND --help` for exact arguments. Mutation commands accept
`--dry-run`; preset application accepts an explicit list of folders and never
selects descendants implicitly. [Example presets](examples/presets) include
project, archive, and private settings.

## Scope and recovery

Presets are patches: omitted settings stay untouched. Foldr writes only its own
attributes (`com.foldr.*` on macOS, `user.foldr.*` on Linux), preserves unrelated
native flags, and does not change existing child permissions. A directory's
immutable flag protects its entries; existing files can remain editable.

Inspection follows directory symlinks and reports a target when the final path is
a link. Mutations refuse symlink components unless `--follow-symlink` is explicitly
provided. Native operations use an open directory handle; applying a plan checks
that the directory identity and observed fields still match.

Changes are verified and recorded before and after writes in a private recovery
directory. A group of filesystem writes is not atomic: partial failure records
which fields were applied. Undo checks that current values match the recorded
post-change values before restoring recorded fields. Unrelated metadata stays
untouched. Native filesystem calls do not provide a transaction or compare-and-swap
across metadata fields; another writer can race a check and write. Directory
identities also have limits across inode reuse and mount changes.

Basic permission editing requires owner read and search bits to remain enabled
(`0500`), so recovery can reopen the directory. It refuses changes when an
extended ACL needs review or the ACL cannot be read. Rich ACL editing is tracked
as future work.

On Linux, a directory's setgid bit gives newly created entries its group ownership.
The sticky bit restricts unprivileged deletion and renaming to the entry's owner
or the directory's owner. Neither bit rewrites existing or moved-in entries'
ownership or modes.

Support depends on the filesystem, mount options, and current user. macOS Finder
hiding is a native flag; Linux dot-name hiding requires a rename and is not a
flag toggle. Linux immutable writes can require additional privileges. Inspection
never tests write support by modifying a folder, so some write capabilities are
reported as unknown until an edit is attempted. Unsupported operations explain
their limitation; foldr does not automatically elevate privileges.

## Output and state

Add `--json` for versioned machine output:

```sh
foldr inspect ./some-folder --json
```

Successful data uses `{ "schema_version": 1, "command": "...", "data": ... }`.
Diagnostics go to stderr. JSON preserves Unix path bytes and binary attribute
values as byte arrays; human output escapes terminal control characters. A closed
downstream pipe exits cleanly. Help, version, completions, and man output are text
documents; completions and man reject `--json`.

Exit codes: `0` success, `1` operational failure, `2` invalid input, `3` unsupported
operation, `4` conflict, `5` partial operation or failed batch target. Batch output
includes per-target results; other targets can succeed when one target fails.

Recovery state defaults to `~/Library/Application Support/foldr/history` on macOS
and `$XDG_STATE_HOME/foldr` or `~/.local/state/foldr` on Linux. Override it with
`--state-dir PATH` or `FOLDR_STATE_DIR`. Keep state outside the folder being edited.
An existing state directory must be private and owned by the current user.

Folder metadata may be lost in Git, archives, cross-filesystem copies, or sync
tools. Preset import/export is explicit; it uses TOML and preserves binary values.

## Development and packages

The workspace separates `foldr-core` (models, native adapters, changes, presets,
recovery) from `foldr-cli` (commands and output). The development toolchain tracks
stable Rust; CI checks stable and the Rust 1.85 minimum on macOS and Linux.

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --workspace --locked
python3 scripts/verify-cli.py target/debug/foldr
```

The Packages workflow creates development archives for Apple Silicon macOS,
Intel macOS, and x86_64 Linux. The Linux build targets glibc 2.35 or newer; macOS
packages target macOS 13 or newer. Archives include the binary, license, README,
manual, bash/zsh/fish completions, and a SHA256 checksum. Download them from the
workflow's artifacts. These are unsigned development builds.

Build a package on a native supported host:

```sh
bash scripts/package.sh aarch64-apple-darwin
```

Use the matching target for Intel macOS (`x86_64-apple-darwin`) or Linux
(`x86_64-unknown-linux-gnu`). The package script executes the resulting binary, so
cross-compiling it alone does not establish that it runs on the target system.

## Project and contributing

The [roadmap](ROADMAP.md) is generated from the repository's
[Cairn items](cairn/items). The
[concept and architecture](cairn/items/0005-record-cli-first-concept-and-rust-architecture.md)
describe the product direction. Foldr is CLI first; a GUI, TUI, watcher service,
and advanced filesystem integrations remain optional exploratory work.

See [CONTRIBUTING.md](CONTRIBUTING.md). Licensed under [MIT](LICENSE).
