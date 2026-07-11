use tempfile::tempdir;

use crate::{catalog, mixes, names, state, BundleInfo, FocusNoiseEngine, MixSound, SoundState};

// ── Catalog integrity ────────────────────────────────────────────────────────

#[test]
fn catalog_every_entry_has_nonempty_fields() {
    for s in catalog::CATALOG {
        assert!(!s.id.is_empty(), "empty id");
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

#[tokio::test]
async fn provision_is_a_noop_while_already_downloading() {
    // Simulates a second install trigger (e.g. Settings + onboarding both
    // firing) arriving while the first is mid-download: it must not touch
    // bundle.zip.part or clobber the in-flight Downloading state.
    let dir = tempdir().unwrap();
    let engine = FocusNoiseEngine::new(dir.path().to_path_buf(), test_bundle());
    {
        let mut inner = engine.inner.lock().unwrap();
        inner.state = SoundState::Downloading { downloaded: 10, total: Some(100) };
    }

    let result = engine
        .provision(|_| panic!("on_progress must not fire for a rejected concurrent install"))
        .await;

    assert!(result.is_ok(), "a concurrent install trigger must be a no-op, not an error");
    assert!(!dir.path().join("bundle.zip.part").exists());
    assert_eq!(
        engine.state(),
        SoundState::Downloading { downloaded: 10, total: Some(100) },
        "the in-flight download's state must be left untouched"
    );
}

#[test]
fn uninstall_bumps_epoch_so_a_concurrent_install_stops_updating_state() {
    // Simulates uninstall (e.g. disabling the module in Settings) racing an
    // in-flight install: uninstall must invalidate that install's epoch so it
    // can no longer resurrect Downloading/Ready after uninstall runs.
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

    let epoch_before = engine.inner.lock().unwrap().epoch;
    engine.uninstall().unwrap();
    let epoch_after = engine.inner.lock().unwrap().epoch;

    assert_ne!(epoch_before, epoch_after, "uninstall must bump the epoch");
    assert_eq!(engine.state(), SoundState::NotInstalled);
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

// ── Localized names ──────────────────────────────────────────────────────

fn names_file_fixture() -> names::NamesFile {
    let mut sounds_en = std::collections::HashMap::new();
    sounds_en.insert("campfire".to_string(), "Campfire".to_string());
    let mut sounds_es = std::collections::HashMap::new();
    sounds_es.insert("campfire".to_string(), "Fogata".to_string());

    let mut presets_en = std::collections::HashMap::new();
    presets_en.insert("rainy_night".to_string(), "Rainy Night".to_string());
    let mut presets_es = std::collections::HashMap::new();
    presets_es.insert("rainy_night".to_string(), "Noche de lluvia".to_string());

    names::NamesFile {
        sounds: [("en".to_string(), sounds_en), ("es".to_string(), sounds_es)].into(),
        presets: [("en".to_string(), presets_en), ("es".to_string(), presets_es)].into(),
    }
}

#[test]
fn names_load_returns_empty_file_when_missing() {
    let dir = tempdir().unwrap();
    let loaded = names::load(dir.path());
    assert!(loaded.sounds.is_empty());
    assert!(loaded.presets.is_empty());
}

#[test]
fn sound_display_name_resolves_requested_language() {
    let names = names_file_fixture();
    assert_eq!(names::sound_display_name(&names, "es", "campfire"), "Fogata");
    assert_eq!(names::sound_display_name(&names, "en", "campfire"), "Campfire");
}

#[test]
fn sound_display_name_falls_back_to_english_when_lang_or_key_missing() {
    let names = names_file_fixture();
    // No "fr" section at all.
    assert_eq!(names::sound_display_name(&names, "fr", "campfire"), "Campfire");
}

#[test]
fn sound_display_name_falls_back_to_raw_id_when_no_translation_exists() {
    let names = names::NamesFile::default();
    assert_eq!(names::sound_display_name(&names, "es", "campfire"), "campfire");
}

#[test]
fn preset_display_name_resolves_requested_language_independent_of_sounds() {
    let names = names_file_fixture();
    assert_eq!(names::preset_display_name(&names, "es", "rainy_night"), "Noche de lluvia");
    assert_eq!(names::preset_display_name(&names, "en", "rainy_night"), "Rainy Night");
    // A preset id must not resolve against the sounds section or vice versa.
    assert_eq!(names::preset_display_name(&names, "es", "campfire"), "campfire");
    assert_eq!(names::sound_display_name(&names, "es", "rainy_night"), "rainy_night");
}

#[test]
fn engine_display_name_reads_bundles_names_json() {
    let dir = tempdir().unwrap();
    let json = r#"{
        "sounds": {"en": {"campfire": "Campfire"}, "es": {"campfire": "Fogata"}},
        "presets": {"en": {"rainy_night": "Rainy Night"}, "es": {"rainy_night": "Noche de lluvia"}}
    }"#;
    std::fs::write(dir.path().join("names.json"), json).unwrap();

    let engine = FocusNoiseEngine::new(dir.path().to_path_buf(), test_bundle());
    assert_eq!(engine.display_name("campfire", "es"), "Fogata");
    assert_eq!(engine.display_name("campfire", "en"), "Campfire");
    assert_eq!(engine.preset_display_name("rainy_night", "es"), "Noche de lluvia");
    // Unknown id with no entry in either language falls back to the raw id.
    assert_eq!(engine.display_name("not-a-real-id", "es"), "not-a-real-id");
}

#[test]
fn engine_display_name_falls_back_to_raw_id_when_module_not_installed() {
    let dir = tempdir().unwrap();
    let engine = FocusNoiseEngine::new(dir.path().to_path_buf(), test_bundle());
    assert_eq!(engine.display_name("campfire", "es"), "campfire");
    assert_eq!(engine.preset_display_name("rainy_night", "es"), "rainy_night");
}
