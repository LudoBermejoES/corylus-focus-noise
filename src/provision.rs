use std::sync::{Arc, Mutex};
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;
use tracing::{info, warn};

use crate::{
    FocusNoiseError, Inner, Result, SoundState,
    state::{self, VersionFile, SCHEMA_VERSION},
};

pub async fn run(
    inner: Arc<Mutex<Inner>>,
    on_progress: impl Fn(SoundState) + Send + 'static,
) -> Result<()> {
    let (bundle, data_dir) = {
        let guard = inner.lock().unwrap();
        if state::is_installed(&guard.data_dir, &guard.bundle) {
            info!("[focus-noise] already installed");
            return Ok(());
        }
        (guard.bundle.clone(), guard.data_dir.clone())
    };

    std::fs::create_dir_all(&data_dir)?;
    let part_path = data_dir.join("bundle.zip.part");
    let ver_path = state::version_path(&data_dir);

    // --- Download ---
    info!("[focus-noise] downloading bundle from {}", bundle.url);
    let client = reqwest::Client::new();
    let resp = client.get(&bundle.url).send().await?;
    let total = resp.content_length().or(Some(bundle.size_bytes));

    set_state(&inner, SoundState::Downloading { downloaded: 0, total });
    on_progress(SoundState::Downloading { downloaded: 0, total });

    let mut file = tokio::fs::File::create(&part_path).await?;
    let mut hasher = Sha256::new();
    let mut downloaded: u64 = 0;

    use futures_util::StreamExt;
    let mut byte_stream = resp.bytes_stream();
    while let Some(chunk) = byte_stream.next().await {
        let chunk = chunk?;
        hasher.update(&chunk);
        downloaded += chunk.len() as u64;
        file.write_all(&chunk).await?;
        let s = SoundState::Downloading { downloaded, total };
        set_state(&inner, s.clone());
        on_progress(s);
    }
    file.flush().await?;
    drop(file);

    // --- Verify checksum before touching any existing sound files ---
    let actual = format!("{:x}", hasher.finalize());
    if actual != bundle.sha256 {
        let _ = std::fs::remove_file(&part_path);
        warn!("[focus-noise] checksum mismatch: expected {} got {}", bundle.sha256, actual);
        let err = FocusNoiseError::ChecksumMismatch { expected: bundle.sha256.clone(), actual };
        set_state(&inner, SoundState::Error { message: err.to_string() });
        return Err(err);
    }
    info!("[focus-noise] checksum ok");

    // --- Extract. On any failure here, the version sidecar is not written,
    // so a subsequent is_installed() check correctly reports not-Ready and a
    // re-install restarts from the download rather than trusting a partial
    // extract. ---
    let extract_result = extract_bundle(&part_path, &data_dir);
    let _ = std::fs::remove_file(&part_path);

    if let Err(e) = extract_result {
        warn!("[focus-noise] extract failed: {e}");
        set_state(&inner, SoundState::Error { message: e.to_string() });
        return Err(e);
    }

    let version = VersionFile { bundle_sha256: bundle.sha256, schema_version: SCHEMA_VERSION };
    std::fs::write(&ver_path, serde_json::to_string_pretty(&version).unwrap())?;

    set_state(&inner, SoundState::Ready);
    on_progress(SoundState::Ready);
    info!("[focus-noise] provision complete");
    Ok(())
}

fn extract_bundle(zip_path: &std::path::Path, dest: &std::path::Path) -> Result<()> {
    let file = std::fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let Some(rel_path) = entry.enclosed_name() else { continue };
        let out_path = dest.join(rel_path);
        if entry.is_dir() {
            std::fs::create_dir_all(&out_path)?;
            continue;
        }
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut out_file = std::fs::File::create(&out_path)?;
        std::io::copy(&mut entry, &mut out_file)?;
    }
    Ok(())
}

fn set_state(inner: &Arc<Mutex<Inner>>, state: SoundState) {
    inner.lock().unwrap().state = state;
}
