use std::collections::{HashMap};
use std::io::Write;
use std::path::{absolute, Path, PathBuf};
use std::pin::Pin;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use async_stream::stream;
use chardetng::EncodingDetector;
use chrono::{DateTime, Local, Utc};
use encoding_rs::Encoding;
use glob::glob;
use handlebars::Handlebars;
use mime::Mime;
use mime_guess::from_path;
use petgraph::graph::Graph;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::{Client, RequestBuilder};
use sanitize_filename::sanitize;
use serde_json::Value;
use tokio::io::{AsyncReadExt};
use tokio::sync::Semaphore;
use tokio::sync::{Notify, RwLock};
use tokio_stream::{Stream, StreamExt};


use crate::models::{ApiError, Edge, IterGlobJsonPattern, IterJsonRangePattern, IterList, IterPattern, IterRange, IterRangePattern, Job, Setting, Step, StepHandle, HttpTask, HtmlTask, TaskIter, TextContent, Task, HttpJob, HtmlJob};

type ItemData = HashMap<String, String>;

type Result<T> = std::result::Result<T, ApiError>;

type Shared<T> = Arc<RwLock<T>>;

pub struct Crawler {
    pub client: Client,
    pub env: Shared<HashMap<String, String>>,
    pub header: Shared<HashMap<String, String>>,
    pub steps: Shared<HashMap<String, Step>>,
    pub step_handles: Shared<HashMap<String, StepHandle>>,
    pub dag: Shared<Graph<String, ()>>,
    // pub output_html: Shared<Option<OutputHtml>>,
    // pub output_html_handle: Shared<Option<StepHandle>>,
}

impl Crawler {
    pub fn new() -> Self {
        Crawler {
            client: Client::new(),
            env: Arc::new(RwLock::new(HashMap::new())),
            header: Arc::new(RwLock::new(HashMap::new())),
            steps: Arc::new(RwLock::new(HashMap::new())),
            step_handles: Arc::new(RwLock::new(HashMap::new())),
            dag: Arc::new(RwLock::new(Graph::<String, ()>::new())),
            // output_html: Arc::new(RwLock::new(None)),
            // output_html_handle: Arc::new(RwLock::new(None)),
        }
    }

    async fn assign<T>(&self, target: &Shared<T>, new_val: T) {
        let arc = Arc::clone(target);
        let mut val = arc.write().await;
        *val = new_val;
    }

    pub fn get_arg_path(&self) -> Option<String> {
        let args: Vec<String> = std::env::args().collect();
        if args.len() > 1 {
            match absolute(&args[1]) {
                Ok(path) => Some(path.to_string_lossy().to_string()),
                Err(e) => {
                    println!("{:?}", e);
                    None
                }
            }
        } else {
            None
        }
    }

