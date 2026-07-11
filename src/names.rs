//! Localized sound and preset display names. `names.json` ships inside the
//! bundle:
//! `{ "sounds": { "en": { "<id>": "<name>" }, "es": { ... } },
//!    "presets": { "en": { "<id>": "<name>" }, "es": { ... } } }`
//! so translations can be added/edited without a Rust code change — just a
//! new key in the bundle's JSON, republished as a new bundle version.

use std::collections::HashMap;
use std::path::Path;

pub type LangMap = HashMap<String, HashMap<String, String>>;

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct NamesFile {
    #[serde(default)]
    pub sounds: LangMap,
    #[serde(default)]
    pub presets: LangMap,
}

fn names_path(data_dir: &Path) -> std::path::PathBuf {
    data_dir.join("names.json")
}

/// Load the bundle's names.json. Returns an empty `NamesFile` if the module
/// isn't installed yet or the file is missing/unparseable — callers fall back
/// to the raw id in that case (see `resolve`).
pub fn load(data_dir: &Path) -> NamesFile {
    let path = names_path(data_dir);
    let Ok(data) = std::fs::read_to_string(&path) else { return NamesFile::default() };
    serde_json::from_str(&data).unwrap_or_default()
}

/// Resolve an id's display name for `lang` from a single lang-map section,
/// falling back to English, then to the raw id if neither is present (e.g. a
/// translation not yet shipped).
fn resolve(section: &LangMap, lang: &str, id: &str) -> String {
    if let Some(name) = section.get(lang).and_then(|m| m.get(id)) {
        return name.clone();
    }
    if let Some(name) = section.get("en").and_then(|m| m.get(id)) {
        return name.clone();
    }
    id.to_string()
}

pub fn sound_display_name(names: &NamesFile, lang: &str, id: &str) -> String {
    resolve(&names.sounds, lang, id)
}

pub fn preset_display_name(names: &NamesFile, lang: &str, id: &str) -> String {
    resolve(&names.presets, lang, id)
}
