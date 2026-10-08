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
fn attribute_field(name: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut field = String::with_capacity(6 + name.len() * 2);
    field.push_str("xattr:");
    for byte in name {
        field.push(char::from(HEX[usize::from(byte >> 4)]));
        field.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    field
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
/// Decode names/colors without writing; legacy-only Finder labels are ambiguous.
pub fn finder_tags_from_snapshot(
    snapshot: &FolderSnapshot,
) -> Property<Vec<crate::platform::tags::FinderTag>> {
    use crate::platform::tags::{FINDER_INFO, TAG_XATTR, decode_finder_tags};
    if snapshot.platform != "macos" {
        return Property::Unsupported {
            reason: "Finder tags are a native macOS feature".into(),
        };
    }
    let attrs = match &snapshot.xattrs {
        Property::Supported { value } => value,
        Property::Unsupported { reason } => {
            return Property::Unsupported {
                reason: reason.clone(),
            };
        }
        Property::Unknown { reason } => {
            return Property::Unknown {
                reason: reason.clone(),
            };
        }
        Property::PermissionDenied { reason } => {
            return Property::PermissionDenied {
                reason: reason.clone(),
            };
        }
        Property::Unavailable { reason } => {
            return Property::Unavailable {
                reason: reason.clone(),
            };
        }
    };
    let raw = attrs
        .iter()
        .find(|a| a.name.bytes == TAG_XATTR)
        .map(|a| a.value.as_slice());
    match decode_finder_tags(raw) {
        Ok(mut tags) => {
            if tags.is_empty() {
                if let Some(info) = attrs.iter().find(|a| a.name.bytes == FINDER_INFO) {
                    if info.value.len() != 32
                        || u16::from_be_bytes([info.value[8], info.value[9]]) & 0x000e != 0
                    {
                        return Property::Unknown { reason: "legacy or malformed FinderInfo label prevents an unambiguous tag check".into() };
                    }
                }
            }
            tags.sort_by(|a, b| a.name.cmp(&b.name));
            Property::Supported { value: tags }
        }
        Err(error) => Property::Unknown {
            reason: error.to_string(),
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
    result.insert(
        "finder_tags".into(),
        property_value(&finder_tags_from_snapshot(snapshot)),
    );
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
                    attribute_field(&attr.name.bytes),
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
    crate::presets::validate_preset(preset)?;
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
    if let Some(names) = &preset.request.finder_tags {
        if preset.schema_version != 2 || preset.platform.as_deref() != Some("macos") {
            return Err(FoldrError::InvalidInput(
                "Finder tag presets require schema 2 and platform=macos".into(),
            ));
        }
        crate::presets::validate_tag_names(names)?;
        let tags = finder_tags_from_snapshot(snapshot);
        let observed = match &tags {
            Property::Supported { value } => ComparisonValue::Present {
                value: json!(value.iter().map(|tag| &tag.name).collect::<Vec<_>>()),
            },
            other => property_value(other),
        };
        let mut names = names.clone();
        names.sort();
        left.insert("finder_tags".into(), observed);
        right.insert(
            "finder_tags".into(),
            ComparisonValue::Present {
                value: json!(names),
            },
        );
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
        let field = attribute_field(&name);
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
/// A compliance check evaluates only fields selected by the partial preset.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceStatus {
    Compliant,
    Drift,
    Unsupported,
    Indeterminate,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PresetCompliance {
    pub status: ComplianceStatus,
    pub differences: Vec<Difference>,
}
pub fn check_preset(
    snapshot: &FolderSnapshot,
    preset: &Preset,
) -> Result<PresetCompliance, FoldrError> {
    let differences = compare_preset(snapshot, preset)?;
    let status = if differences.iter().any(|d| {
        matches!(
            d.left,
            ComparisonValue::Unknown { .. }
                | ComparisonValue::PermissionDenied { .. }
                | ComparisonValue::Unavailable { .. }
        )
    }) {
        ComplianceStatus::Indeterminate
    } else if differences
        .iter()
        .any(|d| matches!(d.left, ComparisonValue::Unsupported { .. }))
    {
        ComplianceStatus::Unsupported
    } else if differences.is_empty() {
        ComplianceStatus::Compliant
    } else {
        ComplianceStatus::Drift
    };
    Ok(PresetCompliance {
        status,
        differences,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn malformed_preset_requests_do_not_count_as_drift() {
        let temp = tempfile::tempdir().unwrap();
        let snapshot = crate::inspect(temp.path()).unwrap();
        let mut preset = Preset::from_toml("schema_version=1\n[settings]").unwrap();
        preset.request.mode = Some(0o10000);
        assert!(matches!(
            check_preset(&snapshot, &preset),
            Err(FoldrError::InvalidInput(_))
        ));
        preset.request.mode = None;
        preset.request.note = Some(Some("one".into()));
        preset
            .request
            .metadata
            .insert("note".into(), Some(b"two".to_vec()));
        assert!(matches!(
            check_preset(&snapshot, &preset),
            Err(FoldrError::InvalidInput(_))
        ));
        assert!(Preset::from_toml("schema_version=1\n[settings]\nmode=4096").is_err());
        assert!(
            Preset::from_toml(
                "schema_version=1\n[settings]\nnote='one'\n[settings.metadata]\nnote=[116,119,111]"
            )
            .is_err()
        );
    }
    #[test]
    fn compliance_reports_capability_failures_before_drift() {
        let temp = tempfile::tempdir().unwrap();
        let mut snapshot = crate::inspect(temp.path()).unwrap();
        let preset =
            Preset::from_toml("schema_version=1\n[settings]\nnote='expected'\nmode=448").unwrap();
        snapshot.mode = 0o755;
        snapshot.xattrs = Property::Unsupported {
            reason: "no attributes".into(),
        };
        assert_eq!(
            check_preset(&snapshot, &preset).unwrap().status,
            ComplianceStatus::Unsupported
        );
        for property in [
            Property::Unknown {
                reason: "unknown".into(),
            },
            Property::PermissionDenied {
                reason: "denied".into(),
            },
            Property::Unavailable {
                reason: "unavailable".into(),
            },
        ] {
            snapshot.xattrs = property;
            let result = check_preset(&snapshot, &preset).unwrap();
            assert_eq!(result.status, ComplianceStatus::Indeterminate);
            assert!(result.differences.iter().any(|d| d.field == "mode"));
        }
        snapshot.xattrs = Property::Supported { value: vec![] };
        assert_eq!(
            check_preset(&snapshot, &preset).unwrap().status,
            ComplianceStatus::Drift
        );
        let omitted = Preset::from_toml("schema_version=1\n[settings]").unwrap();
        snapshot.xattrs = Property::PermissionDenied {
            reason: "not selected".into(),
        };
        assert_eq!(
            check_preset(&snapshot, &omitted).unwrap().status,
            ComplianceStatus::Compliant
        );
    }
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
        assert_eq!(differences[0].field, "xattr:ff");
        assert_eq!(
            differences[0].left,
            ComparisonValue::Present {
                value: json!([0, 255])
            }
        );
        assert_eq!(differences[0].right, ComparisonValue::Missing);
    }
}
