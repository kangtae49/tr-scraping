use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use encoding_rs::Encoding;
use mime::Mime;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::{Client, RequestBuilder};
use sanitize_filename::sanitize;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;
use crate::models::ApiError;
use crate::models::Result;
use crate::utils::{get_handlebars, get_handlebars_safe_dir};
use crate::tasks::task::{Task};

#[derive(Type, Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct HttpJob {
    pub url: String,
    pub method: String,
    pub header: HashMap<String, String>,
    pub filename: String,
    pub output: String,
}

impl HttpJob {
    pub fn pre_process(&mut self) -> Result<()> {
        Ok(())
    }

    pub async fn make_task(&self, cur_env: HashMap<String, String>, client: Client, g_header: HashMap<String, String>) -> Result<Task> {

        let url = get_handlebars(&self.url, &cur_env)?;

        let method = self.method.clone();

        let mut header = HeaderMap::new();
        for (k, v) in g_header.iter() {
            let nm = HeaderName::from_str(k.as_str())?;
            let new_v = get_handlebars(v, &cur_env)?;
            let val = HeaderValue::from_str(&new_v)?;
            header.insert(nm, val);
        }

        for (k, v) in self.header.iter() {
            let nm = HeaderName::from_str(k.as_str())?;
            let new_v = get_handlebars(v, &cur_env)?;
            let val = HeaderValue::from_str(&new_v)?;
            header.insert(nm, val);
        }
        let folder = get_handlebars_safe_dir(&self.output, &cur_env)?;
        let filename = sanitize(get_handlebars(&self.filename, &cur_env)?);
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
}

#[derive(Clone, Debug)]
pub struct HttpTask {
    pub client: Client,
    pub url: String,
    pub method: String,
    pub header: HeaderMap,
    pub folder: String,
    pub save_path: String,
}


impl HttpTask {
    pub async fn run(&mut self) -> Result<()> {
        let folder = self.folder.clone();
        let p_folder = Path::new(&folder);
        if !p_folder.exists() {
            std::fs::create_dir_all(Path::new(&folder))?;
        }

        let save_path = self.save_path.clone();
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
        if self.method == "POST" {
            req_builder = self.client.post(&self.url);
        } else {
            req_builder = self.client.get(&self.url);
        }
        req_builder = req_builder.headers(self.header.clone());
        let res = req_builder.send().await?;

        if !res.status().is_success() {
            println!("run_task err: {:?} {:?}", res.status(), &self);
            return Err(ApiError::CrawlerError(format!("status: {:?} {:?}", res.status(), &self.url)));
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
}


