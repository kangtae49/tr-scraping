use std::collections::{HashMap, VecDeque};
use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::path::{absolute, Path, PathBuf};
use std::pin::Pin;
use std::str::FromStr;

use tokio::{sync::Semaphore};
use tokio::io::AsyncReadExt;
use tokio::sync::{Notify, RwLock};
use tokio_stream::{Stream, StreamExt};
use async_stream::stream;
use glob::glob;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::{Client, RequestBuilder};
use encoding_rs::Encoding;
use chardetng::EncodingDetector;
use handlebars::Handlebars;
use petgraph::graph::Graph;
use mime_guess::from_path;
use serde_json::Value;
use sanitize_filename::sanitize;
use mime::Mime;

use crate::models::{ApiError, Edge, Step, StepHandle, TextContent, Setting, Task, TaskIter, IterRange, IterList, IterPattern, IterRangePattern, IterGlobJsonPattern, IterJsonRangePattern};


type ItemData = HashMap<String, String>;

type Result<T> = std::result::Result<T, ApiError>;


type Shared<T> = Arc<RwLock<T>>;

pub struct Crawler {
    pub client: Client,
    pub env: Shared<HashMap<String, String>>,
    pub header: Shared<HashMap<String, String>>,
    pub steps: Shared<HashMap<String, Step>>,
    pub step_handles: Shared<HashMap<String, StepHandle>>,
    pub dag: Shared<Graph<String, ()>>
}

