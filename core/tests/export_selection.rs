use spiritbyte_core::{backup::{self, ExportSelection}, vault::VaultData};

fn fixture() -> VaultData {
    serde_json::from_str(r##"{
      "folders":[
        {"id":"work","name":"Work","icon":"briefcase","color":"#ffb000"},
        {"id":"mail","name":"Mail","parentId":"work","icon":"mail"},
        {"id":"other","name":"Other","icon":"cloud"},
        {"id":"empty","name":"Empty","parentId":"work","icon":"gift"}
      ],
      "entries":[
        {"id":"a","title":"Selected","password":"included","folderId":"mail","createdAt":1,"updatedAt":2},
        {"id":"b","title":"Unchecked sibling","password":"excluded-sibling","folderId":"mail","createdAt":1,"updatedAt":2},
        {"id":"c","title":"Other","password":"excluded-other","folderId":"other","createdAt":1,"updatedAt":2},
        {"id":"d","title":"Unfiled","password":"unfiled","createdAt":1,"updatedAt":2}
      ]
    }"##).unwrap()
}

#[test]
fn partial_folder_export_never_readds_unchecked_credentials() {
    let original = fixture();
    let selected = backup::select(&original, Some(&ExportSelection {
        entry_ids: vec!["a".into()], folder_ids: vec!["mail".into()],
    })).unwrap();
    assert_eq!(selected.entries.len(), 1);
    assert_eq!(selected.entries[0].id, "a");
    assert_eq!(selected.folders.iter().map(|f| f.id.as_str()).collect::<Vec<_>>(), vec!["work", "mail"]);
    assert_eq!(selected.folders[0].icon.as_deref(), Some("briefcase"));
    assert_eq!(selected.folders[0].color.as_deref(), Some("#ffb000"));
    let encrypted = backup::export(&selected, "selective backup password").unwrap();
    let restored = backup::decrypt(&encrypted, "selective backup password").unwrap();
    assert_eq!(restored.entries.len(), 1);
    assert_eq!(restored.entries[0].password, "included");
    let imported = backup::merge(&VaultData::default(), restored);
    backup::validate(&imported).unwrap();
    assert_eq!(imported.folders.len(), 2);
    assert_eq!(original.entries.len(), 4);
}

#[test]
fn individual_entries_add_only_their_ancestors() {
    let selected = backup::select(&fixture(), Some(&ExportSelection {
        entry_ids: vec!["a".into(), "d".into()], folder_ids: vec![],
    })).unwrap();
    assert_eq!(selected.entries.len(), 2);
    assert_eq!(selected.folders.len(), 2);
    assert!(!selected.folders.iter().any(|f| f.id == "other"));
}

#[test]
fn explicitly_selected_empty_folders_are_preserved() {
    let selected = backup::select(&fixture(), Some(&ExportSelection {
        entry_ids: vec![], folder_ids: vec!["empty".into()],
    })).unwrap();
    assert!(selected.entries.is_empty());
    assert_eq!(selected.folders.iter().map(|f| f.id.as_str()).collect::<Vec<_>>(), vec!["work", "empty"]);
}

#[test]
fn all_exports_everything_but_empty_or_stale_selection_is_rejected() {
    let all = backup::select(&fixture(), None).unwrap();
    assert_eq!(all.entries.len(), 4);
    assert_eq!(all.folders.len(), 4);
    for selection in [
        ExportSelection { entry_ids: vec![], folder_ids: vec![] },
        ExportSelection { entry_ids: vec!["missing".into()], folder_ids: vec![] },
        ExportSelection { entry_ids: vec![], folder_ids: vec!["missing".into()] },
    ] { assert!(backup::select(&fixture(), Some(&selection)).is_err()); }
}
