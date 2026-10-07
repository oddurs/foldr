use crate::{
    args::{
        AttrCommand, FlagsCommand, MutationOptions, NoteCommand, PermissionsCommand, UndoCommand,
    },
    output::{CliError, emit},
};
use foldr_core::{ChangeRequest, EncodedPath};
use std::path::{Path, PathBuf};

pub fn state_directory(explicit: Option<&Path>) -> Result<PathBuf, CliError> {
    if let Some(path) = explicit {
        return Ok(path.to_owned());
    }
    if let Some(path) = std::env::var_os("FOLDR_STATE_DIR") {
        return Ok(path.into());
    }
    if cfg!(target_os = "linux") {
        if let Some(path) = std::env::var_os("XDG_STATE_HOME") {
            let path = PathBuf::from(path);
            if path.is_absolute() {
                return Ok(path.join("foldr"));
            }
        }
    }
    let home = std::env::var_os("HOME")
        .ok_or_else(|| CliError::usage("Set --state-dir when HOME is unavailable."))?;
    Ok(PathBuf::from(home).join(if cfg!(target_os = "macos") {
        "Library/Application Support/foldr/history"
    } else {
        ".local/state/foldr"
    }))
}

pub fn apply_request(
    path: &Path,
    request: &ChangeRequest,
    options: &MutationOptions,
    state: Option<&Path>,
    json: bool,
    command: &str,
) -> Result<(), CliError> {
    let plan = foldr_core::plan(path, request, options.follow_symlink)?;
    if options.dry_run {
        return emit(command, &plan, json);
    }
    let record = foldr_core::apply(&plan, &state_directory(state)?)?;
    emit(command, &record, json)?;
    if !record.succeeded() {
        return Err(CliError {
            code: 5,
            message: format!(
                "Change {} was only partially applied; inspect its recovery record with undo show.",
                record.id
            ),
        });
    }
    Ok(())
}

fn get(path: &Path, key: &str, json: bool, command: &str) -> Result<(), CliError> {
    let value = foldr_core::read_metadata(path, key)?;
    let text = value
        .as_ref()
        .and_then(|value| String::from_utf8(value.clone()).ok());
    emit(
        command,
        &serde_json::json!({"path":EncodedPath::new(path),"key":key,"value":value,"text":text}),
        json,
    )
}

pub fn note(command: &NoteCommand, state: Option<&Path>, json: bool) -> Result<(), CliError> {
    match command {
        NoteCommand::Get { path } => get(path, "note", json, "note.get"),
        NoteCommand::Set {
            path,
            value,
            options,
        } => apply_request(
            path,
            &ChangeRequest {
                note: Some(Some(value.clone())),
                ..Default::default()
            },
            options,
            state,
            json,
            "note.set",
        ),
        NoteCommand::Remove { path, options } => apply_request(
            path,
            &ChangeRequest {
                note: Some(None),
                ..Default::default()
            },
            options,
            state,
            json,
            "note.remove",
        ),
    }
}

pub fn attr(command: &AttrCommand, state: Option<&Path>, json: bool) -> Result<(), CliError> {
    match command {
        AttrCommand::Get { path, key } => get(path, key, json, "attr.get"),
        AttrCommand::Set {
            path,
            key,
            value,
            hex,
            options,
        } => {
            let bytes = if *hex {
                decode_hex(value)?
            } else {
                value.as_bytes().to_vec()
            };
            let request = ChangeRequest {
                metadata: [(key.clone(), Some(bytes))].into(),
                ..Default::default()
            };
            apply_request(path, &request, options, state, json, "attr.set")
        }
        AttrCommand::Remove { path, key, options } => {
            let request = ChangeRequest {
                metadata: [(key.clone(), None)].into(),
                ..Default::default()
            };
            apply_request(path, &request, options, state, json, "attr.remove")
        }
    }
}

fn decode_hex(value: &str) -> Result<Vec<u8>, CliError> {
    if value.len() % 2 != 0 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(CliError::usage(
            "Hex values need an even number of hexadecimal digits, without separators.",
        ));
    }
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|chunk| {
            u8::from_str_radix(std::str::from_utf8(chunk).expect("validated ASCII"), 16)
                .map_err(|error| CliError::usage(error.to_string()))
        })
        .collect()
}

pub fn flags(command: &FlagsCommand, state: Option<&Path>, json: bool) -> Result<(), CliError> {
    let FlagsCommand::Set {
        path,
        hidden,
        immutable,
        options,
    } = command;
    if hidden.is_none() && immutable.is_none() {
        return Err(CliError::usage(
            "Specify --hidden true/false or --immutable true/false.",
        ));
    }
    apply_request(
        path,
        &ChangeRequest {
            hidden: *hidden,
            immutable: *immutable,
            ..Default::default()
        },
        options,
        state,
        json,
        "flags.set",
    )
}

pub fn permissions(
    command: &PermissionsCommand,
    state: Option<&Path>,
    json: bool,
) -> Result<(), CliError> {
    let PermissionsCommand::Set {
        path,
        mode,
        options,
    } = command;
    if mode.is_empty() || mode.len() > 4 || !mode.bytes().all(|byte| (b'0'..=b'7').contains(&byte))
    {
        return Err(CliError::usage(
            "Mode must be 1 to 4 octal digits, from 0000 through 7777.",
        ));
    }
    let mode = u32::from_str_radix(mode, 8).map_err(|error| CliError::usage(error.to_string()))?;
    apply_request(
        path,
        &ChangeRequest {
            mode: Some(mode),
            ..Default::default()
        },
        options,
        state,
        json,
        "permissions.set",
    )
}

pub fn undo(command: &UndoCommand, state: Option<&Path>, json: bool) -> Result<(), CliError> {
    let state = state_directory(state)?;
    match command {
        UndoCommand::History => emit("undo.history", &foldr_core::history(&state)?, json),
        UndoCommand::Show { id } => emit("undo.show", &foldr_core::load_record(&state, id)?, json),
        UndoCommand::Apply { id, dry_run } => {
            let record = foldr_core::load_record(&state, id)?;
            if *dry_run {
                return emit("undo.apply", &foldr_core::undo_plan(&record)?, json);
            }
            let result = foldr_core::undo(&record, &state, false)?;
            emit("undo.apply", &result, json)?;
            if !result.succeeded() {
                return Err(CliError {
                    code: 5,
                    message: format!(
                        "Undo {} was only partially applied; review its recovery record.",
                        result.id
                    ),
                });
            }
            Ok(())
        }
    }
}
