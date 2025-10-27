use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Funscript {
    pub actions: Vec<FunscriptAction>,
    pub inverted: bool,
    #[serde(default)]
    pub metadata: Option<FunscriptMetadata>,
    pub range: u64,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FunscriptAction {
    pub at: u64,
    pub pos: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FunscriptMetadata {
    pub creator: String,
    pub description: String,
    pub duration: u64,
    pub license: String,
    pub notes: String,
    pub performers: Vec<String>,
    pub script_url: String,
    pub tags: Vec<String>,
    pub title: String,
    pub r#type: String,
    pub video_url: String,
}

// TODO: Double-check the Funscript format specification and implement parsing and validation functions.