    pub async fn read_txt(&self, path_str: &str) -> Result<TextContent> {
        let path = PathBuf::from(path_str);

        let mut file = tokio::fs::File::open(&path).await?;
        let mut reader = tokio::io::BufReader::new(file);

        let mut sample = vec![0u8; 16 * 1024];
        let n = reader.read(&mut sample).await?;
        sample.truncate(n);

        let mime_type = match infer::get(&sample) {
            Some(infer_type) => infer_type.mime_type().to_string(),
            None => from_path(path_str).first_or_octet_stream().to_string(),
        };

        // let mut mime_type = from_path(path_str).first_or_octet_stream().to_string();
        // if mime_type == "application/octet-stream" {
        //     if let Some(infer_type) = infer::get(&sample) {
        //         mime_type = infer_type.mime_type().to_string()
        //     }
        // }

        println!("mime_type: {}", mime_type);

        let sz = path.metadata()?.len();

        if sz > 5 * 1024 * 1024 {
            // return Err(ApiError::Folder(String::from("Err MimeType")))
            Ok(TextContent {
                path: path_str.to_string(),
                mimetype: mime_type,
                enc: None,
                text: None,
            })
        } else {
            file = tokio::fs::File::open(&path).await?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).await?;

            let mut detector = EncodingDetector::new();
            detector.feed(&buffer, true);
            let encoding: &Encoding = detector.guess(None, true);

            let (text, _, had_errors) = encoding.decode(&buffer);
            let opt_text = if had_errors {
                None
            } else {
                Some(text.into_owned())
            };

            Ok(TextContent {
                path: path_str.to_string(),
                mimetype: mime_type,
                enc: Some(encoding.name().to_string()),
                text: opt_text,
            })
        }
    }

    pub async fn load(&mut self, setting: Setting) -> Result<()> {
        println!("setting: {:?}", setting);
        let mut step_handles = HashMap::<String, StepHandle>::new();
        for (_nm, step) in setting.steps.iter() {
            // let (tx, rx) = mpsc::channel::<Request>(1000);
            let concurrency_limit = step.concurrency_limit;
            let step_handle = StepHandle {
                name: step.name.clone(),
                // client: self.client.clone(),
                notifier: Arc::new(Notify::new()),
                // rx,
                // tx: tx.clone(),
                paused: Arc::new(AtomicBool::new(false)),
                semaphore: Arc::new(Semaphore::new(concurrency_limit)),
            };
            step_handles.insert(step.name.clone(), step_handle);
        }

        let mut dag = Graph::<String, ()>::new();
        let edges: Vec<Edge> = setting.edges;
        for edge in edges.iter() {
            let edge_a = &edge.a;
            let edge_b = &edge.b;
            if exist_node(&dag, edge_a) {
                dag.add_node(edge_a.clone());
            }
            if exist_node(&dag, edge_b) {
                dag.add_node(edge_b.clone());
            }
        }
        for edge in edges.iter() {
            let edge_a = &edge.a;
            let edge_b = &edge.b;
            if let (Some(a), Some(b)) = (find_node(&dag, edge_a), find_node(&dag, edge_b)) {
                dag.add_edge(a, b, ());
            }
        }

        self.assign(&self.env, setting.env).await;
        self.assign(&self.header, setting.header).await;
        self.assign(&self.steps, setting.steps).await;
        self.assign(&self.step_handles, step_handles).await;
        self.assign(&self.dag, dag).await;

        Ok(())
    }

    pub async fn run_step(&mut self, step_name: String) -> Result<()> {
        println!("Start Step: {}", &step_name);
        let steps_arc = Arc::clone(&self.steps);
        let steps = steps_arc.read().await;
        let env_arc = Arc::clone(&self.env);
        let header_arc = Arc::clone(&self.header);

        let step = steps
            .get(&step_name)
            .ok_or(ApiError::CrawlerError("Step not found".to_string()))?;
        let mut job = step.job.clone();
        let env_lock = env_arc.read().await;
        let env = env_lock.clone();
        let header_lock = header_arc.read().await;
        let g_header = header_lock.clone();

        let step_handles_arc = self.step_handles.clone();
        let step_handles = step_handles_arc.read().await;
        let step_handle = step_handles
            .get(&step_name)
            .ok_or(ApiError::CrawlerError("Step not found".to_string()))?;
        let semaphore = step_handle.semaphore.clone();
        let paused = step_handle.paused.clone();

        let mut task_iters = step.task_iters.clone();
        if task_iters.is_empty() {
            task_iters.push(TaskIter::Range(IterRange {
                name: format!("IDX_{}", &step_name),
                offset: "0".to_string(),
                take: "1".to_string(),
            }))
        }
        match &mut job {
            Job::HttpJob(_http_job) => {}
            Job::HtmlJob(html_job) => {
                let Ok(html_template) = std::fs::read_to_string(&html_job.output_template_file) else { return Err(ApiError::CrawlerError("err html_template".to_string())); };
                html_job.output_template = Some(html_template);
            }
        }


        paused.store(false, Ordering::SeqCst);

        let mut handles = Vec::new();

        let mut stream = get_iters(task_iters, env.clone());
        while let Some((vals, cur_env)) = stream.next().await {
            println!("iter: {:?}", vals);
            if paused.load(Ordering::SeqCst) {
                println!("paused");
                return Ok(());
            }

            let task = self.to_task(job.clone(), cur_env, g_header.clone()).await?;
            println!("task: {:?}", task);
            let _permit = semaphore.clone().acquire_owned().await.unwrap();
            let handle = tokio::task::spawn(async move {
                if let Err(e) = run_task(task).await {
                    eprintln!("Error: {:?}", e);
                }
            });

            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }
        println!("End Step: {}", &step_name);
        Ok(())
    }

    pub async fn to_task(&self, job: Job, cur_env: HashMap<String, String>, g_header: HashMap<String, String>) -> Result<Task> {
        match job {
            Job::HttpJob(http_job) => {self.to_http_task(http_job, cur_env, g_header).await},
            Job::HtmlJob(html_job) => {self.to_html_task(html_job, cur_env).await},
        }
    }

    pub async fn to_http_task(&self, http_job: HttpJob, cur_env: HashMap<String, String>, g_header: HashMap<String, String>) -> Result<Task> {

        let client = self.client.clone();

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
        let p_folder = Path::new(&folder);
        if !p_folder.exists() {
            std::fs::create_dir_all(Path::new(&folder))?;
        }
        let filename = sanitize(get_handlebars(&http_job.filename, &cur_env)?);
        let p: PathBuf = Path::new(&folder).join(filename);
        let save_path = p.to_string_lossy().to_string();

        Ok(Task::HttpTask(HttpTask {
            client,
            url,
            method,
            header,
            save_path,
        }))
    }

    pub async fn to_html_task(&self, html_job: HtmlJob, cur_env: HashMap<String, String>) -> Result<Task> {
        println!("to_html_task: {:?}", html_job);
        let Some(output_template) = html_job.output_template.clone() else { return Err(ApiError::CrawlerError("no output template".to_string())); };

        println!("output_template: {}", output_template);

        let folder = get_handlebars_safe_dir(&html_job.output, &cur_env)?;
        let p_folder = Path::new(&folder);
        if !p_folder.exists() {
            std::fs::create_dir_all(Path::new(&folder))?;
        }
        let filename = sanitize(get_handlebars(&html_job.filename, &cur_env)?);
        let p: PathBuf = Path::new(&folder).join(filename);
        let save_path = p.to_string_lossy().to_string();
        println!("{}", save_path);
        Ok(Task::HtmlTask(HtmlTask {
            cur_env: cur_env.clone(),
            html_template: output_template.clone(),
            json_map: html_job.json_map.clone(),
            save_path,
        }))
    }

}


