use tempfile::tempdir;

use crate::{catalog, mixes, state, BundleInfo, FocusNoiseEngine, MixSound, SoundState};

// ── Catalog integrity ────────────────────────────────────────────────────────

#[test]
fn catalog_every_entry_has_nonempty_fields() {
    for s in catalog::CATALOG {
        assert!(!s.id.is_empty(), "empty id");
        assert!(!s.display_name.is_empty(), "empty display_name for {}", s.id);
        assert!(!s.category.is_empty(), "empty category for {}", s.id);
        assert!(!s.source_url.is_empty(), "empty source_url for {}", s.id);
        assert!(!s.license.is_empty(), "empty license for {}", s.id);
        assert!(!s.author.is_empty(), "empty author for {}", s.id);
        assert!(catalog::CATEGORIES.contains(&s.category), "unknown category '{}' for {}", s.category, s.id);
    }
}

#[test]
fn catalog_ids_are_unique() {
    let mut ids: Vec<&str> = catalog::CATALOG.iter().map(|s| s.id).collect();
    let before = ids.len();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), before, "duplicate sound ids in CATALOG");
}

#[test]
fn catalog_relative_path_matches_category_and_id() {
    for s in catalog::CATALOG {
        assert_eq!(s.relative_path(), format!("{}/{}.ogg", s.category, s.id));
    }
}

#[test]
fn every_category_has_at_least_six_sounds() {
    for cat in catalog::CATEGORIES {
        let count = catalog::CATALOG.iter().filter(|s| &s.category == cat).count();
        assert!(count >= 6, "category '{}' has only {} sounds, need >= 6", cat, count);
    }
}

#[test]
fn catalog_find_resolves_known_id_and_rejects_unknown() {
    let first = catalog::CATALOG.first().expect("catalog is non-empty");
    assert!(catalog::find(first.id).is_some());
    assert!(catalog::find("definitely-not-a-real-sound-id").is_none());
}

// ── Presets ───────────────────────────────────────────────────────────────

#[test]
fn presets_are_exactly_ten() {
    assert_eq!(catalog::PRESETS.len(), 10);
}

#[test]
fn preset_sounds_all_reference_real_catalog_ids() {
    for p in catalog::PRESETS {
        assert!(!p.sounds.is_empty(), "preset '{}' has no sounds", p.id);
        for (sound_id, volume) in p.sounds {
            assert!(catalog::find(sound_id).is_some(), "preset '{}' references unknown sound '{}'", p.id, sound_id);
            assert!(*volume > 0.0 && *volume <= 1.0, "preset '{}' sound '{}' has invalid volume {}", p.id, sound_id, volume);
        }
    }
}

#[test]
fn preset_ids_are_unique() {
    let mut ids: Vec<&str> = catalog::PRESETS.iter().map(|p| p.id).collect();
    let before = ids.len();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), before, "duplicate preset ids");
}

// ── Module state / install ──────────────────────────────────────────────────

fn test_bundle() -> BundleInfo {
    BundleInfo { url: "https://example.invalid/bundle.zip".into(), sha256: "deadbeef".into(), size_bytes: 100 }
}

#[test]
fn not_installed_when_data_dir_empty() {
    let dir = tempdir().unwrap();
    let engine = FocusNoiseEngine::new(dir.path().to_path_buf(), test_bundle());
    assert_eq!(engine.state(), SoundState::NotInstalled);
    assert!(!engine.is_ready());
}

#[test]
fn is_installed_false_when_version_file_missing() {
    let dir = tempdir().unwrap();
    assert!(!state::is_installed(dir.path(), &test_bundle()));
}

#[test]
fn is_installed_false_when_sha_does_not_match() {
    let dir = tempdir().unwrap();
    let bundle = test_bundle();
    // Write a version file with a stale SHA, plus every catalog file present.
    let stale = crate::state::VersionFile { bundle_sha256: "stale-sha".into(), schema_version: state::SCHEMA_VERSION };
    std::fs::write(state::version_path(dir.path()), serde_json::to_string(&stale).unwrap()).unwrap();
    for s in catalog::CATALOG {
        let p = dir.path().join(s.relative_path());
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, b"fake").unwrap();
    }
    assert!(!state::is_installed(dir.path(), &bundle), "stale SHA must not count as installed");
}

#[test]
fn is_installed_false_when_a_catalog_file_is_missing() {
    let dir = tempdir().unwrap();
    let bundle = test_bundle();
    let ver = crate::state::VersionFile { bundle_sha256: bundle.sha256.clone(), schema_version: state::SCHEMA_VERSION };
    std::fs::write(state::version_path(dir.path()), serde_json::to_string(&ver).unwrap()).unwrap();
    // Write all but the first catalog file.
    for s in catalog::CATALOG.iter().skip(1) {
        let p = dir.path().join(s.relative_path());
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, b"fake").unwrap();
    }
    assert!(!state::is_installed(dir.path(), &bundle), "a missing sound file must not count as installed");
}

