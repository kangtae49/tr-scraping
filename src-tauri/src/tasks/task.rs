use std::collections::HashMap;
use reqwest::Client;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use specta::Type;
use crate::tasks::html_task::{HtmlJob, HtmlTask};
use crate::tasks::http_task::{HttpJob, HttpTask};
use crate::tasks::csv_task::{CsvJob, CsvTask};
use crate::tasks::shell_task::{ShellJob, ShellTask};
use crate::Result;

#[derive(Type, Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub enum Job {
    HttpJob(HttpJob),
    HtmlJob(HtmlJob),
    ShellJob(ShellJob),
    CsvJob(CsvJob),
}

impl Job {
    pub fn pre_process(&mut self) -> Result<()> {
        match self {
            Job::HttpJob(job) => job.pre_process(),
            Job::HtmlJob(job) => job.pre_process(),
            Job::CsvJob(job) => job.pre_process(),
            Job::ShellJob(job) => job.pre_process(),
        }
    }

    pub async fn make_task(&self,  cur_env: HashMap<String, String>, client: Client, g_header: HashMap<String, String>) -> Result<Task> {
        match self {
            Job::HttpJob(job) => {job.make_task(cur_env, client, g_header).await},
            Job::HtmlJob(job) => {job.make_task(cur_env).await},
            Job::CsvJob(job) => {job.make_task(cur_env).await},
            Job::ShellJob(job) => {job.make_task(cur_env).await},
        }
    }


}

#[derive(Clone, Debug)]
pub enum Task {
    HttpTask(HttpTask),
    HtmlTask(HtmlTask),
    CsvTask(CsvTask),
    ShellTask(ShellTask),
}

impl Task {
    pub async fn run_task(&mut self) -> crate::models::Result<()> {
        match self {
            Task::HttpTask(task) => {task.run().await}
            Task::HtmlTask(task) => {task.run().await}
            Task::CsvTask(task) => {task.run().await}
            Task::ShellTask(task) => {task.run().await}
        }
    }
}