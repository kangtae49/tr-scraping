use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use sanitize_filename::sanitize;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;
use crate::models::{ApiError, Result};
use crate::utils::{get_json_val, from_unix_time, get_handlebars, get_handlebars_safe_dir};
use crate::tasks::task::Task;

#[derive(Type, Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct HtmlJob {
    pub json_map: HashMap<String, Vec<(String, String)>>,
    pub output_template_file: String,
    pub output_template: Option<String>,
    pub filename: String,
    pub output: String,
}

#[derive(Clone, Debug)]
pub struct HtmlTask {
    pub cur_env: HashMap<String, String>,
    pub html_template: String,
    pub json_map: HashMap<String, Vec<(String, String)>>,
    pub folder: String,
    pub save_path: String,
}

pub async fn to_html_task(html_job: HtmlJob, cur_env: HashMap<String, String>) -> Result<Task> {
    let Some(output_template) = html_job.output_template.clone() else { return Err(ApiError::CrawlerError("no output template".to_string())); };

    let folder = get_handlebars_safe_dir(&html_job.output, &cur_env)?;
    let filename = sanitize(get_handlebars(&html_job.filename, &cur_env)?);
    let p: PathBuf = Path::new(&folder).join(filename);
    let save_path = p.to_string_lossy().to_string();
    Ok(Task::HtmlTask(HtmlTask {
        cur_env: cur_env.clone(),
        html_template: output_template.clone(),
        json_map: html_job.json_map.clone(),
        folder,
        save_path,
    }))
}




pub async fn run_task_html(mut task: HtmlTask) -> Result<()> {
    let folder = task.folder.clone();
    let p_folder = Path::new(&folder);
    if !p_folder.exists() {
        std::fs::create_dir_all(Path::new(&folder))?;
    }

    let save_path = task.save_path.clone();
    let tmp_path = format!("{}.tmp", &save_path);
    let p = Path::new(&save_path);
    let p_tmp = Path::new(tmp_path.as_str());
    if p.exists() {
        // return Ok(());
        let _ = std::fs::remove_file(p).map_err(|e| println!("{:?}", e));
    }

    if p_tmp.exists() {
        let _ = std::fs::remove_file(p_tmp).map_err(|e| println!("{:?}", e));
    }

    let template = task.html_template.clone();

    for (k, v) in task.json_map.iter() {
        let Some(json_str) = task.cur_env.get(k) else { continue };
        let Ok(vec_json) = serde_json::from_str::<Vec<Value>>(json_str) else { continue };
        let mut s = "".to_string();
        for json_val in vec_json {
            s += "<div class=\"row\">";
            for (sk, sv) in v.iter() {
                let Some(mut vv) = get_json_val(&json_val, sv) else {continue};
                if sk.to_uppercase().contains("DATE") {
                    if let Ok(dt) = from_unix_time(vv.clone()) {
                        vv = dt;
                    }
                }
                s += &format!("<div class=\"{}\">{}</div>", sk, vv);
            }
            s += "</div>";
        }
        task.cur_env.insert(k.clone(), s);
    }
    let mut task_env = task.cur_env.clone();
    for (k, v) in task_env.iter_mut() {
        if k.to_uppercase().contains("DATE") {
            if let Ok(dt) = from_unix_time(v.clone()) {
                *v = dt;
            }
        }
    }

    let html_content = get_handlebars(&template, &task_env)?;
    let mut file = std::fs::File::create(p_tmp)?;
    file.write_all(html_content.as_bytes())?;
    std::fs::rename(p_tmp, p)?;
    // use tokio::time::{sleep, Duration};
    // println!("sleep start");
    // sleep(Duration::from_secs(5)).await;
    // println!("sleep end");
    Ok(())
}
