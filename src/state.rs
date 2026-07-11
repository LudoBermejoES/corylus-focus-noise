use std::path::PathBuf;
use crate::BundleInfo;

pub fn version_path(data_dir: &std::path::Path) -> PathBuf {
    data_dir.join(".version.json")
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct VersionFile {
    pub bundle_sha256: String,
    pub schema_version: u32,
}

pub const SCHEMA_VERSION: u32 = 1;

/// The module is considered installed only when the version sidecar exists,
/// matches the currently-expected bundle SHA-256, and every catalog sound's
/// file is actually present on disk (guards against a partially-deleted or
/// hand-tampered data dir reporting `Ready` incorrectly).
pub fn is_installed(data_dir: &std::path::Path, bundle: &BundleInfo) -> bool {
    let ver = version_path(data_dir);
    if !ver.exists() {
        return false;
    }
    let Ok(data) = std::fs::read_to_string(&ver) else { return false; };
    let Ok(v) = serde_json::from_str::<VersionFile>(&data) else { return false; };
    if v.bundle_sha256 != bundle.sha256 || v.schema_version != SCHEMA_VERSION {
        return false;
    }
    crate::catalog::CATALOG
        .iter()
        .all(|s| data_dir.join(s.relative_path()).exists())
}
