mod models;
mod crawler;

use tauri_specta::{collect_commands, Builder};
use std::sync::{Arc, atomic::Ordering};
use tauri::State;
// use tauri::{State, Manager, Listener};
use tokio::sync::{RwLock};
use crate::crawler::Crawler;
use crate::models::{ApiError, TextContent, Setting};

type Result<T> = std::result::Result<T, ApiError>;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
#[specta::specta]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
#[specta::specta]
async fn get_arg_path(state: State<'_, Arc<RwLock<Crawler>>>) -> Result<Option<String>> {
    let crawler = state.read().await;
    Ok(crawler.get_arg_path())
}

#[allow(dead_code)]
#[tauri::command]
#[specta::specta]
async fn pause(
    state: State<'_, Arc<RwLock<Crawler>>>,
    step_name: String
) -> Result<()> {
    println!("pause");
    let crawler = state.read().await;
    let mut map = crawler.step_handles.write().await;
    let step_handle_map = map.get_mut(&step_name).unwrap();
    step_handle_map.paused.store(true, Ordering::SeqCst);
    Ok(())
}

#[allow(dead_code)]
#[tauri::command]
#[specta::specta]
async fn resume(
    state: State<'_, Arc<RwLock<Crawler>>>,
    step_name: String
) -> Result<()> {
    println!("resume");
    let crawler = state.read().await;
    let mut map = crawler.step_handles.write().await;
    let step_handle_map = map.get_mut(&step_name).unwrap();

    step_handle_map.paused.store(false, Ordering::SeqCst);
    step_handle_map.notifier.notify_one();
    Ok(())
}


#[tauri::command]
#[specta::specta]
async fn load_crawler(state: State<'_, Arc<RwLock<Crawler>>>, setting: Setting) -> Result<()> {
    let mut crawler = state.write().await;
    let _ = crawler.load(setting).await;
    Ok(())
}

#[tauri::command]
#[specta::specta]
async fn read_txt(state: State<'_, Arc<RwLock<Crawler>>>, path_str: &str) -> Result<TextContent> {
    let crawler = state.read().await;
    let text_content = crawler.read_txt(path_str).await?;
    Ok(text_content)
}


#[tauri::command]
#[specta::specta]
async fn run_step(state: State<'_, Arc<RwLock<Crawler>>>, step_name: &str) -> Result<()> {
    println!("run_step: {}", step_name);
    let mut crawler = state.write().await;
    crawler.run_step(String::from(step_name)).await?;
    Ok(())
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {

    let builder = Builder::<tauri::Wry>::new()
        .commands(collect_commands![greet, get_arg_path, read_txt, load_crawler, run_step]);

    #[cfg(debug_assertions)] // <- Only export on non-release builds
    {
        use specta_typescript::BigIntExportBehavior;
        use specta_typescript::Typescript;
        let ts = Typescript::default()
            .bigint(BigIntExportBehavior::Number);
        builder
            .export(ts, "../src/bindings.ts")
            .expect("Failed to export typescript bindings");
    }

    tauri::Builder::default()
        .manage(Arc::new(RwLock::new(Crawler::new())))
        // .manage(Arc::new(AtomicBool::new(false)))
        // .manage(Arc::new(Notify::new()))
        .plugin(tauri_plugin_opener::init())
        // .invoke_handler(tauri::generate_handler![greet])
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