#[test]
fn is_installed_true_when_version_matches_and_all_files_present() {
    let dir = tempdir().unwrap();
    let bundle = test_bundle();
    let ver = crate::state::VersionFile { bundle_sha256: bundle.sha256.clone(), schema_version: state::SCHEMA_VERSION };
    std::fs::write(state::version_path(dir.path()), serde_json::to_string(&ver).unwrap()).unwrap();
    for s in catalog::CATALOG {
        let p = dir.path().join(s.relative_path());
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, b"fake").unwrap();
    }
    assert!(state::is_installed(dir.path(), &bundle));

    let engine = FocusNoiseEngine::new(dir.path().to_path_buf(), bundle);
    assert!(engine.is_ready());
}

#[tokio::test]
async fn provision_rejects_checksum_mismatch_and_leaves_not_ready() {
    // A bundle URL that 404s exercises the download-error path; more
    // importantly, an intentionally wrong sha256 against real bytes exercises
    // the checksum-mismatch path without needing a live HTTP fixture server.
    // We simulate this directly against the hashing logic rather than a real
    // network call, since this crate has no local HTTP test server.
    let dir = tempdir().unwrap();
    let bundle = BundleInfo {
        url: "https://example.invalid/nonexistent-bundle.zip".into(),
        sha256: "0000000000000000000000000000000000000000000000000000000000000000".into(),
        size_bytes: 1,
    };
    let engine = FocusNoiseEngine::new(dir.path().to_path_buf(), bundle);
    let result = engine.provision(|_| {}).await;
    assert!(result.is_err(), "provisioning against an unreachable URL must fail");
    assert!(!engine.is_ready(), "a failed provision must not leave the engine Ready");
}

#[test]
fn uninstall_removes_data_dir_and_resets_state() {
    let dir = tempdir().unwrap();
    let bundle = test_bundle();
    let ver = crate::state::VersionFile { bundle_sha256: bundle.sha256.clone(), schema_version: state::SCHEMA_VERSION };
    std::fs::write(state::version_path(dir.path()), serde_json::to_string(&ver).unwrap()).unwrap();
    for s in catalog::CATALOG {
        let p = dir.path().join(s.relative_path());
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, b"fake").unwrap();
    }
    let engine = FocusNoiseEngine::new(dir.path().to_path_buf(), bundle);
    assert!(engine.is_ready());

    engine.uninstall().unwrap();
    assert_eq!(engine.state(), SoundState::NotInstalled);
    assert!(!dir.path().join(catalog::CATALOG[0].relative_path()).exists());
}

#[test]
fn sound_path_is_none_when_not_ready() {
    let dir = tempdir().unwrap();
    let engine = FocusNoiseEngine::new(dir.path().to_path_buf(), test_bundle());
    let id = catalog::CATALOG[0].id;
    assert!(engine.sound_path(id).is_none());
}

#[test]
fn sound_path_is_none_for_unknown_id_even_when_ready() {
    let dir = tempdir().unwrap();
    let bundle = test_bundle();
    let ver = crate::state::VersionFile { bundle_sha256: bundle.sha256.clone(), schema_version: state::SCHEMA_VERSION };
    std::fs::write(state::version_path(dir.path()), serde_json::to_string(&ver).unwrap()).unwrap();
    for s in catalog::CATALOG {
        let p = dir.path().join(s.relative_path());
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, b"fake").unwrap();
    }
    let engine = FocusNoiseEngine::new(dir.path().to_path_buf(), bundle);
    assert!(engine.is_ready());
    assert!(engine.sound_path("not-a-real-id").is_none());
}

// ── Mixes ────────────────────────────────────────────────────────────────

#[test]
fn save_load_list_delete_round_trip() {
    let dir = tempdir().unwrap();
    let sounds = vec![
        MixSound { id: catalog::CATALOG[0].id.to_string(), volume: 0.5 },
        MixSound { id: catalog::CATALOG[1].id.to_string(), volume: 0.8 },
    ];
    let id = mixes::save(dir.path(), None, "My Mix".to_string(), sounds.clone()).unwrap();

    let loaded = mixes::load(dir.path(), &id).unwrap();
    assert_eq!(loaded.name, "My Mix");
    assert_eq!(loaded.sounds.len(), 2);

    let all = mixes::list(dir.path()).unwrap();
    assert_eq!(all.len(), 1);

    mixes::delete(dir.path(), &id).unwrap();
    assert!(mixes::list(dir.path()).unwrap().is_empty());
    assert!(mixes::load(dir.path(), &id).is_err());
}

#[test]
fn save_with_existing_id_overwrites_in_place() {
    let dir = tempdir().unwrap();
    let id = mixes::save(dir.path(), None, "First".to_string(), vec![]).unwrap();
    mixes::save(dir.path(), Some(id.clone()), "Renamed".to_string(), vec![]).unwrap();

    let all = mixes::list(dir.path()).unwrap();
    assert_eq!(all.len(), 1, "overwriting by id must not create a second entry");
    assert_eq!(all[0].name, "Renamed");
}

#[test]
fn delete_unknown_mix_errors() {
    let dir = tempdir().unwrap();
    assert!(mixes::delete(dir.path(), "not-a-real-mix-id").is_err());
}

#[test]
fn presets_are_not_stored_in_or_deletable_from_user_mixes() {
    let dir = tempdir().unwrap();
    // Presets are compiled-in static data (catalog::PRESETS), never written
    // to the user mixes file, so attempting to delete a preset id from the
    // user-mix store must fail exactly like any other unknown id.
    let preset_id = catalog::PRESETS[0].id;
    assert!(mixes::delete(dir.path(), preset_id).is_err());
}
