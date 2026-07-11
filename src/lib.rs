//! Ambient sound catalog and download engine for Corylus's Focus Noise module.
//!
//! Unlike the per-item engines (`rust-thesaurus`, `rust-languagetool`), this
//! module installs **all-at-once**: one bundle ZIP, one module-level state,
//! not a per-sound state map. See `BUNDLE_MANIFEST.json` in the
//! corylus-focus-noise repo root for the currently-published bundle's URL/SHA.

pub mod catalog;
mod error;
mod mixes;
pub mod names;
mod provision;
mod state;

#[cfg(test)]
mod tests;

pub use catalog::{MixSound, Preset, SoundCatalogEntry, CATALOG, CATEGORIES, PRESETS};
pub use error::FocusNoiseError;
pub use mixes::UserMix;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub type Result<T> = std::result::Result<T, FocusNoiseError>;

/// Location and expected checksum of the published sound bundle. Update
/// after each `corylus-focus-noise` release per its own README/CLAUDE.md.
#[derive(Clone, Debug)]
pub struct BundleInfo {
    pub url: String,
    pub sha256: String,
    pub size_bytes: u64,
}

impl BundleInfo {
    /// Currently-published `sounds-v1.0.2` release — adds `names.json`
    /// (localized sound/preset display names, en/es) inside the bundle
    /// alongside the audio. v1.0.1 was the codec-fix release (see
    /// openspec/changes/add-ambient-sounds-module design.md's audio-playback
    /// spike findings for why v1.0.0 was retracted).
    pub fn default_bundle() -> Self {
        Self {
            url: "https://github.com/LudoBermejoES/corylus-focus-noise/releases/download/sounds-v1.0.2/ambient-sounds-bundle.zip".into(),
            sha256: "19bf044c96ef521b9b7c2356f77e771651726a3a65fc6e79834a275c03353be7".into(),
            size_bytes: 110_000_000,
        }
    }
}

/// Observable state of the module. Module-level, not per-sound — installing
/// downloads and extracts the whole bundle in one step.
#[derive(Clone, Debug, PartialEq)]
pub enum SoundState {
    NotInstalled,
    Downloading { downloaded: u64, total: Option<u64> },
    Ready,
    Error { message: String },
}

pub(crate) struct Inner {
    pub data_dir: PathBuf,
    pub bundle: BundleInfo,
    pub state: SoundState,
    /// Bumped by `uninstall()`. `provision::run` captures this at start and
    /// stops writing state updates once it no longer matches, so an in-flight
    /// download can't resurrect `Downloading`/`Ready` after an uninstall runs
    /// concurrently with it.
    pub epoch: u64,
}

/// The Focus Noise (Ambient Sounds) engine. One instance per app.
#[derive(Clone)]
pub struct FocusNoiseEngine {
    pub(crate) inner: Arc<Mutex<Inner>>,
}

impl FocusNoiseEngine {
    /// `data_dir` is probed immediately via `is_installed`, so callers that
    /// don't yet know the real app-data directory (e.g. Tauri's `unconfigured()`
    /// state constructors, which run before `setup()`) must pass a sentinel
    /// path guaranteed not to exist, then call `set_data_dir` with the real
    /// path before any other method runs. This mirrors the same
    /// `PathBuf::from("__unconfigured__")` convention used by every other
    /// engine state in `src-tauri/app/src/lib.rs`.
    pub fn new(data_dir: PathBuf, bundle: BundleInfo) -> Self {
        let initial_state = if state::is_installed(&data_dir, &bundle) {
            SoundState::Ready
        } else {
            SoundState::NotInstalled
        };
        Self {
            inner: Arc::new(Mutex::new(Inner { data_dir, bundle, state: initial_state, epoch: 0 })),
        }
    }

    pub fn data_dir(&self) -> PathBuf {
        self.inner.lock().unwrap().data_dir.clone()
    }

    /// Replace the data directory and re-probe install state. Call from
    /// `setup()` once the real app-data directory is available.
    pub fn set_data_dir(&self, data_dir: PathBuf) {
        let mut inner = self.inner.lock().unwrap();
        inner.data_dir = data_dir;
        inner.state = if state::is_installed(&inner.data_dir, &inner.bundle) {
            SoundState::Ready
        } else {
            SoundState::NotInstalled
        };
    }

    pub fn state(&self) -> SoundState {
        self.inner.lock().unwrap().state.clone()
    }

    pub fn is_ready(&self) -> bool {
        matches!(self.state(), SoundState::Ready)
    }

    /// Download → verify → extract the whole bundle. Emits `Downloading{..}`
    /// then `Ready` (or `Error`) via the callback.
    pub async fn provision(&self, on_progress: impl Fn(SoundState) + Send + 'static) -> Result<()> {
        provision::run(self.inner.clone(), on_progress).await
    }

    /// Remove all downloaded sound files and reset to `NotInstalled`. Also
    /// bumps the epoch so any install already in flight stops updating state
    /// once it notices, rather than resurrecting `Downloading`/`Ready` after
    /// this uninstall completes.
    pub fn uninstall(&self) -> Result<()> {
        let mut inner = self.inner.lock().unwrap();
        if inner.data_dir.exists() {
            std::fs::remove_dir_all(&inner.data_dir)?;
        }
        inner.state = SoundState::NotInstalled;
        inner.epoch += 1;
        Ok(())
    }

    /// Absolute path to a catalog sound's file, if the module is ready.
    /// Returns `None` when not ready or the id is unknown.
    pub fn sound_path(&self, id: &str) -> Option<PathBuf> {
        if !self.is_ready() {
            return None;
        }
        let entry = catalog::find(id)?;
        Some(self.data_dir().join(entry.relative_path()))
    }

    /// Resolve a sound's display name for `lang` from the bundle's
    /// `names.json`, falling back to English then the raw id. Returns the raw
    /// id for every sound when the module isn't installed yet (no names.json
    /// to read).
    pub fn display_name(&self, id: &str, lang: &str) -> String {
        let names = names::load(&self.data_dir());
        names::sound_display_name(&names, lang, id)
    }

    /// Resolve a built-in preset's display name for `lang`, same fallback
    /// behavior as `display_name`.
    pub fn preset_display_name(&self, id: &str, lang: &str) -> String {
        let names = names::load(&self.data_dir());
        names::preset_display_name(&names, lang, id)
    }

    // ── Mixes ──────────────────────────────────────────────────────────────

    pub fn save_mix(&self, id: Option<String>, name: String, sounds: Vec<MixSound>) -> Result<String> {
        mixes::save(&self.data_dir(), id, name, sounds)
    }

    pub fn list_mixes(&self) -> Result<Vec<UserMix>> {
        mixes::list(&self.data_dir())
    }

    pub fn load_mix(&self, id: &str) -> Result<UserMix> {
        mixes::load(&self.data_dir(), id)
    }

    pub fn delete_mix(&self, id: &str) -> Result<()> {
        mixes::delete(&self.data_dir(), id)
    }
}
