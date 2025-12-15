use std::{collections::HashSet, fs::File, io::{Read, Write}, path::{Path, PathBuf}};

use clap::ValueEnum;
use thiserror::Error;
use tracing::{error, info, warn};
use zip::write::SimpleFileOptions;

use crate::{db_client::{self, DbClient}, file_util, funscript::Funscript, metadata::{CreatorInfo, FsvMetadata, ScriptVariant, SubtitleTrack, VideoFormat, WorkCreatorsMetadata, WorkItem}, semver::Version};

const LATEST_FSV_FORMAT_VERSION: Version = Version::new(1, 0, 0);
const MINIMUM_FSV_FORMAT_VERSION: Version = Version::new(1, 0, 0);
const AXES: [&str; 11] = ["pitch", "roll", "suckManual", "surge", "sway", "twist", "valve", "vib", "lube", "suck", "max"]; // TODO: Check if there are more axes in use

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
    UnableToReadItem(ItemType),
    MissingItemFile(ItemType),
    ItemPasswordProtected(ItemType),
    DuplicateItemEntry(ItemType),
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

    let state = validate_item_contents(ItemType::Video, &metadata.video_formats, &mut archive)?;
    if !matches!(state, FsvState::Valid) {
        return Ok(state);
    }

    let state = validate_item_contents(ItemType::Script, &metadata.script_variants, &mut archive)?;
    if !matches!(state, FsvState::Valid) {
        return Ok(state);
    }

    let state = validate_item_contents(ItemType::Subtitle, &metadata.subtitle_tracks, &mut archive)?;
    if !matches!(state, FsvState::Valid) {
        return Ok(state);
    }

    // endregion

    Ok(FsvState::Valid)
}

fn validate_item_contents<Item: WorkItem>(item_type: ItemType, items: &Vec<Item>, archive: &mut zip::ZipArchive<std::fs::File>) -> Result<FsvState, FsvValidationError> {
    // TODO: Maybe add Func for specific item validations
    // TODO: Maybe improve return value to not be confused with caller's return value (mainly since FsvState::Valid doesn't make sense when a different item type may be invalid)
    let mut seen = HashSet::new();
    for item in items {
        let file_name = item.get_name().trim();
        if file_name.is_empty() {
            warn!("A subtitle track has an empty file name");
            continue;
        }

        if !seen.insert(file_name) {
            warn!("Duplicate subtitle track entry found: {}", file_name);
        }

        let result = archive.by_name(file_name);
        match result {
            Ok(_) => (),
            Err(err) => {
                match err {
                    zip::result::ZipError::Io(_) => return Ok(FsvState::ContentIncomplete(ContentIncompleteReason::UnableToReadItem(item_type))),
                    zip::result::ZipError::FileNotFound => return Ok(FsvState::ContentIncomplete(ContentIncompleteReason::MissingItemFile(item_type))),
                    zip::result::ZipError::InvalidPassword => return Ok(FsvState::ContentIncomplete(ContentIncompleteReason::ItemPasswordProtected(item_type))),
                    _ => return Err(FsvValidationError::Zip(err)),
                }
            },
        }
    }

    Ok(FsvState::Valid)
}

