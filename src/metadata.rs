use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::semver::Version;

/// The root FSV metadata object.
#[derive(Debug, Serialize, Deserialize)]
pub struct FsvMetadata {
    pub format_version: Version,
    #[serde(default)]
    pub extensions: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    // Optional in spec, but MUST NOT be null -> use empty string as "missing"
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub creators: CreatorsMetadata,
    pub video_formats: Vec<VideoFormat>,
    pub script_variants: Vec<ScriptVariant>,
    #[serde(default)]
    pub subtitle_tracks: Vec<SubtitleTrack>,
    // Preserve unknown fields
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl FsvMetadata {
    pub fn new(format_version: Version) -> Self {
        Self {
            format_version,
            extensions: Vec::new(),
            tags: Vec::new(),
            title: String::new(),
            creators: CreatorsMetadata::new(),
            video_formats: Vec::new(),
            script_variants: Vec::new(),
            subtitle_tracks: Vec::new(),
            extra: HashMap::new(),
        }
    }

    pub fn add_video_creator(&mut self, work_creator: WorkCreatorsMetadata) {
        self.creators.add_video_creator(work_creator);
    }

    pub fn add_script_creator(&mut self, work_creator: WorkCreatorsMetadata) {
        self.creators.add_script_creator(work_creator);
    }

    pub fn add_subtitle_creator(&mut self, work_creator: WorkCreatorsMetadata) {
        self.creators.add_subtitle_creator(work_creator);
    }

    pub fn add_video_format(&mut self, video_format: VideoFormat) {
        self.video_formats.push(video_format);
    }

    pub fn add_script_variant(&mut self, script_variant: ScriptVariant) {
        self.script_variants.push(script_variant);
    }

    pub fn add_subtitle_track(&mut self, subtitle_track: SubtitleTrack) {
        self.subtitle_tracks.push(subtitle_track);
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatorsMetadata {
    #[serde(default)]
    pub videos: Vec<WorkCreatorsMetadata>,
    #[serde(default)]
    pub scripts: Vec<WorkCreatorsMetadata>,
    #[serde(default)]
    pub subtitles: Vec<WorkCreatorsMetadata>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl CreatorsMetadata {
    pub fn new() -> Self {
        Self {
            videos: Vec::new(),
            scripts: Vec::new(),
            subtitles: Vec::new(),
            extra: HashMap::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.videos.is_empty() && self.scripts.is_empty() && self.subtitles.is_empty()
    }

    pub fn add_video_creator(&mut self, work_creator: WorkCreatorsMetadata) {
        self.videos.push(work_creator);
    }

    pub fn add_script_creator(&mut self, work_creator: WorkCreatorsMetadata) {
        self.scripts.push(work_creator);
    }

    pub fn add_subtitle_creator(&mut self, work_creator: WorkCreatorsMetadata) {
        self.subtitles.push(work_creator);
    }

    pub fn retain<F: FnMut(&WorkCreatorsMetadata) -> bool>(&mut self, mut f: F) {
        self.videos.retain(&mut f);
        self.scripts.retain(&mut f);
        self.subtitles.retain(&mut f);
    }
}

impl Default for CreatorsMetadata {
    fn default() -> Self {
        CreatorsMetadata::new()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkCreatorsMetadata {
    pub work_name: String,
    pub source_url: String,
    pub creator_info: CreatorInfo,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl WorkCreatorsMetadata {
    pub fn new(work_name: String, source_url: String, creator_info: CreatorInfo) -> Self {
        WorkCreatorsMetadata {
            work_name,
            source_url,
            creator_info,
            extra: HashMap::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatorInfo {
    pub name: String,
    #[serde(default)]
    pub socials: Vec<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl CreatorInfo {
    pub fn new(name: String, socials: Vec<String>) -> Self {
        CreatorInfo { name, socials, extra: HashMap::new() }
    }
}

pub trait WorkItem {
    fn get_name(&self) -> &str;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoFormat {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub duration: u64,
    #[serde(default)]
    pub checksum: String,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl VideoFormat {
    pub fn new(name: String, description: String, duration_ms: u64, checksum: String) -> Self {
        VideoFormat {
            name,
            description,
            duration: duration_ms,
            checksum,
            extra: HashMap::new(),
        }
    }
}

impl WorkItem for VideoFormat {
    fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScriptVariant {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub additional_axes: Vec<String>,
    #[serde(default)]
    pub duration: u64,
    #[serde(default)]
    pub start_offset: i64,
    #[serde(default)]
    pub checksum: String,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl ScriptVariant {
    pub fn new(name: String, description: String, additional_axes: Vec<String>, duration: u64, start_offset: i64, checksum: String) -> Self {
        ScriptVariant {
            name,
            description,
            additional_axes,
            duration,
            start_offset,
            checksum,
            extra: HashMap::new(),
        }
    }
}

impl WorkItem for ScriptVariant {
    fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubtitleTrack {
    pub name: String,
    pub language: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub checksum: String,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl SubtitleTrack {
    pub fn new(name: String, language: String, description: String, checksum: String) -> Self {
        SubtitleTrack {
            name,
            language,
            description,
            checksum,
            extra: HashMap::new(),
        }
    }
}

impl WorkItem for SubtitleTrack {
    fn get_name(&self) -> &str {
        &self.name
    }
}
