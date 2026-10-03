use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// Inspect and configure folders on macOS and Linux.
#[derive(Parser)]
#[command(name = "foldr", version, arg_required_else_help = true)]
#[command(
    after_long_help = "Examples:\n  foldr inspect ./project --json\n  foldr note set ./project 'Active project' --dry-run\n  foldr permissions set ./project 0750\n  foldr preset save ./project --output project.toml --fields note,attrs\n  foldr preset apply project.toml ./one ./two --dry-run\n  foldr undo history\n  foldr undo apply CHANGE_ID --dry-run\n\nScope: folder operations never recurse. Mutating through any symbolic link requires\n--follow-symlink. Presets are partial configurations; omitted fields stay untouched.\nPreset outputs are created exclusively and never overwrite existing files.\n\nExit statuses: 0 success (including closed stdout pipes), 1 operational failure,\n2 invalid input, 3 unsupported operation, 4 conflict, 5 partial change/batch failure.\nSuccessful --json output uses {schema_version, command, data}; diagnostics use\n{schema_version, error} on stderr. Human output escapes controls and uses no color.\nHelp/version, completions and man are text; omit --json for shell documents."
)]
pub struct Cli {
    /// Emit versioned JSON; diagnostics go to stderr.
    #[arg(long, global = true)]
    pub json: bool,
    /// Directory for recovery records (defaults to the platform user state directory).
    #[arg(long, global = true)]
    pub state_dir: Option<PathBuf>,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Inspect a folder without scanning its descendants.
    Inspect { path: PathBuf },
    /// Explain supported properties and operation limitations.
    Doctor { path: PathBuf },
    /// Read or edit the foldr-owned folder note.
    #[command(
        subcommand,
        after_long_help = "Notes use the native com.foldr.note (macOS) or user.foldr.note (Linux) attribute.\nValues are UTF-8 bytes. Attribute size and support limits depend on the filesystem;\nuse doctor PATH to review support and shorten values if the filesystem rejects them.\nCopies, archives, sync tools and Git may not preserve notes. Use preset save with\n--fields note and preset import/export for explicit portable configuration."
    )]
    Note(NoteCommand),
    /// Read or edit a foldr-owned attribute using a suffix key (e.g. project-id).
    #[command(
        subcommand,
        after_long_help = "KEY is a suffix inside com.foldr.* (macOS) or user.foldr.* (Linux). Other native\nattributes are never removed or overwritten. Use --hex for arbitrary binary bytes.\nSize/support limits vary by filesystem; inspect errors and doctor PATH for details.\nCopies, archives, synchronization and Git may lose attributes; preset save with\n--fields attrs, followed by import/export, preserves foldr metadata explicitly."
    )]
    Attr(AttrCommand),
    /// Change native directory flags; flags affect the directory itself.
    #[command(subcommand)]
    Flags(FlagsCommand),
    /// Change Unix permissions: read lists names, execute traverses, write edits entries.
    #[command(subcommand)]
    Permissions(PermissionsCommand),
    /// Save, review, and apply versioned partial TOML presets.
    #[command(subcommand)]
    Preset(PresetCommand),
    /// Review changes and restore recorded fields if they still match.
    #[command(subcommand)]
    Undo(UndoCommand),
    /// Compare folder configuration without writing.
    Diff {
        left: PathBuf,
        right: PathBuf,
        /// Compare the left folder with a TOML preset.
        #[arg(long, conflicts_with = "snapshot")]
        preset: bool,
        /// Compare JSON snapshots previously emitted by inspect --json.
        #[arg(long)]
        snapshot: bool,
    },
    /// Generate shell completions on stdout.
    Completions { shell: clap_complete::Shell },
    /// Generate a roff manual page on stdout.
    Man,
}

#[derive(clap::Args)]
pub struct MutationOptions {
    /// Show planned changes without writing metadata or recovery records.
    #[arg(long)]
    pub dry_run: bool,
    /// Explicitly edit the target when PATH is a symbolic link.
    #[arg(long)]
    pub follow_symlink: bool,
}

#[derive(Subcommand)]
pub enum NoteCommand {
    /// Show the note, including its lossless byte value in JSON.
    Get { path: PathBuf },
    /// Set a UTF-8 note on this folder.
    Set {
        path: PathBuf,
        value: String,
        #[command(flatten)]
        options: MutationOptions,
    },
    /// Remove the note while preserving other attributes.
    Remove {
        path: PathBuf,
        #[command(flatten)]
        options: MutationOptions,
    },
}

#[derive(Subcommand)]
pub enum AttrCommand {
    /// Show one foldr-owned attribute.
    Get { path: PathBuf, key: String },
    /// Set one foldr-owned attribute from text or hexadecimal bytes.
    Set {
        path: PathBuf,
        key: String,
        value: String,
        /// Interpret VALUE as hexadecimal bytes.
        #[arg(long)]
        hex: bool,
        #[command(flatten)]
        options: MutationOptions,
    },
    /// Remove only the named foldr-owned attribute.
    Remove {
        path: PathBuf,
        key: String,
        #[command(flatten)]
        options: MutationOptions,
    },
}

#[derive(Subcommand)]
pub enum FlagsCommand {
    /// Explicitly enable or disable supported native flags.
    Set {
        path: PathBuf,
        /// Hidden in Finder; macOS only.
        #[arg(long, value_name = "BOOL")]
        hidden: Option<bool>,
        /// Protect directory entries; does not freeze existing file contents.
        #[arg(long, value_name = "BOOL")]
        immutable: Option<bool>,
        #[command(flatten)]
        options: MutationOptions,
    },
}

#[derive(Subcommand)]
pub enum PermissionsCommand {
    /// Set basic mode bits for this directory; existing ACLs require review.
    Set {
        path: PathBuf,
        /// Octal Unix mode, e.g. 0750; retain owner read/search (0500) for recovery.
        mode: String,
        #[command(flatten)]
        options: MutationOptions,
    },
}

#[derive(Subcommand)]
pub enum PresetCommand {
    /// Capture only selected settings in a new TOML file.
    Save {
        path: PathBuf,
        #[arg(long)]
        output: PathBuf,
        /// Capture only listed fields: note,attrs,mode,flags.
        #[arg(long, value_delimiter = ',', default_value = "note,attrs,mode")]
        fields: Vec<String>,
    },
    /// Validate and display a TOML preset.
    Show { file: PathBuf },
    /// Apply a partial preset to explicit folders, with independent recovery.
    Apply {
        file: PathBuf,
        /// Explicit folders; no recursion or implicit selection.
        #[arg(required = true, num_args = 1..)]
        paths: Vec<PathBuf>,
        #[command(flatten)]
        options: MutationOptions,
    },
    /// Validate and copy a preset into a new file.
    Import {
        file: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
    /// Validate and copy a preset out to a new file.
    Export {
        file: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
}

#[derive(Subcommand)]
pub enum UndoCommand {
    /// List recorded changes without modifying folders.
    History,
    /// Show one recovery record.
    Show { id: String },
    /// Restore recorded fields only when identity and current values match.
    Apply {
        id: String,
        #[arg(long)]
        dry_run: bool,
    },
}
