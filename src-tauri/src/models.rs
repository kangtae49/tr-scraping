use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none};
use specta::Type;
use std::collections::HashMap;
use std::sync::atomic::AtomicU8;
use std::sync::{Arc, Condvar, Mutex};
use thiserror::Error;
use tokio::sync::{RwLock, Semaphore};
use crate::tasks::task::Job;

pub type Result<T> = std::result::Result<T, ApiError>;
pub type ItemData = HashMap<String, String>;

pub type Shared<T> = Arc<RwLock<T>>;

pub const STEP_RUNNING: u8 = 0;
pub const STEP_PAUSED: u8 = 1;
pub const STEP_STOPPED: u8 = 2;

#[derive(Type, Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub enum TaskIter {
    Range(IterRange),
    Pattern(IterPattern),
    RangePattern(IterRangePattern),
    Vec(IterList),
    GlobJsonPattern(IterGlobJsonPattern),
    GlobJsonRangePattern(IterJsonRangePattern),
}

#[derive(Type, Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct IterGlobJsonPattern {
    pub glob_pattern: String,
    pub item_pattern: String,
    pub env_pattern: HashMap<String, String>,
}

#[derive(Type, Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct IterJsonRangePattern {
    pub name: String,
    pub file_pattern: String,
    pub offset_pattern: String,
    pub take_pattern: String,
}

#[derive(Type, Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct IterRange {
    pub name: String,
    pub offset: String,
    pub take: String,
}

#[derive(Type, Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct IterPattern {
    pub name: String,
    pub glob_pattern: String,
    pub content_pattern: String,
}

#[derive(Type, Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct IterRangePattern {
    pub name: String,
    pub glob_pattern: String,
    pub offset: String,
    pub take: String,
}

#[derive(Type, Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct IterList {
    pub name: String,
    pub val: Vec<String>,
}




#[derive(Type, Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct Step {
    pub name: String,
    pub task_iters: Vec<TaskIter>,
    pub job: Job,
    pub concurrency_limit: usize,
}

#[serde_as]
#[derive(Type, Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct Setting {
    pub env: HashMap<String, String>,
    pub header: HashMap<String, String>,
    pub steps: HashMap<String, Step>,
}

pub struct StepHandle {
    #[allow(dead_code)]
    pub name: String,
    pub semaphore: Arc<Semaphore>,
    pub state: Arc<AtomicU8>,
    pub control: Arc<(Mutex<()>, Condvar)>
}


#[allow(dead_code)]
#[skip_serializing_none]
#[serde_as]
#[derive(Type, Serialize, Deserialize, Clone, Debug, Default)]
pub struct TextContent {
    pub path: String,
    pub mimetype: String,
    pub enc: Option<String>,
    pub text: Option<String>,
}

#[allow(dead_code)]
#[skip_serializing_none]
#[serde_as]
#[derive(Type, Serialize, Deserialize, Clone, Debug, Default)]
pub struct StepNotify {
    pub name: String,
    pub status: String,
    pub message: String,
}

#[derive(Type, Serialize, Deserialize, Error, Debug)]
pub enum ApiError {
    #[error("Crawler error: {0}")]
    CrawlerError(String),

    #[error("handlebars error: {0}")]
    TemplateError(String),

    #[error("reqwest error: {0}")]
    ReqwestError(String),

    #[error("IO error: {0}")]
    Io(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("JSON error: {0}")]
    JsonError(String),

    #[error("Glob error: {0}")]
    GlobError(String),
}

impl From<handlebars::TemplateError> for ApiError {
    fn from(e: handlebars::TemplateError) -> Self {
        ApiError::TemplateError(e.to_string())
    }
}

impl From<handlebars::RenderError> for ApiError {
    fn from(e: handlebars::RenderError) -> Self {
        ApiError::TemplateError(e.to_string())
    }
}

impl From<reqwest::header::InvalidHeaderName> for ApiError {
    fn from(e: reqwest::header::InvalidHeaderName) -> Self {
        ApiError::ReqwestError(e.to_string())
    }
}

impl From<reqwest::header::InvalidHeaderValue> for ApiError {
    fn from(e: reqwest::header::InvalidHeaderValue) -> Self {
        ApiError::ReqwestError(e.to_string())
    }
}
impl From<reqwest::Error> for ApiError {
    fn from(e: reqwest::Error) -> Self {
        ApiError::ReqwestError(e.to_string())
    }
}

impl From<std::io::Error> for ApiError {
    fn from(e: std::io::Error) -> Self {
        ApiError::Io(e.to_string())
    }
}

impl From<std::num::ParseIntError> for ApiError {
    fn from(e: std::num::ParseIntError) -> Self {
        ApiError::ParseError(e.to_string())
    }
}

impl From<serde_json::error::Error> for ApiError {
    fn from(e: serde_json::error::Error) -> Self {
        ApiError::JsonError(e.to_string())
    }
}

impl From<glob::PatternError> for ApiError {
    fn from(e: glob::PatternError) -> Self {
        ApiError::GlobError(e.to_string())
    }
}

impl From<jsonpath_lib::JsonPathError> for ApiError {
    fn from(e: jsonpath_lib::JsonPathError) -> Self {
        ApiError::JsonError(e.to_string())
    }
}
