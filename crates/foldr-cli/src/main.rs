mod args;
mod compare;
mod library;
mod mutations;
mod output;
mod presets;

use args::{Cli, Command};
use clap::{CommandFactory, Parser};
use output::{CliError, emit};
use std::{
    io::{self, Write},
    process::ExitCode,
};

fn main() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => {
            if matches!(
                error.kind(),
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
            ) {
                let _ = error.print();
                return ExitCode::SUCCESS;
            }
            let json = std::env::args_os().any(|argument| argument == "--json");
            if json {
                output::diagnostic(&CliError::usage(error.to_string()), true);
            } else {
                let _ = error.print();
            }
            return ExitCode::from(2);
        }
    };
    match run(&cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            output::diagnostic(&error, cli.json);
            ExitCode::from(error.code)
        }
    }
}

fn run(cli: &Cli) -> Result<(), CliError> {
    match &cli.command {
        Command::Inspect { path } => output::inspect(&foldr_core::inspect(path)?, cli.json),
        Command::Doctor { path } => {
            let snapshot = foldr_core::inspect(path)?;
            emit(
                "doctor",
                &serde_json::json!({
                    "path": snapshot.path,
                    "platform": snapshot.platform,
                    "filesystem": snapshot.filesystem,
                    "capabilities": snapshot.capabilities,
                    "guidance": [
                        "Inspection never writes to the folder or scans descendants.",
                        "Support depends on this folder's filesystem and current user permissions.",
                        "A directory lock protects entries; existing files can remain editable.",
                        "Extended attributes may be lost when copying, archiving, syncing, or using Git."
                    ],
                }),
                cli.json,
            )
        }
        Command::Completions { shell } => {
            if cli.json {
                return Err(CliError::usage(
                    "completions emits shell source; omit --json.",
                ));
            }
            let mut document = Vec::new();
            clap_complete::generate(*shell, &mut Cli::command(), "foldr", &mut document);
            io::stdout().write_all(&document)?;
            Ok(())
        }
        Command::Man => {
            if cli.json {
                return Err(CliError::usage("man emits a roff document; omit --json."));
            }
            let mut document = Vec::new();
            clap_mangen::Man::new(Cli::command()).render(&mut document)?;
            io::stdout().write_all(&document)?;
            Ok(())
        }
        Command::Note(command) => mutations::note(command, cli.state_dir.as_deref(), cli.json),
        Command::Attr(command) => mutations::attr(command, cli.state_dir.as_deref(), cli.json),
        Command::Tags(command) => mutations::tags(command, cli.state_dir.as_deref(), cli.json),
        Command::Flags(command) => mutations::flags(command, cli.state_dir.as_deref(), cli.json),
        Command::Permissions(command) => {
            mutations::permissions(command, cli.state_dir.as_deref(), cli.json)
        }
        Command::Undo(command) => mutations::undo(command, cli.state_dir.as_deref(), cli.json),
        Command::Preset(command) => presets::run(
            command,
            cli.state_dir.as_deref(),
            cli.preset_dir.as_deref(),
            cli.json,
        ),
        Command::Diff {
            left,
            right,
            preset,
            snapshot,
        } => compare::run(left, right, *preset, *snapshot, cli.json),
    }
}
