use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use specta::Type;
use crate::tasks::html_task::{HtmlJob, HtmlTask};
use crate::tasks::http_task::{HttpJob, HttpTask};
use crate::tasks::shell_task::{ShellJob, ShellTask};

#[derive(Type, Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub enum Job {
    HttpJob(HttpJob),
    HtmlJob(HtmlJob),
    ShellJob(ShellJob),
}

#[derive(Clone, Debug)]
pub enum Task {
    HttpTask(HttpTask),
    HtmlTask(HtmlTask),
    ShellTask(ShellTask),
}