use serde::{Deserialize, Serialize};

use crate::semver::Version;

#[derive(Debug, Serialize, Deserialize)]
pub struct FsvMetadata {
    pub format_version: Version,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub creators: CreatorsMetadata,
    pub video_formats: Vec<VideoFormat>,
    pub script_variants: Vec<ScriptVariant>,
    #[serde(default)]
    pub subtitle_tracks: Vec<SubtitleTrack>,
}

impl FsvMetadata {
    pub fn new(format_version: Version) -> Self {
        FsvMetadata {
            format_version,
            tags: Vec::new(),
            title: String::new(),
            creators: CreatorsMetadata::new(),
            video_formats: Vec::new(),
            script_variants: Vec::new(),
            subtitle_tracks: Vec::new(),
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
}

impl CreatorsMetadata {
    pub fn new() -> Self {
        CreatorsMetadata {
            videos: Vec::new(),
            scripts: Vec::new(),
            subtitles: Vec::new(),
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
}

impl WorkCreatorsMetadata {
    pub fn new(work_name: String, source_url: String, creator_info: CreatorInfo) -> Self {
        WorkCreatorsMetadata {
            work_name,
            source_url,
            creator_info,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatorInfo {
    pub name: String,
    #[serde(default)]
    pub socials: Vec<String>,
}

impl CreatorInfo {
    pub fn new(name: String, socials: Vec<String>) -> Self {
        CreatorInfo { name, socials }
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
    pub duration_ms: u64,
    #[serde(default)]
    pub start_offset_ms: u64,
    #[serde(default)]
    pub checksum: String,
}

impl VideoFormat {
    pub fn new(name: String, description: String, duration_ms: u64, start_offset_ms: u64, checksum: String) -> Self {
        VideoFormat {
            name,
            description,
            duration_ms,
            start_offset_ms,
            checksum,
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
    pub duration_ms: u64,
    #[serde(default)]
    pub start_offset_ms: u64,
    #[serde(default)]
    pub checksum: String,
}

impl ScriptVariant {
    pub fn new(name: String, description: String, additional_axes: Vec<String>, duration_ms: u64, start_offset_ms: u64, checksum: String) -> Self {
        ScriptVariant {
            name,
            description,
            additional_axes,
            duration_ms,
            start_offset_ms,
            checksum,
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
    #[serde(default)]
    pub language: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub checksum: String,
}

impl SubtitleTrack {
    pub fn new(name: String, language: String, description: String, checksum: String) -> Self {
        SubtitleTrack {
            name,
            language,
            description,
            checksum,
        }
    }
}

impl WorkItem for SubtitleTrack {
    fn get_name(&self) -> &str {
        &self.name
    }
}
