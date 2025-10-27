use serde::{Deserialize, Serialize};

use crate::semver::Version;

#[derive(Debug, Serialize, Deserialize)]
pub struct FsvMetadata {
    pub format_version: Version,
    #[serde(default)]
    pub tags: Vec<String>,
    pub title: String,
    pub creators: CreatorsMetadata,
    pub video_formats: Vec<VideoFormat>,
    pub script_variants: Vec<ScriptVariant>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatorsMetadata {
    #[serde(default)]
    pub videos: Vec<WorkCreatorsMetadata>,
    #[serde(default)]
    pub scripts: Vec<WorkCreatorsMetadata>,
    #[serde(default)]
    pub subtitles: Vec<WorkCreatorsMetadata>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkCreatorsMetadata {
    pub work_name: String,
    pub source_url: String,
    pub creator_info: CreatorInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatorInfo {
    pub name: String,
    #[serde(default)]
    pub socials: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoFormat {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub duration_ms: u64,
    #[serde(default)]
    pub start_offset_ms: u64,
    #[serde(default)]
    pub checksum: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScriptVariant {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub additional_axes: Vec<String>,
    #[serde(default)]
    pub duration_ms: u64,
    #[serde(default)]
    pub start_offset_ms: u64,
    #[serde(default)]
    pub checksum: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubtitleTrack {
    pub name: String,
    #[serde(default)]
    pub language: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub duration_ms: u64,
    #[serde(default)]
    pub start_offset_ms: u64,
    #[serde(default)]
    pub checksum: String,
}
