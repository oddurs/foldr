use std::io::{self, Write};

use serde::Serialize;
use serde_json::{Value, json};

pub struct CliError {
    pub code: u8,
    pub message: String,
}

impl CliError {
    pub fn usage(message: impl Into<String>) -> Self {
        Self {
            code: 2,
            message: message.into(),
        }
    }
}

impl From<io::Error> for CliError {
    fn from(error: io::Error) -> Self {
        Self {
            code: if error.kind() == io::ErrorKind::BrokenPipe {
                0
            } else {
                1
            },
            message: error.to_string(),
        }
    }
}

impl From<serde_json::Error> for CliError {
    fn from(error: serde_json::Error) -> Self {
        Self {
            code: 1,
            message: error.to_string(),
        }
    }
}

impl From<foldr_core::FoldrError> for CliError {
    fn from(error: foldr_core::FoldrError) -> Self {
        use foldr_core::FoldrError;
        let code = match &error {
            FoldrError::InvalidInput(_) => 2,
            FoldrError::Unsupported(_) => 3,
            FoldrError::Conflict(_) => 4,
            FoldrError::Io { .. } | FoldrError::Journal(_) => 1,
        };
        Self {
            code,
            message: error.to_string(),
        }
    }
}

pub fn emit<T: Serialize>(command: &str, value: &T, json_output: bool) -> Result<(), CliError> {
    let value = serde_json::to_value(value)?;
    let stdout = io::stdout();
    let mut writer = stdout.lock();
    if json_output {
        serde_json::to_writer_pretty(
            &mut writer,
            &json!({
                "schema_version": 1,
                "command": command,
                "data": value,
            }),
        )
        .map_err(|error| {
            let code = if error.io_error_kind() == Some(io::ErrorKind::BrokenPipe) {
                0
            } else {
                1
            };
            CliError {
                code,
                message: error.to_string(),
            }
        })?;
        writeln!(writer)?;
    } else {
        render_value(&mut writer, "", &value, 0)?;
    }
    Ok(())
}

pub fn escaped(value: &str) -> String {
    value.chars().flat_map(char::escape_debug).collect()
}

pub fn inspect(snapshot: &foldr_core::FolderSnapshot, json_output: bool) -> Result<(), CliError> {
    if json_output {
        return emit("inspect", snapshot, true);
    }
    use foldr_core::Property;
    let stdout = io::stdout();
    let mut writer = stdout.lock();
    writeln!(
        writer,
        "Folder: {}",
        foldr_core::escape_bytes(&snapshot.path.bytes)
    )?;
    if let Some(target) = &snapshot.symlink_target {
        writeln!(
            writer,
            "Symlink target: {}",
            foldr_core::escape_bytes(&target.bytes)
        )?;
    }
    writeln!(writer, "Platform: {}", escaped(&snapshot.platform))?;
    writeln!(
        writer,
        "Identity: device {}, inode {}",
        snapshot.identity.device, snapshot.identity.inode
    )?;
    writeln!(
        writer,
        "Owner: {}  Group: {}  Mode: {:04o}",
        snapshot.owner, snapshot.group, snapshot.mode
    )?;
    writeln!(
        writer,
        "  Directory read lists names; execute traverses; write changes entries."
    )?;
    match &snapshot.filesystem {
        Property::Supported { value } => writeln!(
            writer,
            "Filesystem: {}{}",
            escaped(&value.name),
            if value.read_only { " (read-only)" } else { "" }
        )?,
        value => render_value(&mut writer, "Filesystem", &serde_json::to_value(value)?, 0)?,
    }
    match &snapshot.flags {
        Property::Supported { value } => {
            writeln!(
                writer,
                "Native flags: {} (0x{:x})",
                if value.names.is_empty() {
                    "none".into()
                } else {
                    escaped(&value.names.join(", "))
                },
                value.raw
            )?;
            if value
                .names
                .iter()
                .any(|name| name.contains("immutable") || name == "uchg")
            {
                writeln!(
                    writer,
                    "  Directory locking protects entries; existing file contents may remain editable."
                )?;
            }
        }
        value => render_value(
            &mut writer,
            "Native flags",
            &serde_json::to_value(value)?,
            0,
        )?,
    }
    match &snapshot.acl {
        Property::Supported { value } => {
            writeln!(
                writer,
                "ACL: {}",
                if value.has_extended_entries {
                    "extended entries present"
                } else {
                    "no extended entries"
                }
            )?;
            for entry in &value.entries {
                writeln!(writer, "  {}", escaped(entry))?;
            }
        }
        value => render_value(&mut writer, "ACL", &serde_json::to_value(value)?, 0)?,
    }
    match &snapshot.xattrs {
        Property::Supported { value } => {
            writeln!(writer, "Extended attributes: {}", value.len())?;
            for attribute in value {
                let value = match std::str::from_utf8(&attribute.value) {
                    Ok(text) => escaped(text),
                    Err(_) => format!("hex:{}", hex_bytes(&attribute.value)),
                };
                writeln!(
                    writer,
                    "  {} = {value}",
                    foldr_core::escape_bytes(&attribute.name.bytes)
                )?;
            }
        }
        value => render_value(
            &mut writer,
            "Extended attributes",
            &serde_json::to_value(value)?,
            0,
        )?,
    }
    writeln!(
        writer,
        "Use foldr doctor PATH for capability details. Inspection does not scan descendants."
    )?;
    Ok(())
}