fn get_iters(
    task_iters: Vec<TaskIter>,
    env: HashMap<String, String>,
) -> Pin<Box<dyn Stream<Item = (Vec<Option<ItemData>>, HashMap<String, String>)> + Send>> {
    Box::pin(stream! {
        let mut cur_vals: Vec<Option<ItemData>> = Vec::new();
        let mut iters: Vec<Pin<Box<dyn Stream<Item = ItemData> + Send>>> = Vec::new();
        let mut env = env.clone();
        let len = task_iters.len();
        
        for _i in 0..len {
            iters.push(Box::pin(tokio_stream::empty()));
            cur_vals.push(None);
        }
        
        let mut pos = 0;
        iters[pos] = get_iter(task_iters[pos].clone(), env.clone());
        println!("Start iter loop");
        loop {
            if pos != 0 && cur_vals[pos].is_none() {
                iters[pos] = get_iter(task_iters[pos].clone(), env.clone());
            }
            let pos_val = iters[pos].next().await;
            cur_vals[pos] = pos_val.clone();
            match pos_val {
                Some(pos_v) => {
                    env.extend(pos_v.clone());
                    if pos == len - 1 {
                        yield (cur_vals.clone(), env.clone());
                    } else {
                        pos += 1;
                    }
                }
                None => {
                    if pos == 0 {
                        println!("End iter loop");
                        break;
                    } else {
                        pos -= 1;
                    }
                }
            }
        }
    })
}


fn get_iter(task_iter: TaskIter, env: HashMap<String, String>) -> Pin<Box<dyn Stream<Item = ItemData> + Send>> {
    match task_iter {
        TaskIter::Vec(iter_vec) => get_iter_vec(iter_vec),
        TaskIter::Range(iter_range) => get_iter_range(iter_range, env),
        TaskIter::Pattern(iter_pattern) => get_iter_pattern(iter_pattern),
        TaskIter::RangePattern(iter_range_pattern) => {
            get_iter_range_pattern(iter_range_pattern, env)
        }
        TaskIter::GlobJsonPattern(iter_glob_json_pattern) => {
            get_iter_glob_json_pattern(iter_glob_json_pattern, env)
        }
        TaskIter::GlobJsonRangePattern(iter_glob_json_range_pattern) => {
            get_iter_glob_json_range_pattern(iter_glob_json_range_pattern, env)
        }
    }
}

