use crate::{
    args::{PresetCommand, PresetTargets},
    mutations::state_directory,
    output::{CliError, emit},
};
use foldr_core::EncodedPath;
use std::path::Path;

pub fn source<'a>(
    source: &'a PresetTargets,
    library: Option<&Path>,
) -> Result<(foldr_core::Preset, &'a [std::path::PathBuf]), CliError> {
    if let Some(name) = &source.name {
        return Ok((crate::library::resolve(library, name)?, &source.inputs));
    }
    if source.inputs.len() < 2 {
        return Err(CliError::usage(
            "Specify FILE followed by at least one PATH, or --name NAME followed by PATH.",
        ));
    }
    Ok((
        foldr_core::read_preset(&source.inputs[0])?,
        &source.inputs[1..],
    ))
}

pub fn run(
    command: &PresetCommand,
    state: Option<&Path>,
    library: Option<&Path>,
    json: bool,
) -> Result<(), CliError> {
    match command {
        PresetCommand::List => emit("preset.list", &crate::library::list(library)?, json),
        PresetCommand::Check { source: targets } => {
            let (preset, paths) = source(targets, library)?;
            let mut code = 0;
            let mut results = Vec::with_capacity(paths.len());
            for path in paths {
                let result = foldr_core::inspect(path)
                    .and_then(|snapshot| foldr_core::check_preset(&snapshot, &preset));
                let target = match result {
                    Ok(compliance) => {
                        let target_code = match compliance.status {
                            foldr_core::ComplianceStatus::Compliant => 0,
                            foldr_core::ComplianceStatus::Drift => 6,
                            foldr_core::ComplianceStatus::Unsupported => 3,
                            foldr_core::ComplianceStatus::Indeterminate => 1,
                        };
                        code = aggregate_code(code, target_code);
                        serde_json::json!({"path": EncodedPath::new(path), "status": compliance.status, "differences": compliance.differences})
                    }
                    Err(error) => {
                        let error = CliError::from(error);
                        code = aggregate_code(code, error.code);
                        serde_json::json!({"path": EncodedPath::new(path), "status": if error.code == 3 { "unsupported" } else { "indeterminate" }, "differences": [], "error": {"code": error.code, "message": error.message}})
                    }
                };
                results.push(target);
            }
            emit(
                "preset.check",
                &serde_json::json!({"scope":"explicit_folders", "recursive":false, "targets":results}),
                json,
            )?;
            if code != 0 {
                return Err(CliError {
                    code,
                    message: if code == 6 {
                        "Explicit folders differ from requested preset settings.".into()
                    } else {
                        "Preset compliance could not be established for every target; review per-target results.".into()
                    },
                });
            }
            Ok(())
        }
        PresetCommand::Install { file, name } => {
            let path = crate::library::install(library, file, name)?;
            emit(
                "preset.install",
                &serde_json::json!({"name": name, "file": EncodedPath::new(&path)}),
                json,
            )
        }
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
            source: targets,
            options,
        } => {
            let (preset, paths) = source(targets, library)?;
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

fn aggregate_code(left: u8, right: u8) -> u8 {
    fn rank(code: u8) -> u8 {
        match code {
            1 => 5,
            4 => 4,
            3 => 3,
            2 => 2,
            6 => 1,
            _ => 0,
        }
    }
    if rank(right) > rank(left) {
        right
    } else {
        left
    }
}