fn render_value(
    writer: &mut impl Write,
    label: &str,
    value: &Value,
    depth: usize,
) -> io::Result<()> {
    let indentation = "  ".repeat(depth);
    match value {
        Value::Object(fields) => {
            if fields.contains_key("display") {
                if let Some(bytes) = fields.get("bytes").and_then(json_bytes) {
                    return writeln!(
                        writer,
                        "{indentation}{}: {}",
                        escaped(label),
                        foldr_core::escape_bytes(&bytes)
                    );
                }
            }
            if let Some(kind) = fields.get("kind").and_then(Value::as_str) {
                match kind {
                    "xattr" => {
                        if let Some(bytes) = json_bytes(&fields["name"]) {
                            return writeln!(
                                writer,
                                "{indentation}{}: xattr {}",
                                escaped(label),
                                foldr_core::escape_bytes(&bytes)
                            );
                        }
                    }
                    "bytes" => {
                        if fields.get("value").is_some_and(Value::is_null) {
                            return writeln!(writer, "{indentation}{}: (missing)", escaped(label));
                        }
                        if let Some(bytes) = json_bytes(&fields["value"]) {
                            let value = match std::str::from_utf8(&bytes) {
                                Ok(text) => format!("\"{}\"", escaped(text)),
                                Err(_) => format!("hex:{}", hex_bytes(&bytes)),
                            };
                            return writeln!(
                                writer,
                                "{indentation}{}: {value} ({} bytes)",
                                escaped(label),
                                bytes.len()
                            );
                        }
                    }
                    "mode" | "flags" => {
                        if let Some(value) = fields.get("value").and_then(Value::as_u64) {
                            return if kind == "mode" {
                                writeln!(writer, "{indentation}{}: {value:04o}", escaped(label))
                            } else {
                                writeln!(writer, "{indentation}{}: 0x{value:x}", escaped(label))
                            };
                        }
                    }
                    _ => {}
                }
            }
            if !label.is_empty() {
                writeln!(writer, "{indentation}{}:", escaped(label))?;
            }
            for (key, value) in fields {
                // The lossless byte representation remains available in JSON.
                if key == "bytes" && fields.contains_key("display") {
                    continue;
                }
                render_value(writer, key, value, depth + usize::from(!label.is_empty()))?;
            }
        }
        Value::Array(values) => {
            if values.iter().all(Value::is_number) {
                writeln!(writer, "{indentation}{}: {value}", escaped(label))?;
            } else {
                writeln!(writer, "{indentation}{}:", escaped(label))?;
                for value in values {
                    render_value(writer, "-", value, depth + 1)?;
                }
                if values.is_empty() {
                    writeln!(writer, "{indentation}  (none)")?;
                }
            }
        }
        Value::String(text) => {
            writeln!(writer, "{indentation}{}: {}", escaped(label), escaped(text))?
        }
        _ => writeln!(writer, "{indentation}{}: {value}", escaped(label))?,
    }
    Ok(())
}

fn json_bytes(value: &Value) -> Option<Vec<u8>> {
    value
        .as_array()?
        .iter()
        .map(|value| value.as_u64().and_then(|value| u8::try_from(value).ok()))
        .collect()
}

fn hex_bytes(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut value = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut value, "{byte:02x}").expect("writing to a String cannot fail");
    }
    value
}

pub fn diagnostic(error: &CliError, json_output: bool) {
    if matches!(error.code, 0 | 6) {
        return;
    }
    let stderr = io::stderr();
    let mut writer = stderr.lock();
    if json_output {
        let value = json!({ "schema_version": 1, "error": { "code": error.code, "message": error.message } });
        let _ = writeln!(writer, "{value}");
    } else {
        let _ = writeln!(writer, "foldr: {}", escaped(&error.message));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn human_text_cannot_inject_terminal_controls() {
        assert_eq!(escaped("a\n\u{1b}[31m\tb"), "a\\n\\u{1b}[31m\\tb");
    }
}
