use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use reqwest::Client;
use reqwest::header::HeaderMap;
use serde::{Serialize, Deserialize};
use serde_with::{serde_as, skip_serializing_none};
use specta::Type;
use tokio::sync::{Notify, Semaphore};
use thiserror::Error;

#[derive(Type, Serialize, Deserialize, Clone, Debug)]
pub enum TaskIter {
    Range(IterRange),
    Pattern(IterPattern),
    RangePattern(IterRangePattern),
    Vec(IterList)
}

#[derive(Type, Serialize, Deserialize, Clone, Debug)]
pub struct IterRange {
    pub name: String,
    pub offset: String,
    pub take: String,
}

#[derive(Type, Serialize, Deserialize, Clone, Debug)]
pub struct IterPattern {
    pub name: String,
    pub file_pattern: String,
    pub content_pattern: String,
}

#[derive(Type, Serialize, Deserialize, Clone, Debug)]
pub struct IterRangePattern {
    pub name: String,
    pub file_pattern: String,
    pub offset: String,
    pub take: String,
}

#[derive(Type, Serialize, Deserialize, Clone, Debug)]
pub struct IterList {
    pub name: String,
    pub val: Vec<String>,
}




#[derive(Type, Serialize, Deserialize, Clone, Debug)]
pub struct Request {
    pub url: String,
    pub method: String,
    pub header: HashMap<String, String>,
    pub filename: String
}

#[derive(Type, Serialize, Deserialize, Clone, Debug)]
pub struct Step {
    pub name: String,
    pub task_iters: Vec<TaskIter>,
    pub req: Request,
    pub output: String,
    pub concurrency_limit: usize,
}

#[derive(Clone)]
pub struct Task {
    pub client: Client,
    pub url: String,
    pub method: String,
    pub header: HeaderMap,
    pub save_path: String,
}

#[serde_as]
#[derive(Type, Serialize, Deserialize, Clone, Debug)]
pub struct Setting {
    pub env: HashMap<String, String>,
    pub steps: HashMap<String, Step>,
    pub edges: Vec<Edge>,
}

pub struct StepHandle {
    #[allow(dead_code)]
    pub name: String,
    // pub rx: mpsc::Receiver<Request>,
    // pub tx: mpsc::Sender<Request>,
    // pub client: Client,
    pub semaphore: Arc<Semaphore>,
    pub paused: Arc<AtomicBool>,
    pub notifier: Arc<Notify>
}

#[derive(Type, Serialize, Deserialize, Clone, Debug)]
pub struct Edge {
    pub a: String,
    pub b: String,
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