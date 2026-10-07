use crate::{
    args::PresetCommand,
    mutations::state_directory,
    output::{CliError, emit},
};
use foldr_core::EncodedPath;
use std::path::Path;

pub fn run(command: &PresetCommand, state: Option<&Path>, json: bool) -> Result<(), CliError> {
    match command {
        PresetCommand::Save {
            path,
            output,
            fields,
        } => {
            let fields = fields
                .iter()
                .map(|field| {
                    if field == "attrs" {
                        "metadata".into()
                    } else {
                        field.clone()
                    }
                })
                .collect::<Vec<_>>();
            let preset = foldr_core::preset_from_snapshot(&foldr_core::inspect(path)?, &fields)?;
            foldr_core::write_preset(output, &preset)?;
            emit(
                "preset.save",
                &serde_json::json!({"file":EncodedPath::new(output),"preset":preset}),
                json,
            )
        }
        PresetCommand::Show { file } => emit("preset.show", &foldr_core::read_preset(file)?, json),
        PresetCommand::Import { file, output } | PresetCommand::Export { file, output } => {
            let preset = foldr_core::read_preset(file)?;
            foldr_core::write_preset(output, &preset)?;
            let command = if matches!(command, PresetCommand::Import { .. }) {
                "preset.import"
            } else {
                "preset.export"
            };
            emit(
                command,
                &serde_json::json!({"file":EncodedPath::new(output),"preset":preset}),
                json,
            )
        }
        PresetCommand::Apply {
            file,
            paths,
            options,
        } => {
            let preset = foldr_core::read_preset(file)?;
            if paths.len() == 1 {
                let plan = foldr_core::preset_plan(&paths[0], &preset, options.follow_symlink)?;
                if options.dry_run {
                    return emit("preset.apply", &plan, json);
                }
                let record = foldr_core::apply(&plan, &state_directory(state)?)?;
                emit("preset.apply", &record, json)?;
                if !record.succeeded() {
                    return Err(CliError {
                        code: 5,
                        message: format!(
                            "Change {} was only partially applied; review undo show.",
                            record.id
                        ),
                    });
                }
                return Ok(());
            }
            // Dry-run must not need HOME or touch the state directory.
            let state = if options.dry_run {
                std::path::PathBuf::new()
            } else {
                state_directory(state)?
            };
            let results = foldr_core::apply_batch(
                paths,
                &preset,
                &state,
                options.follow_symlink,
                options.dry_run,
            );
            let failures = results
                .iter()
                .filter(|result| result.error.is_some())
                .count();
            emit(
                "preset.apply",
                &serde_json::json!({"scope":"explicit_folders","recursive":false,"dry_run":options.dry_run,"targets":results}),
                json,
            )?;
            if failures > 0 {
                return Err(CliError {
                    code: 5,
                    message: format!(
                        "{failures} of {} explicit targets failed; successful targets retain their recovery records.",
                        paths.len()
                    ),
                });
            }
            Ok(())
        }
    }
}
