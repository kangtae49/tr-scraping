use std::collections::{HashMap};
use std::path::{absolute, PathBuf};
use std::pin::Pin;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Condvar, Mutex};

use async_stream::stream;
use chardetng::EncodingDetector;
use encoding_rs::Encoding;
use mime_guess::from_path;
use reqwest::{Client};
use tokio::io::{AsyncReadExt};
use tokio::sync::Semaphore;
use tokio::sync::{RwLock};
use tokio_stream::{Stream, StreamExt};
use tauri::Emitter;

use crate::tasks::task::{Task};
use crate::models::{Result, ApiError, IterRange,
                    Setting, Step, StepHandle, TaskIter,
                    TextContent, StepNotify,
                    Shared, ItemData,
                    STEP_RUNNING, STEP_STOPPED, STEP_PAUSED
};

use crate::iters::vec_iter::get_iter_vec;
use crate::iters::range_iter::get_iter_range;
use crate::iters::pattern_iter::get_iter_pattern;
use crate::iters::range_pattern_iter::get_iter_range_pattern;
use crate::iters::glob_json_range_pattern_iter::get_iter_glob_json_range_pattern;
use crate::iters::glob_json_pattern_iter::get_iter_glob_json_pattern;

pub struct Scraping {
    pub client: Client,
    pub env: Shared<HashMap<String, String>>,
    pub header: Shared<HashMap<String, String>>,
    pub steps: Shared<HashMap<String, Step>>,
    pub step_handles: Shared<HashMap<String, StepHandle>>,
}

impl Scraping {
    pub fn new() -> Self {
        Scraping {
            client: Client::new(),
            env: Arc::new(RwLock::new(HashMap::new())),
            header: Arc::new(RwLock::new(HashMap::new())),
            steps: Arc::new(RwLock::new(HashMap::new())),
            step_handles: Arc::new(RwLock::new(HashMap::new())),
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
        for (nm, step) in setting.steps.iter() {
            let concurrency_limit = step.concurrency_limit;
            let step_handle = StepHandle {
                name: nm.clone(),
                semaphore: Arc::new(Semaphore::new(concurrency_limit)),
                // stop: Arc::new(AtomicBool::new(false)),
                state: Arc::new(AtomicU8::new(STEP_RUNNING)),
                control: Arc::new((Mutex::new(()), Condvar::new()))
            };
            step_handles.insert(nm.clone(), step_handle);
        }

        self.assign(&self.env, setting.env).await;
        self.assign(&self.header, setting.header).await;
        self.assign(&self.steps, setting.steps).await;
        self.assign(&self.step_handles, step_handles).await;

        Ok(())
    }

    pub async fn update_state(&self, step_name: String, val: u8) -> Result<()> {
        let step_handles = self.step_handles.read().await;
        let step_handle = step_handles
            .get(&step_name)
            .ok_or(ApiError::ScrapingError("Step not found".to_string()))?;
        let state = step_handle.state.clone();
        state.store(val, Ordering::SeqCst);
        let control = step_handle.control.clone();
        let (_, cvar) = &*control;
        cvar.notify_all();
        Ok(())
    }

    pub async fn run_step(&self, step_name: String, window: tauri::Window) -> Result<()> {
        println!("Start Step: {}", &step_name);
        let notify = StepNotify {
            name: "status".to_string(),
            status: "start".to_string(),
            message: format!("Start Step {}", &step_name)
        };
        let window_clone = window.clone();
        window_clone.emit("status", notify).unwrap();

        let steps = self.steps.read().await;
        let step = steps
            .get(&step_name)
            .ok_or(ApiError::ScrapingError("Step not found".to_string()))?;
        let mut job = step.job.clone();
        let env_lock = self.env.read().await;
        let env = env_lock.clone();
        let header_lock = self.header.read().await;
        let g_header = header_lock.clone();

        let mut task_iters = step.task_iters.clone();
        if task_iters.is_empty() {
            task_iters.push(TaskIter::Range(IterRange {
                name: format!("IDX_{}", &step_name),
                offset: "0".to_string(),
                take: "1".to_string(),
            }))
        }
        job.pre_process()?;

        let step_handles = self.step_handles.read().await;
        let step_handle = step_handles
            .get(&step_name)
            .ok_or(ApiError::ScrapingError("Step not found".to_string()))?;
        let semaphore = step_handle.semaphore.clone();

        let state = step_handle.state.clone();
        state.store(STEP_RUNNING, Ordering::SeqCst);
        let control = step_handle.control.clone();

        let mut handles = Vec::new();

        let mut stream = get_iters(task_iters, env.clone());
        while let Some((vals, cur_env)) = stream.next().await {
            println!("iter: {:?}", vals);
            let semaphore = semaphore.clone();
            let Ok(permit) = semaphore.acquire_owned().await else { return Err(ApiError::ScrapingError("err semaphore.acquire_owned".to_string())); };

            match state.load(Ordering::SeqCst) {
                STEP_RUNNING => {
                    println!("STEP_RUNNING");
                }
                STEP_PAUSED => {
                    println!("STEP_PAUSED");
                    let (lock, cvar) = &*control;
                    let _guard = cvar
                        .wait_while(lock.lock().unwrap(), |_| {
                            state.load(Ordering::SeqCst) == STEP_PAUSED
                        })
                        .unwrap();
                }
                STEP_STOPPED => {
                    println!("STEP_STOPPED");
                    break;
                }
                _ => {}
            }

            let window_clone = window.clone();
            let task = job.make_task(cur_env, self.client.clone(), g_header.clone()).await?;
            let handle = tokio::task::spawn(async move {
                if let Err(e) = task.clone().run_task().await {
                    eprintln!("Error: {:?}", e);
                    let notify = StepNotify {
                        name: "error".to_string(),
                        status: "".to_string(),
                        message: format!("{:?}", e)
                    };
                    let window_clone = window_clone.clone();
                    window_clone.emit("error", notify).unwrap();
                }

                let task_notify = match task.clone() {
                    Task::HttpTask(http_task) => {
                        StepNotify {
                            name: "progress".to_string(),
                            status: "".to_string(),
                            message: http_task.save_path.clone()
                        }
                    }
                    Task::HtmlTask(html_task) => {
                        StepNotify {
                            name: "progress".to_string(),
                            status: "".to_string(),
                            message: html_task.save_path.clone()
                        }
                    }
                    Task::CsvTask(csv_task) => {
                        StepNotify {
                            name: "progress".to_string(),
                            status: "".to_string(),
                            message: csv_task.save_path.clone()
                        }
                    }
                    Task::ShellTask(shell_task) => {
                        StepNotify {
                            name: "progress".to_string(),
                            status: "".to_string(),
                            message: format!("{} {:?}", shell_task.shell, shell_task.args)
                        }
                    }
                };
                window_clone.emit(&task_notify.name.clone(), task_notify.clone()).unwrap();
                drop(permit);
            });
            handles.push(handle);
        }

        for handle in handles {
            match handle.await {
                Ok(_) => {}
                Err(e) => eprintln!("Error: {:?}", e),
            };
        }

        let notify = StepNotify {
            name: "status".to_string(),
            status: "end".to_string(),
            message: format!("End Step {}", &step_name)
        };
        let window_clone = window.clone();
        window_clone.emit("status", notify).unwrap();

        println!("End Step: {}", &step_name);
        Ok(())
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

