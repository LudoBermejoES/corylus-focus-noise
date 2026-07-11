use thiserror::Error;

#[derive(Debug, Error)]
pub enum FocusNoiseError {
    #[error("download failed: {0}")]
    Download(#[from] reqwest::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("SHA-256 mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },

    #[error("bundle archive error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("unknown sound id: {0}")]
    UnknownSound(String),

    #[error("unknown mix: {0}")]
    UnknownMix(String),

    #[error("cannot delete a built-in preset")]
    CannotDeletePreset,

    #[error("data directory unavailable")]
    NoDataDir,
}