fn get_iter_glob_json_pattern(
    iter_glob_json_pattern: IterGlobJsonPattern,
    env: HashMap<String, String>,
) -> Pin<Box<dyn Stream<Item = ItemData> + Send>> {
    Box::pin(stream! {
        let glob_pattern = iter_glob_json_pattern.glob_pattern;
        let item_pattern = iter_glob_json_pattern.item_pattern;
        let mut env_pattern = iter_glob_json_pattern.env_pattern;

        let Ok(glob_pattern) = get_handlebars(&glob_pattern, &env) else { return ;};
        let Ok(item_pattern) = get_handlebars(&item_pattern, &env) else { return ;};

        for (_k, v) in env_pattern.iter_mut() {
            let Ok(new_val) = get_handlebars(&v, &env) else { continue; };
            *v = new_val;
        }

        let Ok(paths) = glob(&glob_pattern) else { return ;};
        for entry in paths {
            let Ok(p) = entry else { continue };
            let Ok(json_str) = std::fs::read_to_string(p) else {
                continue;
            };
            let Ok(json) = serde_json::from_str(&json_str) else {
                continue;
            };
            let Ok(item_vals) = jsonpath_lib::select(&json, &item_pattern) else {
                continue;
            };
            for item in item_vals {
                let mut env_item = HashMap::new();
                for (k, v) in env_pattern.iter() {
                    if let Some(j_val) = get_json_val(item, v) {
                        env_item.insert(k.to_string(), j_val);
                    }
                }
                yield env_item;
            }
        }

    })
}

fn get_iter_glob_json_range_pattern(
    iter_glob_json_range_pattern: IterJsonRangePattern,
    env: HashMap<String, String>,
) -> Pin<Box<dyn Stream<Item = ItemData> + Send>> {
    Box::pin(stream! {
        let name = iter_glob_json_range_pattern.name;
        let file_pattern = iter_glob_json_range_pattern.file_pattern;
        let offset_pattern = iter_glob_json_range_pattern.offset_pattern;
        let take_pattern = iter_glob_json_range_pattern.take_pattern;

        let Ok(file_pattern) = get_handlebars(&file_pattern, &env) else { return ; };
        let Ok(offset_pattern) = get_handlebars(&offset_pattern, &env) else { return ;};
        let Ok(take_pattern) = get_handlebars(&take_pattern, &env) else { return; };

        let Ok(mut paths) = glob(&file_pattern) else { return ; };
        let Some(entry) = paths.next() else { return ; };
        let Ok(p) = entry else { return ; };
        let Ok(json_str) = std::fs::read_to_string(p) else { return; };
        let Ok(json) = serde_json::from_str(&json_str) else { return ; };
        let offset_str = get_json_val(&json, &offset_pattern).unwrap_or(offset_pattern);
        let take_str = get_json_val(&json, &take_pattern).unwrap_or(take_pattern);
        let Ok(offset) = offset_str.parse::<usize>() else { return ;};
        let Ok(take) = take_str.parse::<usize>() else {return ;};

        let start = offset;
        let end = offset + take;
        for i in start..end {
            yield HashMap::from([(name.to_string(), i.to_string())]);
        }
    })
}

fn get_iter_range(iter_range: IterRange, env: HashMap<String, String>) -> Pin<Box<dyn Stream<Item = ItemData> + Send>> {
    Box::pin(stream! {
        let name = iter_range.name;
        let offset_pattern = iter_range.offset;
        let take_pattern = iter_range.take;
        let Ok(offset_str) = get_handlebars(&offset_pattern, &env) else { return ; };
        let Ok(take_str) = get_handlebars(&take_pattern, &env) else { return ; };
        let Ok(offset) = offset_str.parse::<usize>() else { return ; };
        let Ok(take) = take_str.parse::<usize>() else { return ;};
        let start = offset;
        let end = offset + take;
        for i in start..end {
            yield HashMap::from([(name.to_string(), i.to_string())]);
        }
    })
}

