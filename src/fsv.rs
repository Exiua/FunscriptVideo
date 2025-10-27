use std::{collections::HashSet, io::Read, path::Path};

use thiserror::Error;
use tracing::{error, warn};

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
    #[error("Invalid state for extraction")]
    InvalidState(FsvState),
}

pub fn extract_fsv(path: &Path, output_dir: &Path, allow_content_incomplete_extract: bool) -> Result<(), FsvExtractError> {
    let fsv_state = validate_fsv(path)?;
    match &fsv_state {
        FsvState::Valid => (),
        FsvState::ContentIncomplete(_) => {
            if !allow_content_incomplete_extract {
                return Err(FsvExtractError::InvalidState(fsv_state));
            }
        },
        FsvState::MetadataInvalid(_) => return Err(FsvExtractError::InvalidState(fsv_state)),
    }

    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let metadata_json = {
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
        
        metadata_json
    };

    let result = serde_json::from_str::<FsvMetadata>(&metadata_json);
    let metadata = match result {
        Ok(metadata) => metadata,
        Err(err) => return Err(FsvExtractError::SerdeJson(err)), // TODO: better error handling
    };

    let output_dirname = metadata.title.trim();
    let output_dirname = if output_dirname.is_empty() {
        path.file_stem()
            .and_then(|os_str| os_str.to_str())
            .unwrap_or("extracted_fsv")
    }
    else {
        output_dirname
    };

    let extraction_path = output_dir.join(output_dirname);
    std::fs::create_dir_all(&extraction_path)?;

    // Create video-script pairs for each combination of video format and script variant
    for video_format in &metadata.video_formats {
        let file_name = video_format.name.trim();
        if file_name.is_empty() {
            warn!("A video format has an empty name, skipping extraction");
            continue;
        }

        // Need to scope to release borrow on archive
        let video_data = {
            let file_in_archive = archive.by_name(file_name);
            let mut file_in_archive = match file_in_archive {
                Ok(file) => file,
                Err(err) => {
                    match err {
                        zip::result::ZipError::Io(_) => {
                            warn!("Unable to read video file '{}', skipping extraction", file_name);
                            continue;
                        },
                        zip::result::ZipError::FileNotFound => {
                            warn!("Video file '{}' not found in archive, skipping extraction", file_name);
                            continue;
                        },
                        zip::result::ZipError::InvalidPassword => {
                            warn!("Video file '{}' is password protected, skipping extraction", file_name);
                            continue;
                        },
                        _ => return Err(FsvExtractError::Zip(err)),
                    }
                },
            };

            let mut buffer = Vec::new();
            let result = file_in_archive.read_to_end(&mut buffer);
            match result {
                Ok(_) => (),
                Err(err) => {
                    warn!("Error reading video file '{}': {}, skipping extraction", file_name, err);
                    continue;
                },
            }

            buffer
        };

        for script_variant in &metadata.script_variants {
            let script_file_name = script_variant.name.trim();
            if script_file_name.is_empty() {
                warn!("A script variant has an empty name, skipping extraction");
                continue;
            }

            let file_in_archive = archive.by_name(script_file_name);
            let mut file_in_archive = match file_in_archive {
                Ok(file) => file,
                Err(err) => {
                    match err {
                        zip::result::ZipError::Io(_) => {
                            warn!("Unable to read script file '{}', skipping extraction", script_file_name);
                            continue;
                        },
                        zip::result::ZipError::FileNotFound => {
                            warn!("Script file '{}' not found in archive, skipping extraction", script_file_name);
                            continue;
                        },
                        zip::result::ZipError::InvalidPassword => {
                            warn!("Script file '{}' is password protected, skipping extraction", script_file_name);
                            continue;
                        },
                        _ => return Err(FsvExtractError::Zip(err)),
                    }
                },
            };

            let script_data = {
                let mut buffer = Vec::new();
                let result = file_in_archive.read_to_end(&mut buffer);
                match result {
                    Ok(_) => (),
                    Err(err) => {
                        warn!("Error reading script file '{}': {}, skipping extraction", script_file_name, err);
                        continue;
                    },
                }

                buffer
            };

            const DEFAULT_VIDEO_EXT: &str = "mp4";
            const DEFAULT_SCRIPT_EXT: &str = "funscript";
            let mut video_parts = file_name.splitn(2, '.');
            let video_stem = video_parts.next().unwrap_or(file_name);
            let video_ext = video_parts.next().unwrap_or(DEFAULT_VIDEO_EXT);

            let mut script_parts = script_file_name.splitn(2, '.');
            let script_stem = script_parts.next().unwrap_or(script_file_name);
            let script_ext = script_parts.next().unwrap_or(DEFAULT_SCRIPT_EXT); // Some scripts may have multiple extensions (e.g., .roll.funscript)

            let output_video_filename = format!("{}_{}.{}", video_stem, script_stem, video_ext);
            let output_script_filename = format!("{}_{}.{}", video_stem, script_stem, script_ext);
            let output_video_path = extraction_path.join(output_video_filename);
            let output_script_path = extraction_path.join(output_script_filename);
            std::fs::write(&output_video_path, &video_data)?;
            std::fs::write(&output_script_path, &script_data)?;
        }
    }

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
    UnableToReadVideo,
    MissingVideoFile,
    VideoPasswordProtected,
    DuplicateVideoFormatEntry,
    UnableToReadScript,
    MissingScriptFile,
    ScriptPasswordProtected,
    DuplicateScriptVariantEntry,
}

