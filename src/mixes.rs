//! User-saved mix persistence. Presets (10 built-in, read-only) live in
//! `catalog::PRESETS` and are never written here — this module only ever
//! touches the user-mixes JSON file.

use std::path::{Path, PathBuf};
use crate::{FocusNoiseError, MixSound, Result};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserMix {
    pub id: String,
    pub name: String,
    pub sounds: Vec<MixSound>,
}

fn mixes_path(data_dir: &Path) -> PathBuf {
    data_dir.join("mixes.json")
}

fn load_all(data_dir: &Path) -> Result<Vec<UserMix>> {
    let path = mixes_path(data_dir);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let data = std::fs::read_to_string(&path)?;
    Ok(serde_json::from_str(&data)?)
}

fn save_all(data_dir: &Path, mixes: &[UserMix]) -> Result<()> {
    std::fs::create_dir_all(data_dir)?;
    let data = serde_json::to_string_pretty(mixes)?;
    std::fs::write(mixes_path(data_dir), data)?;
    Ok(())
}

/// Save (or overwrite, by id) a user mix. Returns the mix's id.
pub fn save(data_dir: &Path, id: Option<String>, name: String, sounds: Vec<MixSound>) -> Result<String> {
    let mut mixes = load_all(data_dir)?;
    let id = id.unwrap_or_else(|| ulid::Ulid::new().to_string());
    if let Some(existing) = mixes.iter_mut().find(|m| m.id == id) {
        existing.name = name;
        existing.sounds = sounds;
    } else {
        mixes.push(UserMix { id: id.clone(), name, sounds });
    }
    save_all(data_dir, &mixes)?;
    Ok(id)
}

pub fn list(data_dir: &Path) -> Result<Vec<UserMix>> {
    load_all(data_dir)
}

pub fn load(data_dir: &Path, id: &str) -> Result<UserMix> {
    load_all(data_dir)?
        .into_iter()
        .find(|m| m.id == id)
        .ok_or_else(|| FocusNoiseError::UnknownMix(id.to_string()))
}

pub fn delete(data_dir: &Path, id: &str) -> Result<()> {
    let mut mixes = load_all(data_dir)?;
    let before = mixes.len();
    mixes.retain(|m| m.id != id);
    if mixes.len() == before {
        return Err(FocusNoiseError::UnknownMix(id.to_string()));
    }
    save_all(data_dir, &mixes)
}
