use crate::output::{CliError, emit};
use foldr_core::FolderSnapshot;
use std::{fs::File, path::Path};

fn snapshot(path: &Path) -> Result<FolderSnapshot, CliError> {
    let value: serde_json::Value = serde_json::from_reader(File::open(path)?)
        .map_err(|error| CliError::usage(format!("Invalid snapshot JSON: {error}")))?;
    let data = if value.get("command").is_some() {
        if value["schema_version"] != 1 || value["command"] != "inspect" {
            return Err(CliError::usage(
                "Snapshot must be an inspect JSON envelope with schema_version 1.",
            ));
        }
        value["data"].clone()
    } else {
        value
    };
    let snapshot: FolderSnapshot = serde_json::from_value(data)
        .map_err(|error| CliError::usage(format!("Invalid folder snapshot: {error}")))?;
    if snapshot.schema_version != 1 {
        return Err(CliError::usage("Unsupported snapshot schema version."));
    }
    Ok(snapshot)
}

pub fn run(
    left: &Path,
    right: &Path,
    preset: bool,
    snapshots: bool,
    json: bool,
) -> Result<(), CliError> {
    let left_snapshot = if snapshots {
        snapshot(left)?
    } else {
        foldr_core::inspect(left)?
    };
    let differences = if preset {
        foldr_core::compare_preset(&left_snapshot, &foldr_core::read_preset(right)?)?
    } else {
        let right_snapshot = if snapshots {
            snapshot(right)?
        } else {
            foldr_core::inspect(right)?
        };
        foldr_core::compare_snapshots(&left_snapshot, &right_snapshot)
    };
    emit(
        "diff",
        &serde_json::json!({"left":foldr_core::EncodedPath::new(left),"right":foldr_core::EncodedPath::new(right),"differences":differences}),
        json,
    )
}
