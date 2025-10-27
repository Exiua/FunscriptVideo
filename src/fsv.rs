use std::{io::Read, path::Path};

use thiserror::Error;
use tracing::warn;

use crate::{metadata::FsvMetadata, semver::Version};

const LATEST_FSV_FORMAT_VERSION: Version = Version::new(1, 0, 0);
const MINIMUM_FSV_FORMAT_VERSION: Version = Version::new(1, 0, 0);

#[derive(Debug, Error)]
pub enum FsvExtractError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("ZIP archive error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("JSON deserialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("FSV Validation error: {0}")]
    Validation(#[from] FsvValidationError),
    #[error("Metadata file not found in FSV archive")]
    MetadataNotFound,
}

pub fn extract_fsv(path: &Path, output_dir: &Path) -> Result<(), FsvExtractError> {
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let result = archive.by_name("metadata.json");
    let mut metadata_file = match result {
        Ok(file) => file,
        Err(zip_err) => {
            match zip_err {
                zip::result::ZipError::FileNotFound => {
                    return Err(FsvExtractError::MetadataNotFound);
                }
                _ => {
                    return Err(FsvExtractError::Zip(zip_err));
                }
            }
        },
    };

    let mut metadata_json = String::new();
    metadata_file.read_to_string(&mut metadata_json)?;
    let result = serde_json::from_str::<FsvMetadata>(&metadata_json);
    let metadata = match result {
        Ok(metadata) => metadata,
        Err(err) => return Err(FsvExtractError::SerdeJson(err)), // TODO: better error handling
    };

    Ok(())
}

#[derive(Debug, Error)]
pub enum FsvValidationError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("ZIP archive error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("JSON deserialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Metadata file not found in FSV archive")]
    MetadataNotFound,
}

#[derive(Debug, Clone)]
pub enum FsvState {
    Valid,
    ContentIncomplete(ContentIncompleteReason),
    MetadataInvalid(MetadataInvalidReason),
}

#[derive(Debug, Clone, Copy)]
pub enum ContentIncompleteReason {

}

#[derive(Debug, Clone)]
pub enum MetadataInvalidReason {
    InvalidFormatVersion,
    MalformedJson(String),
    UnsupportedFormatVersion(Version),
}

pub fn validate_fsv(path: &Path) -> Result<FsvState, FsvValidationError> {
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let result = archive.by_name("metadata.json");
    let mut metadata_file = match result {
        Ok(file) => file,
        Err(zip_err) => {
            match zip_err {
                zip::result::ZipError::FileNotFound => {
                    return Err(FsvValidationError::MetadataNotFound);
                }
                _ => {
                    return Err(FsvValidationError::Zip(zip_err));
                }
            }
        },
    };

    let mut metadata_json = String::new();
    metadata_file.read_to_string(&mut metadata_json)?;
    let result = serde_json::from_str::<FsvMetadata>(&metadata_json);
    let metadata = match result {
        Ok(metadata) => metadata,
        Err(err) => {
            let err_msg = err.to_string();
            if err_msg.contains("Invalid version format") || err_msg.contains("Invalid number in version") {
                return Ok(FsvState::MetadataInvalid(MetadataInvalidReason::InvalidFormatVersion));
            }
            else {
                return Ok(FsvState::MetadataInvalid(MetadataInvalidReason::MalformedJson(err_msg)));

            }
        },
    };

    if metadata.format_version > LATEST_FSV_FORMAT_VERSION || metadata.format_version < MINIMUM_FSV_FORMAT_VERSION {
        return Ok(FsvState::MetadataInvalid(MetadataInvalidReason::UnsupportedFormatVersion(metadata.format_version)));
    }

    if metadata.title.trim().is_empty() {
        warn!("FSV metadata title is empty");
    }

    

    Ok(FsvState::Valid)
}