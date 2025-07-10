use std::collections::HashMap;
use std::path::Path;
use specta::Type;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::models::{ApiError, Result};
use crate::tasks::task::Task;
use crate::utils::get_handlebars;
use std::process::Command;
use encoding::label::encoding_from_whatwg_label;
use encoding::{DecoderTrap};

#[derive(Type, Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct ShellJob {
    pub shell: String,
    pub args: Vec<String>,
    pub working_dir: String,
    pub encoding: String,
}

#[derive(Clone, Debug)]
pub struct ShellTask {
    pub shell: String,
    pub args: Vec<String>,
    pub working_dir: String,
    pub encoding: String,
}

pub async fn to_shell_task(shell_job: ShellJob, cur_env: HashMap<String, String>) -> Result<Task>  {
    let shell = get_handlebars(&shell_job.shell, &cur_env)?;
    let working_dir = get_handlebars(&shell_job.working_dir, &cur_env)?;
    let encoding = get_handlebars(&shell_job.encoding, &cur_env)?;

    let mut new_args = Vec::new();
    for arg in shell_job.args.iter() {
        let new_arg = get_handlebars(arg, &cur_env)?;
        new_args.push(new_arg);
    }

    Ok(Task::ShellTask(ShellTask {
        shell,
        args: new_args,
        working_dir,
        encoding
    }))
}

pub async fn run_task_shell(task: ShellTask) -> Result<()>  {
    let folder = task.working_dir.clone();
    let p_folder = Path::new(&folder);
    if !p_folder.exists() {
        std::fs::create_dir_all(Path::new(&folder))?;
    }
    let output = Command::new(task.shell)
        .args(task.args)
        .current_dir(task.working_dir)
        .output()
        .map_err(|e| ApiError::CrawlerError(format!("{:?}", e)))?;
    // let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let encoding = encoding_from_whatwg_label(&task.encoding)
        .unwrap_or(encoding::all::UTF_8);
    let stdout = encoding
        .decode(&output.stdout, DecoderTrap::Replace)
        .map_err(|e| ApiError::CrawlerError(format!("{:?}", e)))?;
    println!("{}", stdout);
    Ok(())
}