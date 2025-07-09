use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use encoding_rs::Encoding;
use mime::Mime;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::{Client, RequestBuilder};
use sanitize_filename::sanitize;
use serde_json::Value;
use crate::models::{HttpJob, HttpTask, Task};
use crate::models::Result;
use crate::utils::{get_handlebars, get_handlebars_safe_dir};

pub async fn to_http_task(client: Client, http_job: HttpJob, cur_env: HashMap<String, String>, g_header: HashMap<String, String>) -> Result<Task> {

    // let client = self.client.clone();

    let url = get_handlebars(&http_job.url, &cur_env)?;

    let method = http_job.method.clone();

    let mut header = HeaderMap::new();
    for (k, v) in g_header.iter() {
        let nm = HeaderName::from_str(k.as_str())?;
        let new_v = get_handlebars(v, &cur_env)?;
        let val = HeaderValue::from_str(&new_v)?;
        header.insert(nm, val);
    }

    for (k, v) in http_job.header.iter() {
        let nm = HeaderName::from_str(k.as_str())?;
        let new_v = get_handlebars(v, &cur_env)?;
        let val = HeaderValue::from_str(&new_v)?;
        header.insert(nm, val);
    }
    let folder = get_handlebars_safe_dir(&http_job.output, &cur_env)?;
    let filename = sanitize(get_handlebars(&http_job.filename, &cur_env)?);
    let p: PathBuf = Path::new(&folder).join(filename);
    let save_path = p.to_string_lossy().to_string();

    Ok(Task::HttpTask(HttpTask {
        client,
        url,
        method,
        header,
        folder,
        save_path,
    }))
}

pub async fn run_task_http(task: HttpTask) -> Result<()> {
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
        return Ok(());
    }

    if p_tmp.exists() {
        let _ = std::fs::remove_file(p_tmp).map_err(|e| println!("{:?}", e));
    }

    let mut req_builder: RequestBuilder;
    if task.method == "POST" {
        req_builder = task.client.post(&task.url);
    } else {
        req_builder = task.client.get(&task.url);
    }
    req_builder = req_builder.headers(task.header.clone());
    let res = req_builder.send().await?;

    if !res.status().is_success() {
        println!("run_task err: {:?} {:?}", res.status(), &task);
        return Ok(());
    }

    let mut charset: Option<String> = None;
    let mut mime_type: Option<String> = None;
    let content_type = res
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    println!("content_type: {:?}", content_type);
    if let Some(content_type) = &content_type {
        charset = content_type
            .clone()
            .split("charset=")
            .nth(1)
            .and_then(|s| Some(s.to_string()));
        if let Ok(mime) = content_type.clone().parse::<Mime>() {
            mime_type = Some(mime.essence_str().to_string());
        };
    }
    // let body = res.text().await?;
    // println!("body: {:?}", &body);
    let bytes = res.bytes().await?;


    if Some("application/json".to_string()) == mime_type {
        let label = charset.unwrap_or("utf-8".to_string());
        let (text, _, _) = Encoding::for_label(label.as_bytes())
            .unwrap_or(encoding_rs::UTF_8)
            .decode(&bytes);

        let json_value: Value = serde_json::from_str(&text)?;
        let formatted = serde_json::to_string_pretty(&json_value)?;

        let mut file = std::fs::File::create(p_tmp)?;
        file.write_all(formatted.as_bytes())?;
        std::fs::rename(p_tmp, p)?;
    } else {
        let mut file = std::fs::File::create(p_tmp)?;
        file.write_all(&bytes)?;
        std::fs::rename(p_tmp, p)?;
    }

    Ok(())
}
