//! Semantic, read-only comparisons exclude path identity and volatile metadata.
use crate::{model::*, presets::Preset};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ComparisonValue {
    Present { value: Value },
    Missing,
    Unsupported { reason: String },
    Unknown { reason: String },
    PermissionDenied { reason: String },
    Unavailable { reason: String },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Difference {
    pub field: String,
    pub scope: Scope,
    pub left: ComparisonValue,
    pub right: ComparisonValue,
}
fn property_value<T: Serialize>(property: &Property<T>) -> ComparisonValue {
    match property {
        Property::Supported { value } => ComparisonValue::Present {
            value: json!(value),
        },
        Property::Unsupported { reason } => ComparisonValue::Unsupported {
            reason: reason.clone(),
        },
        Property::Unknown { reason } => ComparisonValue::Unknown {
            reason: reason.clone(),
        },
        Property::PermissionDenied { reason } => ComparisonValue::PermissionDenied {
            reason: reason.clone(),
        },
        Property::Unavailable { reason } => ComparisonValue::Unavailable {
            reason: reason.clone(),
        },
    }
}
fn values(snapshot: &FolderSnapshot) -> BTreeMap<String, ComparisonValue> {
    let mut result = BTreeMap::from([
        (
            "owner".into(),
            ComparisonValue::Present {
                value: json!(snapshot.owner),
            },
        ),
        (
            "group".into(),
            ComparisonValue::Present {
                value: json!(snapshot.group),
            },
        ),
        (
            "mode".into(),
            ComparisonValue::Present {
                value: json!(snapshot.mode),
            },
        ),
        ("acl".into(), property_value(&snapshot.acl)),
    ]);
    match &snapshot.flags {
        Property::Supported { value } => {
            result.insert(
                "immutable".into(),
                ComparisonValue::Present {
                    value: json!(
                        value.raw
                            & if snapshot.platform == "macos" {
                                2
                            } else {
                                0x10
                            }
                            != 0
                    ),
                },
            );
            if snapshot.platform == "macos" {
                result.insert(
                    "hidden".into(),
                    ComparisonValue::Present {
                        value: json!(value.raw & 0x8000 != 0),
                    },
                );
            } else {
                result.insert(
                    "hidden".into(),
                    ComparisonValue::Unsupported {
                        reason: "native hidden flag unavailable on this platform".into(),
                    },
                );
            }
        }
        other => {
            result.insert("immutable".into(), property_value(other));
            result.insert("hidden".into(), property_value(other));
        }
    }
    match &snapshot.xattrs {
        Property::Supported { value } => {
            for attr in value {
                result.insert(
                    format!(
                        "xattr:{}",
                        attr.name
                            .bytes
                            .iter()
                            .map(|b| format!("{b:02x}"))
                            .collect::<String>()
                    ),
                    ComparisonValue::Present {
                        value: json!(attr.value),
                    },
                );
            }
        }
        other => {
            result.insert("xattrs".into(), property_value(other));
        }
    }
    result
}
pub fn compare_snapshots(left: &FolderSnapshot, right: &FolderSnapshot) -> Vec<Difference> {
    let mut left_values = values(left);
    let mut right_values = values(right);
    if left.xattrs.value().is_none() || right.xattrs.value().is_none() {
        left_values.retain(|k, _| !k.starts_with("xattr:"));
        right_values.retain(|k, _| !k.starts_with("xattr:"));
        left_values.insert("xattrs".into(), property_value(&left.xattrs));
        right_values.insert("xattrs".into(), property_value(&right.xattrs));
    }
    differences(left_values, right_values)
}
fn differences(
    left: BTreeMap<String, ComparisonValue>,
    right: BTreeMap<String, ComparisonValue>,
) -> Vec<Difference> {
    let mut keys = left.keys().chain(right.keys()).cloned().collect::<Vec<_>>();
    keys.sort();
    keys.dedup();
    keys.into_iter()
        .filter_map(|field| {
            let a = left
                .get(&field)
                .cloned()
                .unwrap_or(ComparisonValue::Missing);
            let b = right
                .get(&field)
                .cloned()
                .unwrap_or(ComparisonValue::Missing);
            if a != b {
                Some(Difference {
                    field,
                    scope: Scope::Folder,
                    left: a,
                    right: b,
                })
            } else {
                None
            }
        })
        .collect()
}
pub fn compare_preset(
    snapshot: &FolderSnapshot,
    preset: &Preset,
) -> Result<Vec<Difference>, FoldrError> {
    if preset.schema_version != 1 {
        return Err(FoldrError::Unsupported("unsupported preset schema".into()));
    }
    if preset
        .platform
        .as_deref()
        .is_some_and(|platform| platform != snapshot.platform)
    {
        return Err(FoldrError::Unsupported(
            "preset targets another platform".into(),
        ));
    }
    let mut left = BTreeMap::new();
    let mut right = BTreeMap::new();
    let actual = values(snapshot);
    for (name, value) in [
        ("mode", preset.request.mode.map(|v| json!(v))),
        ("hidden", preset.request.hidden.map(|v| json!(v))),
        ("immutable", preset.request.immutable.map(|v| json!(v))),
    ] {
        if let Some(value) = value {
            left.insert(
                name.into(),
                actual
                    .get(name)
                    .cloned()
                    .unwrap_or(ComparisonValue::Missing),
            );
            right.insert(name.into(), ComparisonValue::Present { value });
        }
    }
    let mut metadata = preset.request.metadata.clone();
    if let Some(note) = &preset.request.note {
        metadata.insert("note".into(), note.as_ref().map(|n| n.as_bytes().to_vec()));
    }
    for (key, value) in metadata {
        let prefix = if snapshot.platform == "macos" {
            "com.foldr."
        } else {
            "user.foldr."
        };
        let name = format!("{prefix}{key}").into_bytes();
        let field = format!(
            "xattr:{}",
            name.iter().map(|b| format!("{b:02x}")).collect::<String>()
        );
        let observed = if snapshot.xattrs.value().is_some() {
            actual
                .get(&field)
                .cloned()
                .unwrap_or(ComparisonValue::Missing)
        } else {
            property_value(&snapshot.xattrs)
        };
        left.insert(field.clone(), observed);
        right.insert(
            field,
            value.map_or(ComparisonValue::Missing, |v| ComparisonValue::Present {
                value: json!(v),
            }),
        );
    }
    Ok(differences(left, right))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn differences_keep_missing_and_unsupported_distinct() {
        let temp = tempfile::tempdir().unwrap();
        let snapshot = crate::inspect(temp.path()).unwrap();
        let mut other = snapshot.clone();
        other.xattrs = Property::Unsupported {
            reason: "filesystem".into(),
        };
        let diff = compare_snapshots(&snapshot, &other);
        assert!(
            diff.iter()
                .any(|d| d.field == "xattrs"
                    && matches!(d.right, ComparisonValue::Unsupported { .. }))
        );
        let preset = Preset::from_toml("schema_version=1\n[settings]\nnote='hi'").unwrap();
        let diff = compare_preset(&snapshot, &preset).unwrap();
        assert!(
            diff.iter()
                .any(|d| matches!(d.left, ComparisonValue::Missing))
        );
    }
    #[test]
    fn binary_differences_and_volatile_identity() {
        let temp = tempfile::tempdir().unwrap();
        let mut left = crate::inspect(temp.path()).unwrap();
        let mut right = left.clone();
        left.xattrs = Property::Supported {
            value: vec![ExtendedAttribute {
                name: EncodedPath::from_bytes(vec![255]),
                value: vec![0, 255],
            }],
        };
        right.xattrs = Property::Supported { value: vec![] };
        right.identity.inode += 1;
        let differences = compare_snapshots(&left, &right);
        assert_eq!(differences.len(), 1);
        assert_eq!(
            differences[0].left,
            ComparisonValue::Present {
                value: json!([0, 255])
            }
        );
        assert_eq!(differences[0].right, ComparisonValue::Missing);
    }
}