fn get_iter_range_pattern(
    iter_range_pattern: IterRangePattern,
    env: HashMap<String, String>,
) -> Pin<Box<dyn Stream<Item = ItemData> + Send>> {
    Box::pin(stream! {
        let name = iter_range_pattern.name;
        let glob_pattern = iter_range_pattern.glob_pattern;
        let Ok(file_path) = get_handlebars(&glob_pattern, &env) else { return ; };
        let Ok(mut offset_str) = get_handlebars(&iter_range_pattern.offset, &env) else { return ;};
        let Ok(mut take_str) = get_handlebars(&iter_range_pattern.take, &env) else { return ;};

        let Ok(json_str) = std::fs::read_to_string(Path::new(&file_path)) else { return ; };
        let Ok(json) = serde_json::from_str(&json_str) else { return; };
        match get_json_val(&json, &offset_str) {
            Some(val) => {
                offset_str = val;
            }
            None => {}
        }
        match get_json_val(&json, &take_str) {
            Some(val) => {
                take_str = val;
            }
            None => {}
        }

        let Ok(offset) = offset_str.parse::<usize>() else { return ; };
        let Ok(take) = take_str.parse::<usize>() else { return ; };
        let start = offset;
        let end = offset + take;

        for i in start..end {
            yield HashMap::from([(name.to_string(), i.to_string())]);
        }
    })
}

fn get_json_val(json: &Value, path: &str) -> Option<String> {
    let Ok(values) = jsonpath_lib::select(json, path) else {
        return None;
    };
    let Some(&val) = values.first() else {
        return None;
    };
    match val {
        Value::String(s) => Some(s.clone().trim().to_string()),
        _ => Some(val.to_string().trim().to_string()),
    }
}

fn get_iter_pattern(iter_pattern: IterPattern) -> Pin<Box<dyn Stream<Item = ItemData> + Send>> {
    Box::pin(stream!{
        let name = iter_pattern.name;
        let glob_pattern = iter_pattern.glob_pattern;
        let content_pattern = iter_pattern.content_pattern;
        if let Ok(paths) = glob(&glob_pattern) {
            for entry in paths {
                let Ok(p) = entry else { continue };
                let Ok(json_str) = std::fs::read_to_string(p) else { continue; };
                let Ok(json) = serde_json::from_str(&json_str) else { continue; };
                let Ok(values) = jsonpath_lib::select(&json, &content_pattern) else { continue; };
                for val in values {
                    yield HashMap::from([(name.to_string(), val.to_string())]);
                }
            }
        }
    })
}

fn get_iter_vec(iter_vec: IterList) -> Pin<Box<dyn Stream<Item = ItemData> + Send>> {
    Box::pin(stream! {
        let name = iter_vec.name;
        for v in iter_vec.val.iter() {
            yield HashMap::from([(name.clone(), v.clone())]);
        }
    })
}

async fn run_task(task: Task) -> Result<()> {
    match task {
        Task::HttpTask(http_task) => {run_task_http(http_task).await}
        Task::HtmlTask(html_task) => {run_task_html(html_task).await}
    }
}

async fn run_task_html(mut task: HtmlTask) -> Result<()> {
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
    Ok(())
}

async fn run_task_http(task: HttpTask) -> Result<()> {
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

fn get_handlebars(s: &str, env: &HashMap<String, String>) -> Result<String> {
    let mut handlebars = Handlebars::new();
    handlebars.register_template_string("output", s)?;
    Ok(handlebars.render("output", &env)?)
}

fn get_handlebars_safe_dir(s: &str, env: &HashMap<String, String>) -> Result<String> {
    let mut new_env = env.clone();
    for (_k, v) in new_env.iter_mut() {
        *v = sanitize(v.clone());
    }
    let mut handlebars = Handlebars::new();
    handlebars.register_template_string("output", s)?;
    Ok(handlebars.render("output", &new_env)?)
}

fn exist_node(dag: &Graph<String, ()>, edge: &String) -> bool {
    !dag.node_indices()
        .any(|idx| dag.node_weight(idx).map_or(false, |w| w == edge))
}

fn find_node(dag: &Graph<String, ()>, edge: &String) -> Option<petgraph::graph::NodeIndex> {
    dag.node_indices()
        .find(|&idx| dag.node_weight(idx).map_or(false, |w| w == edge))
}


fn from_unix_time(s: String) -> Result<String> {
    let timestamp_ms: i64 = s.parse::<i64>()?;
    let timestamp_sec = timestamp_ms / 1000;
    let timestamp_nano = (timestamp_ms % 1000) * 1_000_000;

    let Some(datetime_utc) = DateTime::<Utc>::from_timestamp(timestamp_sec, timestamp_nano as u32) else { return Ok(s) } ;

    let datetime_local = datetime_utc.with_timezone(&Local);
    Ok(datetime_local.format("%Y-%m-%d %H:%M:%S").to_string())
}