#[derive(Debug, Clone)]
pub enum MetadataInvalidReason {
    InvalidFormatVersion,
    MalformedJson(String),
    UnsupportedFormatVersion(Version),
    MissingVideoFormat,
    MissingScriptVariant,
}

pub fn validate_fsv(path: &Path) -> Result<FsvState, FsvValidationError> {
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    // Scope needed to release borrow on archive
    let metadata_json = {
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

        // region Validate metadata.json

        let mut metadata_json = String::new();
        metadata_file.read_to_string(&mut metadata_json)?;

        metadata_json
    };

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

    if metadata.creators.is_empty() {
        warn!("FSV metadata creators information is empty");
    }

    let mut video_present = false; // at least one video format should be present
    for format in &metadata.video_formats {
        if format.name.trim().is_empty() {
            warn!("A video format has an empty name");
        }
        else{
            video_present = true;
        }
    }

    if !video_present {
        return Ok(FsvState::MetadataInvalid(MetadataInvalidReason::MissingVideoFormat));
    }

    let mut script_present = false; // at least one script variant should be present
    for variant in &metadata.script_variants {
        if variant.name.trim().is_empty() {
            warn!("A script variant has an empty name");
        }
        else{
            script_present = true;
        }
    }

    if !script_present {
        return Ok(FsvState::MetadataInvalid(MetadataInvalidReason::MissingScriptVariant));
    }

    // endregion

    // region Validate content files

    let mut seen = HashSet::new();
    for format in &metadata.video_formats {
        let file_name = format.name.trim();
        if file_name.is_empty() {
            // Already warned above
            //warn!("A video format has an empty file name");
            continue;
        }

        if !seen.insert(file_name) {
            error!("Duplicate video format entry found: {}", file_name);
            return Ok(FsvState::ContentIncomplete(ContentIncompleteReason::DuplicateVideoFormatEntry));
        }

        let result = archive.by_name(file_name);
        match result {
            Ok(_) => (),
            Err(err) => {
                match err {
                    zip::result::ZipError::Io(_) => return Ok(FsvState::ContentIncomplete(ContentIncompleteReason::UnableToReadVideo)),
                    zip::result::ZipError::FileNotFound => return Ok(FsvState::ContentIncomplete(ContentIncompleteReason::MissingVideoFile)),
                    zip::result::ZipError::InvalidPassword => return Ok(FsvState::ContentIncomplete(ContentIncompleteReason::VideoPasswordProtected)),
                    _ => return Err(FsvValidationError::Zip(err)),
                }
            },
        }

        // TODO: Validate duration and checksums if present
    }

    let mut seen = HashSet::new();
    for variant in &metadata.script_variants {
        let file_name = variant.name.trim();
        if file_name.is_empty() {
            // Already warned above
            //warn!("A script variant has an empty file name");
            continue;
        }

        if !seen.insert(file_name) {
            error!("Duplicate script variant entry found: {}", file_name);
            return Ok(FsvState::ContentIncomplete(ContentIncompleteReason::DuplicateScriptVariantEntry));
        }

        let result = archive.by_name(file_name);
        match result {
            Ok(_) => (),
            Err(err) => {
                match err {
                    zip::result::ZipError::Io(_) => return Ok(FsvState::ContentIncomplete(ContentIncompleteReason::UnableToReadScript)),
                    zip::result::ZipError::FileNotFound => return Ok(FsvState::ContentIncomplete(ContentIncompleteReason::MissingScriptFile)),
                    zip::result::ZipError::InvalidPassword => return Ok(FsvState::ContentIncomplete(ContentIncompleteReason::ScriptPasswordProtected)),
                    _ => return Err(FsvValidationError::Zip(err)),
                }
            },
        }

        // TODO: Validate duration and checksums if present
    }

    // endregion

    Ok(FsvState::Valid)
}

#[derive(Debug, Error)]
pub enum FsvCreateError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("ZIP archive error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("JSON serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
}