impl Crawler  {
    pub fn new() -> Self {
        Crawler {
            client: Client::new(),
            env: Arc::new(RwLock::new(HashMap::new())),
            header: Arc::new(RwLock::new(HashMap::new())),
            steps: Arc::new(RwLock::new(HashMap::new())),
            step_handles: Arc::new(RwLock::new(HashMap::new())),
            dag: Arc::new(RwLock::new(Graph::<String, ()>::new())),
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
                },
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
            None => from_path(path_str).first_or_octet_stream().to_string()
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
                text: None
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
                text: opt_text
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
                semaphore: Arc::new(Semaphore::new(concurrency_limit))
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
        println!("crawler run_step: {}", &step_name);
        let steps_arc = Arc::clone(&self.steps);
        let steps = steps_arc.read().await;
        let env_arc = Arc::clone(&self.env);
        let header_arc = Arc::clone(&self.header);

        let step = steps.get(&step_name).ok_or(ApiError::CrawlerError("Step not found".to_string()))?;
        let req = step.req.clone();
        let env_lock = env_arc.read().await;
        let mut env = env_lock.clone();
        let header_lock = header_arc.read().await;
        let g_header = header_lock.clone();

        let step_handles_arc = self.step_handles.clone();
        let step_handles = step_handles_arc.read().await;
        let step_handle = step_handles.get(&step_name).ok_or(ApiError::CrawlerError("Step not found".to_string()))?;
        let semaphore = step_handle.semaphore.clone();
        let paused = step_handle.paused.clone();

        let mut task_iters = step.task_iters.clone();
        if task_iters.is_empty() {
            task_iters.push(TaskIter::Range(IterRange{
                name: format!("IDX_{}", &step_name),
                offset: "0".to_string(),
                take: "1".to_string(),
            }))
        }

        paused.store(false, Ordering::SeqCst);

        let mut handles = Vec::new();

        let mut stream = get_iters(&task_iters, &mut env).await?;
        while let Some((vals, cur_env)) = stream.next().await {
            println!("iter: {:?}", vals);
            if paused.load(Ordering::SeqCst) {
                println!("paused");
                return Ok(());
            }

            let client = self.client.clone();

            let url = get_handlebars(&req.url, &cur_env)?;

            let method = req.method.clone();

            let mut header = HeaderMap::new();
            for (k, v) in g_header.iter() {
                let nm = HeaderName::from_str(k.as_str())?;
                let new_v = get_handlebars(v, &cur_env)?;
                let val = HeaderValue::from_str(&new_v)?;
                header.insert(nm, val);
            }
            
            for (k, v) in req.header.iter() {
                let nm = HeaderName::from_str(k.as_str())?;
                let new_v = get_handlebars(v, &cur_env)?;
                let val = HeaderValue::from_str(&new_v)?;
                header.insert(nm, val);
            }
            let folder = get_handlebars_safe_dir(&step.output, &cur_env)?;
            std::fs::create_dir_all(Path::new(&folder))?;
            let filename = sanitize(get_handlebars(&req.filename, &cur_env)?);
            let p: PathBuf = Path::new(&folder).join(filename);
            let save_path= p.to_string_lossy().to_string();

            let task = Task {
                client,
                url,
                method,
                header,
                save_path,
            };

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
        Ok(())
    }

}

async fn get_iters<'a>(task_iters: &'a Vec<TaskIter>, env: &'a mut HashMap<String, String>) -> Result<Pin<Box<impl Stream<Item=(Vec<Option<ItemData>>, HashMap<String, String>)> + 'a>>> {
    let mut cur_vals: Vec<Option<ItemData>> = Vec::new();
    let mut iters: Vec<VecDeque<ItemData>> = Vec::new();
    for i in 0..task_iters.len() {
        let list = get_iter(&task_iters[i], env).unwrap_or_else(|_e| vec![]);
        iters.push(VecDeque::from(list));
        let cur_val = iters[i].pop_front();
        cur_vals.push(cur_val.clone());
        if let Some(val) = cur_val.clone() {
            env.extend(val.clone());
        }
    }

    let len = iters.len();
    Ok(Box::pin(stream! {
        loop {
            if !cur_vals.iter().any(|v| v.is_none()) {
                yield (cur_vals.clone(), env.clone());
            }

            for i in 0..len {
                let idx = len - 1 - i;
                let cur_val = iters[idx].pop_front();
                cur_vals[idx] = cur_val.clone();
                if let Some(val) = cur_val.clone() {
                    env.extend(val.clone());
                    break;
                }
            }
            if cur_vals[0].is_none() {
                break;
            }
            for i in 0..len {
                if cur_vals[i].is_none() {
                    if let Ok(list) = get_iter(&task_iters[i], env) {
                        iters[i] = VecDeque::from(list);
                        let cur_val = iters[i].pop_front();
                        cur_vals[i] = cur_val.clone();
                        if let Some(val) = cur_val.clone() {
                            env.extend(val.clone());
                        }
                    }
                }
            }
        }
    }))
}

fn get_iter(task_iter: &TaskIter, env: &HashMap<String, String>) -> Result<Vec<ItemData>> {
    let list = match task_iter {
        TaskIter::Vec(iter_vec) => {
            get_iter_vec(iter_vec)?
        }
        TaskIter::Range(iter_range) => {
            get_iter_range(iter_range, env)?
        }
        TaskIter::Pattern(iter_pattern) => {
            get_iter_pattern(iter_pattern)?
        }
        TaskIter::RangePattern(iter_range_pattern) => {
            get_iter_range_pattern(iter_range_pattern, env)?
        }
        TaskIter::GlobJsonPattern(iter_glob_json_pattern) => {
            get_iter_glob_json_pattern(iter_glob_json_pattern, env)?
        }
        TaskIter::GlobJsonRangePattern(iter_glob_json_range_pattern) => {
            get_iter_glob_json_range_pattern(iter_glob_json_range_pattern, env)?
        }
    };
    Ok(list)
}

fn get_iter_glob_json_pattern(iter_glob_json_pattern: &IterGlobJsonPattern, env: &HashMap<String, String>) -> Result<Vec<ItemData>> {
    let mut glob_pattern = iter_glob_json_pattern.glob_pattern.clone();
    let mut item_pattern = iter_glob_json_pattern.item_pattern.clone();
    let mut env_pattern = iter_glob_json_pattern.env_pattern.clone();

    glob_pattern = get_handlebars(&glob_pattern, env)?;
    item_pattern = get_handlebars(&item_pattern, env)?;
    for (_k, v) in env_pattern.iter_mut() {
        *v = get_handlebars(&v, env)?;
    }

    let paths = glob(&glob_pattern)?;
    let mut ret = vec![];
    for entry in paths {
        let Ok(p) = entry else { continue };
        let Ok(json_str) = std::fs::read_to_string(p) else { continue };
        let Ok(json) = serde_json::from_str(&json_str) else { continue };
        let Ok(item_vals) = jsonpath_lib::select(&json, &item_pattern) else { continue };
        for item in item_vals {
            let mut env_item = HashMap::new();
            for (k, v) in env_pattern.iter() {
                if let Some(j_val) = get_json_val(item, v) {
                    env_item.insert(k.to_string(), j_val);
                }
            }
            ret.push(env_item);
        }
    }
    Ok(ret)

}

fn get_iter_glob_json_range_pattern(iter_glob_json_range_pattern: &IterJsonRangePattern, env: &HashMap<String, String>) -> Result<Vec<ItemData>> {
    let name = &iter_glob_json_range_pattern.name;
    let mut file_pattern = iter_glob_json_range_pattern.file_pattern.clone();
    let mut offset_pattern = iter_glob_json_range_pattern.offset_pattern.clone();
    let mut take_pattern = iter_glob_json_range_pattern.take_pattern.clone();

    file_pattern = get_handlebars(&file_pattern, env)?;
    offset_pattern = get_handlebars(&offset_pattern, env)?;
    take_pattern = get_handlebars(&take_pattern, env)?;

    let mut paths = glob(&file_pattern)?;
    let mut ret = vec![];
    let entry = paths.next().ok_or(ApiError::JsonError("path error".into()))?;
    let p = entry.map_err(|_| ApiError::JsonError("path error".into()))?;
    let json_str = std::fs::read_to_string(p).map_err(|_| ApiError::JsonError("read error".into()))?;
    let json = serde_json::from_str(&json_str).map_err(|_| ApiError::JsonError("json error".into()))?;
    let offset_str = get_json_val(&json, &offset_pattern).unwrap_or(offset_pattern);
    let take_str = get_json_val(&json, &take_pattern).unwrap_or(take_pattern);
    let offset: usize = offset_str.parse()?;
    let take: usize = take_str.parse()?;

    let start = offset;
    let end = offset + take;
    for i in start..end {
        ret.push(HashMap::from([(name.to_string(), i.to_string())]));
    }
    Ok(ret)

}

fn get_iter_range(iter_range: &IterRange, env: &HashMap<String, String>) -> Result<Vec<ItemData>> {
    let name = &iter_range.name;
    let offset_str = get_handlebars(&iter_range.offset, env)?;
    let take_str = get_handlebars(&iter_range.take, env)?;
    let offset: usize = offset_str.parse()?;
    let take: usize = take_str.parse()?;
    let start = offset;
    let end = offset + take;
    let mut ret = vec![];
    for i in start..end {
        ret.push(HashMap::from([(name.to_string(), i.to_string())]));
    }
    Ok(ret)
}

fn get_iter_range_pattern(iter_range_pattern: &IterRangePattern, env: &HashMap<String, String>) -> Result<Vec<ItemData>> {
    let name = &iter_range_pattern.name;
    let glob_pattern = iter_range_pattern.glob_pattern.clone();
    let file_path = get_handlebars(&glob_pattern, env)?;
    let mut offset_str = get_handlebars(&iter_range_pattern.offset, env)?;
    let mut take_str = get_handlebars(&iter_range_pattern.take, env)?;

    if let Ok(json_str) = std::fs::read_to_string(Path::new(&file_path)) {
        if let Ok(json) = serde_json::from_str(&json_str) {
            match get_json_val(&json, &offset_str) {
                Some(val) => {offset_str = val;}
                None => {}
            }
            match get_json_val(&json, &take_str) {
                Some(val) => {take_str = val;}
                None => {}
            }
        }
    }
    let offset: usize = offset_str.parse()?;
    let take: usize = take_str.parse()?;
    let start = offset;
    let end = offset + take;

    let mut ret = vec![];
    for i in start..end {
        ret.push(HashMap::from([(name.to_string(), i.to_string())]));
    }
    Ok(ret)
}

fn get_json_val(json: &Value, path: &str) -> Option<String> {
    let Ok(values) = jsonpath_lib::select(json, path) else { return None };
    let Some(&val) = values.first() else { return None };
    match val {
        Value::String(s) => Some(s.clone().trim().to_string()),
        _ => Some(val.to_string().trim().to_string()),
    }
}

fn get_iter_pattern(iter_pattern: &IterPattern) -> Result<Vec<ItemData>> {
    let name = &iter_pattern.name;
    let glob_pattern = &iter_pattern.glob_pattern;
    let content_pattern = &iter_pattern.content_pattern;
    let paths = glob(glob_pattern)?;
    let mut ret = vec![];
    for entry in paths {
        let Ok(p) = entry else { continue };
        let Ok(json_str) = std::fs::read_to_string(p) else { continue };
        let Ok(json) = serde_json::from_str(&json_str) else { continue };
        let Ok(values) = jsonpath_lib::select(&json, content_pattern) else { continue };
        for val in values {
            ret.push(HashMap::from([(name.to_string(), val.to_string())]));
        }
    }
    Ok(ret)
}

fn get_iter_vec(iter_vec: &IterList) -> Result<Vec<ItemData>> {
    let mut ret = vec![];
    for val in iter_vec.val.clone() {
        ret.push(HashMap::from([(val.to_string(), val.to_string())]));
    }
    Ok(ret)
}

async fn run_task(task: Task) -> Result<()> {
    let save_path = task.save_path.clone();
    let tmp_path = format!("{}.tmp", &save_path);
    let p = Path::new(&save_path);
    let p_tmp = Path::new(tmp_path.as_str());
    if p.exists() {
        return Ok(())
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
        println!("run_task err: {:?}", &task);
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
        charset = content_type.clone()
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
    !dag.node_indices().any(|idx| dag.node_weight(idx).map_or(false, |w| w == edge))
}

fn find_node(dag: &Graph<String, ()>, edge: &String) -> Option<petgraph::graph::NodeIndex> {
    dag.node_indices().find(|&idx| dag.node_weight(idx).map_or(false, |w| w == edge))
}
