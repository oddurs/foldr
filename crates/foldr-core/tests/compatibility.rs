use foldr_core::{ChangeRecord, EncodedPath, FieldValue, FolderSnapshot, Preset};

#[test]
fn actual_v010_presets_preserve_binary_omission_and_removal() {
    let preset = Preset::from_toml(include_str!("fixtures/v0.1.0/preset.toml")).unwrap();
    assert_eq!(preset.request.note, Some(Some("old note".into())));
    assert_eq!(preset.request.metadata["binary"], Some(vec![0, 255, 10]));
    assert_eq!(preset.request.hidden, None);
    assert_eq!(preset.request.immutable, None);
    assert_eq!(
        Preset::from_toml(&preset.to_toml().unwrap()).unwrap(),
        preset
    );
    let removal = Preset::from_toml(include_str!("fixtures/v0.1.0/removal.toml")).unwrap();
    assert_eq!(removal.request.note, Some(None));
    assert_eq!(removal.request.metadata["binary"], None);
    assert_eq!(removal.request.mode, None);
    assert_eq!(
        Preset::from_toml(&removal.to_toml().unwrap()).unwrap(),
        removal
    );
}

#[test]
fn actual_v010_snapshot_envelope_roundtrips_losslessly() {
    let envelope: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/v0.1.0/snapshot.json")).unwrap();
    assert_eq!(envelope["schema_version"], 1);
    assert_eq!(envelope["command"], "inspect");
    let snapshot: FolderSnapshot = serde_json::from_value(envelope["data"].clone()).unwrap();
    assert_eq!(serde_json::to_value(&snapshot).unwrap(), envelope["data"]);
    assert!(
        snapshot
            .xattrs
            .value()
            .unwrap()
            .iter()
            .any(|a| a.value == [0, 255, 10])
    );
    let encoded = EncodedPath::from_bytes(b"folder\xff\n".to_vec());
    assert_eq!(
        serde_json::from_value::<EncodedPath>(serde_json::to_value(&encoded).unwrap()).unwrap(),
        encoded
    );
}

#[test]
fn actual_v010_recovery_records_roundtrip_losslessly() {
    for text in [
        include_str!("fixtures/v0.1.0/binary-record.json"),
        include_str!("fixtures/v0.1.0/removal-record.json"),
    ] {
        let original: serde_json::Value = serde_json::from_str(text).unwrap();
        let record: ChangeRecord = serde_json::from_str(text).unwrap();
        assert!(record.succeeded());
        assert_eq!(serde_json::to_value(&record).unwrap(), original);
        assert!(record.fields.iter().any(|f| matches!(&f.change.before, FieldValue::Bytes { value: Some(v) } if v == &[0,255,10]) || matches!(&f.change.after, FieldValue::Bytes { value: Some(v) } if v == &[0,255,10])));
        #[cfg(target_os = "macos")]
        {
            let state = tempfile::tempdir().unwrap();
            std::fs::write(state.path().join(format!("{}.json", record.id)), text).unwrap();
            assert_eq!(
                foldr_core::load_record(state.path(), &record.id).unwrap(),
                record
            );
        }
    }
}