#[derive(Debug, Error)]
pub enum FsvCreateError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("ZIP archive error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("Serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("From UTF-8 error: {0}")]
    FromUtf8(#[from] std::string::FromUtf8Error),
    #[error("Database client error: {0}")]
    DbClient(#[from] db_client::DbClientError),
    #[error("FSV error: {0}")]
    Fsv(#[from] FsvError),
    #[error("Get duration error: {0}")]
    GetDurationError(#[from] file_util::GetDurationError),
    #[error("FSV already exists at path: {0}")]
    FsvAlreadyExists(PathBuf),
    #[error("Creator info for {0} not found for key: {1}")]
    CreatorInfoNotFound(ItemType, String),
}

#[derive(Debug)]
pub struct CreateArgs {
    pub path: PathBuf,
    pub title: String,
    pub tags: Vec<String>,
    pub video: Option<PathBuf>,
    pub script: Option<PathBuf>,
    pub video_creator_key: Option<String>,
    pub script_creator_key: Option<String>,
}

impl CreateArgs {
    pub fn new(path: PathBuf, title: String, tags: Vec<String>, video: Option<PathBuf>, script: Option<PathBuf>, video_creator_key: Option<String>, script_creator_key: Option<String>) -> Self {
        CreateArgs {
            path,
            title,
            tags,
            video,
            script,
            video_creator_key,
            script_creator_key,
        }
    }
}

pub async fn create_fsv(args: CreateArgs, db_client: &DbClient, interactive: bool) -> Result<(), FsvCreateError> {
    let CreateArgs { path, title, tags, video, script, video_creator_key, script_creator_key } = args;
    // Create file but don't overwrite if it exists
    let result = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path);
    let file = match result {
        Ok(file) => file,
        Err(err) => match err.kind() {
            std::io::ErrorKind::AlreadyExists => return Err(FsvCreateError::FsvAlreadyExists(path)),
            _ => return Err(FsvCreateError::Io(err)),
        },
    };

    let result = create_inner(file, title, tags, video, script, video_creator_key, script_creator_key, db_client, interactive).await;
    match result {
        Ok(_) => Ok(()),
        Err(err) => {
            // Clean up by removing the created file
            if let Err(remove_err) = std::fs::remove_file(&path) {
                error!("Error removing incomplete FSV file at '{}': {}", path.display(), remove_err);
            }

            Err(err)
        }
    }
}

// Providing the creator without the accompanying file path will silently skip adding the creator info (e.g., providing a video creator without a video file)
async fn create_inner(file: File, title: String, tags: Vec<String>, video: Option<PathBuf>, script: Option<PathBuf>, video_creator_key: Option<String>, script_creator_key: Option<String>, db_client: &DbClient, interactive: bool) -> Result<(), FsvCreateError> {
    let mut metadata = FsvMetadata::new(LATEST_FSV_FORMAT_VERSION);
    metadata.title = title;
    metadata.tags = tags;

    let mut add_files = Vec::new();
    // _filename and _path variables are needed to keep the PathBuf alive while being used in AddFile, do not access them directly
    let video_filename;
    let video_path;
    let mut video_added = false;
    if let Some(video) = video {
        video_path = video;
        let video_creator_key = get_creator_info_from_key(&db_client, video_creator_key.as_deref(), interactive).await?;
        video_filename = video_path.file_name().and_then(|f| f.to_str()).unwrap_or("video.mp4").to_string();
        let video_duration = file_util::get_video_duration(&video_path)?;
        let content = std::fs::read(&video_path)?;
        let hash = get_file_hash(&content);
        if let Some(creator_info) = video_creator_key {
            let work_info = WorkCreatorsMetadata::new(video_filename.clone(), String::new(), creator_info);
            metadata.add_video_creator(work_info);
        }

        let video_format = VideoFormat::new(video_filename.clone(), String::new(), video_duration, hash);
        metadata.add_video_format(video_format);
        let add_file = AddFile::new(&video_filename, &video_path);
        video_added = true;
        add_files.push(add_file);
    }

    let script_filename;
    let script_path;
    let mut script_added = false;
    if let Some(script) = script {
        script_path = script;
        let script_creator_key = get_creator_info_from_key(&db_client, script_creator_key.as_deref(), interactive).await?;
        script_filename = script_path.file_name().and_then(|f| f.to_str()).unwrap_or("script.funscript").to_string();
        let content = std::fs::read(&script_path)?;
        let hash = get_file_hash(&content);
        let file_content = String::from_utf8(content)?;
        let funscript = serde_json::from_str::<Funscript>(&file_content)?;
        let script_duration = file_util::get_funscript_duration(&funscript)?;
        if let Some(creator_info) = script_creator_key {
            let work_info = WorkCreatorsMetadata::new(script_filename.to_string(), String::new(), creator_info);
            metadata.add_script_creator(work_info);
        }

        let script_variant = ScriptVariant::new(script_filename.to_string(), String::new(), vec![], script_duration, 0, hash);
        metadata.add_script_variant(script_variant);
        let add_file = AddFile::new(&script_filename, &script_path);
        script_added = true;
        add_files.push(add_file);
    }

    match (video_added, script_added) {
        (true, true) => (),
        (true, false) => warn!("No script provided for FSV creation, creating incomplete FSV"),
        (false, true) => warn!("No video provided for FSV creation, creating incomplete FSV"),
        (false, false) => warn!("No video or script provided for FSV creation, creating incomplete FSV"),
    }

    build_archive(file, &metadata, add_files)?;
    
    Ok(())
}

#[derive(Debug, Error)]
pub enum FsvAddError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("ZIP archive error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("Serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Database client error: {0}")]
    DbClient(#[from] db_client::DbClientError),
    #[error("FSV error: {0}")]
    Fsv(#[from] FsvError),
    #[error("Get video duration error: {0}")]
    GetVideoDuration(#[from] file_util::GetDurationError),
    #[error("Unable to get file name from path: {0}")]
    UnableToGetFileName(std::path::PathBuf),
    #[error("Creator info not found for key: {0}")]
    CreatorInfoNotFound(String),
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ItemType {
    Video,
    Script,
    Subtitle,
}

impl ItemType {
    pub fn get_name(&self) -> &str {
        match self {
            ItemType::Video => "Video",
            ItemType::Script => "Script",
            ItemType::Subtitle => "Subtitle",
        }
    }

    pub fn get_name_lower(&self) -> &str {
        match self {
            ItemType::Video => "video",
            ItemType::Script => "script",
            ItemType::Subtitle => "subtitle",
        }
    }
}

impl std::fmt::Display for ItemType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_name())
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum EntryType {
    Creator,
    Video,
    Script,
    Subtitle,
}

impl EntryType {
    pub fn get_name(&self) -> &str {
        match self {
            EntryType::Creator => "Creator",
            EntryType::Video => "Video",
            EntryType::Script => "Script",
            EntryType::Subtitle => "Subtitle",
        }
    }
}

#[derive(Debug)]
pub struct AddArgs {
    path: PathBuf,
    item_type: ItemType,
    item_path: PathBuf,
    creator_key: Option<String>,
}

impl AddArgs {
    pub fn new(path: PathBuf, item_type: ItemType, item_path: PathBuf, creator_key: Option<String>) -> Self {
        AddArgs {
            path,
            item_type,
            item_path,
            creator_key,
        }
    }
}

pub async fn add_to_fsv(args: AddArgs, db_client: &DbClient, interactive: bool) -> Result<(), FsvAddError> {
    let AddArgs { path, item_type, item_path, creator_key } = args;
    let filname = item_path.file_name().and_then(|f| f.to_str()).ok_or_else(|| FsvAddError::UnableToGetFileName(item_path.to_path_buf()))?;
    let content = std::fs::read(&item_path)?;
    let hash = get_file_hash(&content);
    let creator_info = get_creator_info_from_key(&db_client, creator_key.as_deref(), interactive).await?;

    let (archive, mut metadata) = open_fsv(&path)?;
    match item_type {
        ItemType::Video => {
            for format in &metadata.video_formats {
                if format.name == filname {
                    warn!("Video format '{}' already exists in FSV, skipping addition", filname);
                    return Ok(());
                }
            }
            
            // TODO: Add validation for video format (duration, checksum, etc.)

            let video_duration = file_util::get_video_duration(&item_path)?;
            if let Some(creator_info) = creator_info {
                let work_info = WorkCreatorsMetadata::new(filname.to_string(), String::new(), creator_info);
                metadata.add_video_creator(work_info);
            }

            let video_format = VideoFormat::new(filname.to_string(), String::new(), video_duration, hash);
            metadata.add_video_format(video_format);
            let add_file = AddFile::new(filname, &item_path);
            rebuild_archive(&path, archive, &metadata, vec![add_file], vec![])?;
        },
        ItemType::Script => {
            for variant in &metadata.script_variants {
                if variant.name == filname {
                    warn!("Script variant '{}' already exists in FSV, skipping addition", filname);
                    return Ok(());
                }
            }

            let file_content = std::fs::read_to_string(&path)?;
            let funscript = serde_json::from_str::<Funscript>(&file_content)?; // validates funscript structure
            let script_duration = file_util::get_funscript_duration(&funscript)?;
            if let Some(creator_info) = creator_info {
                let work_info = WorkCreatorsMetadata::new(filname.to_string(), String::new(), creator_info);
                metadata.add_script_creator(work_info);
            }

            let script_variant = ScriptVariant::new(filname.to_string(), String::new(), vec![], script_duration, 0, hash);
            metadata.add_script_variant(script_variant);
            let add_file = AddFile::new(filname, &item_path);
            rebuild_archive(&path, archive, &metadata, vec![add_file], vec![])?;
        },
        ItemType::Subtitle => {
            for track in &metadata.subtitle_tracks {
                if track.name == filname {
                    warn!("Subtitle track '{}' already exists in FSV, skipping addition", filname);
                    return Ok(());
                }
            }

            // TODO: Add validation for subtitle track (checksum, etc.)

            if let Some(creator_info) = creator_info {
                let work_info = WorkCreatorsMetadata::new(filname.to_string(), String::new(), creator_info);
                metadata.add_subtitle_creator(work_info);
            }

            let subtitle_track = SubtitleTrack::new(filname.to_string(), String::new(), String::new(), hash);
            metadata.add_subtitle_track(subtitle_track);
            let add_file = AddFile::new(filname, &item_path);
            rebuild_archive(&path, archive, &metadata, vec![add_file], vec![])?;
        },
    }

    Ok(())
}

pub async fn add_creator_to_fsv(fsv_path: &Path, work_type: ItemType, creator_key: &str, work_name: &str, source_url: &str, db_client: &DbClient) -> Result<(), FsvAddError> {
    let (archive, mut metadata) = open_fsv(fsv_path)?;
    let creator_info = db_client.get_creator_info_by_key(creator_key).await?;
    let creator_info = match creator_info {
        Some(info) => info,
        None => return Err(FsvAddError::CreatorInfoNotFound(creator_key.to_string())),
    };

    let work_info = WorkCreatorsMetadata::new(work_name.to_string(), source_url.to_string(), creator_info);
    match work_type {
        ItemType::Video => metadata.add_video_creator(work_info),
        ItemType::Script => metadata.add_script_creator(work_info),
        ItemType::Subtitle => metadata.add_subtitle_creator(work_info),
    }

    rebuild_archive(fsv_path, archive, &metadata, vec![], vec![])?;
    
    Ok(())
}

#[derive(Debug, Error)]
pub enum FsvRemoveError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("ZIP archive error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("Serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Database client error: {0}")]
    DbClient(#[from] db_client::DbClientError),
    #[error("FSV error: {0}")]
    Fsv(#[from] FsvError),
    #[error("Entry not found: {0}")]
    EntryNotFound(String),
}

pub fn remove_from_fsv(path: &Path, entry_type: EntryType, entry_id: &str) -> Result<(), FsvRemoveError> {
    let (archive, mut metadata) = open_fsv(path)?;
    match entry_type {
        EntryType::Creator => {
            let mut found = false;
            metadata.creators.retain(|creator| {
                if creator.work_name == entry_id {
                    found = true;
                    false
                }
                else {
                    true
                }
            });

            if !found {
                return Err(FsvRemoveError::EntryNotFound(entry_id.to_string()));
            }

            rebuild_archive(path, archive, &metadata, vec![], vec![])?;
        },
        EntryType::Video => {
            let mut found = false;
            metadata.video_formats.retain(|format| {
                if format.name == entry_id {
                    found = true;
                    false
                }
                else {
                    true
                }
            });

            if !found {
                return Err(FsvRemoveError::EntryNotFound(entry_id.to_string()));
            }

            let remove_files = vec![entry_id];
            rebuild_archive(path, archive, &metadata, vec![], remove_files)?;
        },
        EntryType::Script => {
            let mut parts = entry_id.splitn(2, '.');
            let stem = parts.next().unwrap_or(entry_id);
            let ext = parts.next().unwrap_or("funscript"); // Some scripts may have multiple extensions (e.g., .roll.funscript)
            let scripts = if ext != "funscript" { // If specific axis was provided, only remove that one
                vec![entry_id.to_string()]
            }
            else {  // Else remove all axis variants in addition to the base script
                let scripts = AXES.iter().map(|axis| format!("{}.{}.{}", stem, axis, ext));
                std::iter::once(entry_id.to_string()).chain(scripts).collect()
            };

            let mut found = false;
            metadata.script_variants.retain(|variant| {
                if scripts.contains(&variant.name) {
                    found = true;
                    false
                }
                else {
                    true
                }
            });

            if !found {
                return Err(FsvRemoveError::EntryNotFound(entry_id.to_string()));
            }

            let remove_files = scripts.iter().map(|s| s.as_str()).collect();
            rebuild_archive(path, archive, &metadata, vec![], remove_files)?;
        },
        EntryType::Subtitle => {
            let mut found = false;
            metadata.subtitle_tracks.retain(|track| {
                if track.name == entry_id {
                    found = true;
                    false
                }
                else {
                    true
                }
            });

            if !found {
                return Err(FsvRemoveError::EntryNotFound(entry_id.to_string()));
            }

            let remove_files = vec![entry_id];
            rebuild_archive(path, archive, &metadata, vec![], remove_files)?;
        },
    }

    Ok(())
}

pub async fn remove_creator_from_db(creator_key: &str, db_client: &DbClient) -> Result<(), FsvRemoveError> {
    db_client.delete_creator_info_by_key(creator_key).await?;
    Ok(())
}

#[derive(Debug, Error)]
pub enum FsvRebuildError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("ZIP archive error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("Serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Database client error: {0}")]
    DbClient(#[from] db_client::DbClientError),
    #[error("FSV error: {0}")]
    Fsv(#[from] FsvError),
}

/// Rebuild the FSV archive without any changes. This ensures that the only files present are those listed in the central directory of the ZIP archive.
pub fn rebuild_fsv(path: &Path) -> Result<(), FsvRebuildError> {
    let (archive, metadata) = open_fsv(path)?;
    rebuild_archive(path, archive, &metadata, vec![], vec![])?;

    Ok(())
}

#[derive(Debug)]
pub struct FsvInfo {
    // Define fields to hold information about the FSV file
    pub title: String,
    pub videos: Vec<(String, bool)>, // (filename, is_present)
    pub scripts: Vec<(String, bool)>, // (filename, is_present)
    pub subtitles: Vec<(String, bool)>, // (filename, is_present)
    pub extra_files: Vec<String>,
}

impl FsvInfo {
    fn new(title: String, videos: Vec<(String, bool)>, scripts: Vec<(String, bool)>, subtitles: Vec<(String, bool)>, extra_files: Vec<String>) -> Self {
        FsvInfo { title, videos, scripts, subtitles, extra_files }
    }
}

// TODO: Add parameter for extracting other info such as creators, tags, etc.
pub fn get_fsv_info(path: &Path) -> Result<FsvInfo, FsvError> {
    let (mut archive, metadata) = open_fsv(path)?;
    let title = if metadata.title.trim().is_empty() {
        path.file_stem()
            .and_then(|os_str| os_str.to_str())
            .unwrap_or("unknown")
            .to_string()
    }
    else{
        metadata.title.to_string()
    };

    let mut seen_files = HashSet::new();
    let mut videos = Vec::new();
    for video in &metadata.video_formats {
        let is_present = archive.by_name(&video.name).is_ok();
        videos.push((video.name.to_string(), is_present));
        seen_files.insert(video.name.to_string());
    }

    let mut scripts = Vec::new();
    for variant in &metadata.script_variants {
        let is_present = archive.by_name(&variant.name).is_ok();
        scripts.push((variant.name.to_string(), is_present));
        seen_files.insert(variant.name.to_string());
    }

    let mut subtitles = Vec::new();
    for track in &metadata.subtitle_tracks {
        let is_present = archive.by_name(&track.name).is_ok();
        subtitles.push((track.name.to_string(), is_present));
        seen_files.insert(track.name.to_string());
    }

    let mut extra_files = Vec::new();
    for i in 0..archive.len() {
        let file = archive.by_index(i)?;
        let file_name = file.name();
        if !seen_files.contains(file_name) {
            extra_files.push(file_name.to_string());
        }
    }
    
    Ok(FsvInfo::new(title, videos, scripts, subtitles, extra_files))
}

#[derive(Debug, Error)]
pub enum FsvError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("ZIP archive error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("JSON deserialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Database client error: {0}")]
    DbClient(#[from] db_client::DbClientError),
    #[error("Metadata file not found in FSV archive")]
    MetadataFileNotFound,
    #[error("Creator info not found for key: {0}")]
    CreatorInfoNotFound(String),
}

#[derive(Debug)]
pub struct AddFile<'a> {
    pub name: &'a str,
    pub path: &'a Path,
}

impl<'a> AddFile<'a> {
    pub fn new(name: &'a str, path: &'a Path) -> Self {
        AddFile { name, path }
    }
}

fn build_archive(file: File, metadata: &FsvMetadata, add_files: Vec<AddFile>) -> Result<(), FsvError> {
    let mut zip_writer = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Bzip2);
    // Write metadata first
    let metadata_json = serde_json::to_string_pretty(metadata)?;
    zip_writer.start_file("metadata.json", options)?;
    zip_writer.write_all(metadata_json.as_bytes())?;

    // Add files
    for file_path in add_files {
        let mut file = std::fs::File::open(file_path.path)?;
        zip_writer.start_file(file_path.name, options)?;
        std::io::copy(&mut file, &mut zip_writer)?;
    }
    
    zip_writer.finish()?.flush()?;

    Ok(())
}

/// Rebuild the FSV archive with updated metadata and added/removed files (metadata is assumed to already have added/removed the relevant entries)
fn rebuild_archive(archive_path: &Path, mut archive: zip::ZipArchive<std::fs::File>, metadata: &FsvMetadata, add_files: Vec<AddFile>, remove_files: Vec<&str>) -> Result<(), FsvError> {
    let temp_path = archive_path.with_extension("tmp");
    let temp_file = std::fs::File::create(&temp_path)?;
    let mut zip_writer = zip::ZipWriter::new(temp_file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Bzip2);
    // Write updated metadata.json
    let metadata_json = serde_json::to_string_pretty(metadata)?;
    zip_writer.start_file("metadata.json", options)?;
    zip_writer.write_all(metadata_json.as_bytes())?;
    // Copy existing files, skipping removed files
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let file_name = file.name();
        if file_name == "metadata.json" || remove_files.contains(&file_name) {
            continue; // skip metadata.json (already written) and removed files
        }
        zip_writer.start_file(file_name, options)?;
        std::io::copy(&mut file, &mut zip_writer)?;
    }

    // Add new files
    for file_path in add_files {
        let mut file = std::fs::File::open(file_path.path)?;
        zip_writer.start_file(file_path.name, options)?;
        std::io::copy(&mut file, &mut zip_writer)?;
    }

    zip_writer.finish()?.flush()?;
    drop(archive);
    std::fs::rename(temp_path, archive_path)?;

    Ok(())
}

fn open_fsv(path: &Path) -> Result<(zip::ZipArchive<std::fs::File>, FsvMetadata), FsvError> {
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let metadata_json = {
        let result = archive.by_name("metadata.json");
        let mut metadata_file = match result {
            Ok(file) => file,
            Err(zip_err) => {
                match zip_err {
                    zip::result::ZipError::FileNotFound => {
                        return Err(FsvError::MetadataFileNotFound);
                    }
                    _ => {
                        return Err(FsvError::Zip(zip_err));
                    }
                }
            },
        };
        let mut metadata_json = String::new();
        metadata_file.read_to_string(&mut metadata_json)?;

        metadata_json
    };

    let metadata = serde_json::from_str::<FsvMetadata>(&metadata_json)?;

    Ok((archive, metadata))
}

/// Prompt the user and return trimmed input
fn prompt_input(prompt: &str) -> std::io::Result<String> {
    print!("{}", prompt);
    std::io::stdout().flush()?; // make sure the prompt appears immediately
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf)?;
    Ok(buf.trim().to_string())
}

pub async fn get_creator_info_from_key(db_client: &DbClient, creator_key: Option<&str>, interactive: bool) -> Result<Option<CreatorInfo>, FsvError> {
    if let Some(key) = creator_key {
        let creator_info = db_client.get_creator_info_by_key(&key).await?;
        if let Some(creator_info) = creator_info {
            Ok(Some(creator_info))
        }
        else if interactive {
            warn!("Creator with key '{}' not found in database; entering interactive mode.", key);
            let creator_info = get_creator_info_from_user(db_client, Some(&key)).await?;
            Ok(Some(creator_info))
        }
        else{
            Err(FsvError::CreatorInfoNotFound(key.to_string()))
        }
    }
    else {
        Ok(None)
    }
}

pub async fn get_creator_info_from_user(db_client: &DbClient, creator_key: Option<&str>) -> Result<CreatorInfo, FsvError> {
    // Name (required)
    let name = loop {
        let input = prompt_input("Enter creator name: ")?;
        if input.is_empty() {
            println!("Name cannot be empty. Please try again.");
        } else {
            break input;
        }
    };

    // Socials (comma-separated)
    let socials_input = prompt_input("Enter creator socials (comma-separated): ")?;
    let socials: Vec<String> = socials_input
        .split(',')
        .filter_map(|s| {
            let trimmed = s.trim();
            if trimmed.is_empty() { None } else { Some(trimmed.to_string()) }
        })
        .collect();

    let creator_info = CreatorInfo::new(name, socials);

    // Needed to resolve lifetime issues in else branch
    let input_key;
    // Save to DB if key provided or in interactive mode
    let key = if let Some(key) = creator_key {
        info!("Saving creator info with key '{}' to database.", key);
        key
    }
    else{
        // Optional DB save
        input_key = prompt_input("Enter creator key (leave blank to skip saving to DB): ")?;
        &input_key
    };

    if !key.is_empty() {
        match db_client.insert_creator_info(&key, &creator_info).await {
            Ok(_) => info!("Creator '{}' saved to database.", key),
            Err(e) => error!("Failed to insert creator info: {}", e),
        }
    }

    Ok(creator_info)
}

pub fn get_file_hash(data: &[u8]) -> String {
    let hash = file_util::get_hash_string(data);
    format!("sha256:{}", hash)
}