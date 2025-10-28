use std::{path::Path, process::Command, str::FromStr};

use sha2::{Digest, Sha256};
use thiserror::Error;

//const VIDEO_SIG: Map<u64, &'static str> 

pub fn get_hash_string(data: &[u8]) -> String {
    let result = Sha256::digest(data);
    format!("{:x}", result)
}

#[derive(Debug, Error)]
pub enum GetVideoDurationError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse float error: {0}")]
    ParseFloat(#[from] std::num::ParseFloatError),
    #[error("FFprobe error: {0}")]
    Ffprobe(String),
}

/// Get video duration (in seconds) using `ffprobe`.
/// Requires ffprobe to be installed and on PATH.
pub fn get_video_duration<P: AsRef<Path>>(path: P) -> Result<u64, GetVideoDurationError> {
    let output = Command::new("ffprobe")
        .args([
            "-v", "error",
            "-select_streams", "v:0",
            "-show_entries", "format=duration",
            "-of", "default=noprint_wrappers=1:nokey=1",
            path.as_ref().to_str().unwrap(),
        ])
        .output()?;

    if !output.status.success() {
        return Err(GetVideoDurationError::Ffprobe(format!(
            "{}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let trimmed = stdout.trim();

    // Parse seconds (float) -> milliseconds (u64)
    let seconds = f64::from_str(trimmed)?;
    let ms = (seconds * 1000.0).round() as u64;

    Ok(ms)
}

