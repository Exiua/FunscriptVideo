use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FsvMetadata {
    pub format_version: String,
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
    pub description: String,
    pub checksum: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScriptVariant {
    pub name: String,
    pub description: String,
    pub additional_axes: Vec<String>,
    pub checksum: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubtitleTrack {
    pub name: String,
    pub language: String,
    pub description: String,
    pub duration_ms: u64,
    pub start_offset_ms: u64,
    pub checksum: String,
}
