# foldr

A Rust CLI for inspecting and configuring folders on macOS and Linux.

See the properties behind a folder's name: permissions, extended attributes,
native flags, ACL summaries, and filesystem capabilities. Edit notes, binary
metadata, supported flags, and basic permissions; save partial presets; preview
changes; and recover recorded edits. Reuse named profiles, check for drift in
scripts, explain directory access rules, and manage native Finder tags on macOS.

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

Install a preset into your user library and reuse it by name:

```sh
foldr preset install inbox.toml --name inbox
foldr preset list
foldr preset apply --name inbox "$foldr_demo_dir" --dry-run
foldr preset check --name inbox "$foldr_demo_dir"
foldr permissions explain "$foldr_demo_dir"
foldr undo history --path "$foldr_demo_dir" --limit 10
```

A compliance check returns `0` when requested settings match and `6` when they
drift. It changes neither folder metadata nor recovery state. Unknown,
unsupported, or inaccessible settings produce errors and cannot count as a
match. With `--name`, all positional operands are target folders; otherwise the
first operand selects a preset file. Named presets are applied only by an explicit
command.

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
| `permissions explain PATH` | Explain mode bits, native ACLs, and inheritance without writing |
| `preset save/show/apply/import/export` | Work with versioned, human-editable TOML presets |
| `preset install FILE --name NAME` / `preset list` | Install and list an explicit user preset library |
| `preset check FILE PATH...` / `preset check --name NAME PATH...` | Report compliance, drift, or indeterminate properties |
| `undo history/show/apply` | Inspect recovery records and restore recorded fields |
| `undo history --path PATH --limit N` | Find a folder's records and summarize their status |
| `tags list/add/remove PATH` | Read and edit Finder tag names on macOS |
| `diff LEFT RIGHT` | Compare folders, JSON snapshots (`--snapshot`), or a folder and preset (`--preset`) |
| `completions SHELL` | Generate shell completions |
| `man` | Generate the manual page |

Use `foldr COMMAND --help` for exact arguments. Mutation commands accept
`--dry-run`; preset application accepts an explicit list of folders and never
selects descendants implicitly. [Example presets](examples/presets) include
project, archive, and private settings.

## Scope and recovery

Presets are patches: omitted settings stay untouched. Note and attribute commands
write only foldr's attributes (`com.foldr.*` on macOS, `user.foldr.*` on Linux).
Native Finder tags use a separate, explicitly authorized field with the same
preview and recovery engine. Foldr preserves unrelated metadata and native flags,
and does not change existing child permissions. A directory's
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

`permissions explain` describes the observed access rules and inheritance. Linux
default ACLs affect newly created entries; macOS has separate allow/deny and
inheritance semantics. The explanation states remaining uncertainty about
effective access rather than guessing results for other identities or security
policies.

Support depends on the filesystem, mount options, and current user. macOS Finder
hiding is a native flag; Linux dot-name hiding requires a rename and is not a
flag toggle. Linux immutable writes can require additional privileges. Inspection
never tests write support by modifying a folder, so some write capabilities are
reported as unknown until an edit is attempted. Unsupported operations explain
their limitation; foldr does not automatically elevate privileges.

On macOS, add or remove a Finder tag from the terminal:

```sh
foldr tags list "$foldr_demo_dir"
foldr tags add "$foldr_demo_dir" Inbox --dry-run
foldr tags add "$foldr_demo_dir" Inbox
foldr tags remove "$foldr_demo_dir" Inbox --dry-run
```

Tag edits preserve existing colors, tag order and unrelated Finder metadata.
Malformed tags, unrecognized color encodings, and ambiguous legacy-only Finder
labels require review. Clearing the last tag while a legacy Finder label remains
is refused. Linux reports native Finder tags as unsupported. Existing preset and
recovery formats remain readable; native Finder fields use an explicit newer
schema so older clients refuse them instead of silently dropping them.

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
Preset checks additionally use `6` for definite drift. Their aggregate errors
take precedence over drift: operational `1`, conflict `4`, unsupported `3`, then
invalid-target `2`. A pure drift result goes to stdout without a diagnostic.

Recovery state defaults to `~/Library/Application Support/foldr/history` on macOS
and `$XDG_STATE_HOME/foldr` or `~/.local/state/foldr` on Linux. Override it with
`--state-dir PATH` or `FOLDR_STATE_DIR`. Keep state outside the folder being edited.
An existing state directory must be private and owned by the current user.

Preset libraries default to `~/Library/Application Support/foldr/presets` on
macOS and `$XDG_CONFIG_HOME/foldr/presets` or `~/.config/foldr/presets` on Linux.
Override this with `--preset-dir PATH`. Library paths refuse symlink components;
use canonical paths for macOS `/tmp` and `/var` aliases. Installing an existing
name refuses to overwrite it. Listing an absent library creates nothing.
Folder-filtered history uses directory identity and summarizes complete, partial
or interrupted records; apply an explicitly selected change ID to recover it.

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
python3 scripts/verify-v020.py target/debug/foldr
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
