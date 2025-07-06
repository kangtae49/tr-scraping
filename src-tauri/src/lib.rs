mod crawler;
mod models;

use std::sync::{atomic::Ordering, Arc};
use tauri::State;
use tauri_specta::{collect_commands, Builder};
// use tauri::{State, Manager, Listener};
use crate::crawler::Crawler;
use crate::crawler::save_file;
use crate::models::{ApiError, Setting, TextContent};
use tokio::sync::RwLock;
// use tauri::api::dialog::FileDialogBuilder;

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

// #[allow(dead_code)]
// #[tauri::command]
// #[specta::specta]
// async fn stop_step(state: State<'_, Arc<RwLock<Crawler>>>, step_name: String) -> Result<()> {
//     println!("stop_step");
//     let crawler = state.read().await;
//     let mut map = crawler.step_handles.write().await;
//     let step_handle_map = map.get_mut(&step_name).unwrap();
//     step_handle_map.paused.store(true, Ordering::SeqCst);
//     Ok(())
// }


// #[allow(dead_code)]
// #[tauri::command]
// #[specta::specta]
// async fn resume(state: State<'_, Arc<RwLock<Crawler>>>, step_name: String) -> Result<()> {
//     println!("resume");
//     let crawler = state.read().await;
//     let mut map = crawler.step_handles.write().await;
//     let step_handle_map = map.get_mut(&step_name).unwrap();
//
//     step_handle_map.paused.store(false, Ordering::SeqCst);
//     step_handle_map.notifier.notify_one();
//     Ok(())
// }

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
    let crawler = state.read().await;
    crawler.run_step(String::from(step_name)).await?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
async fn save_setting(file_path: String, txt: String) -> Result<()> {
    Ok(save_file(file_path, txt).await?)
}

#[tauri::command]
#[specta::specta]
async fn pause_step(state: State<'_, Arc<RwLock<Crawler>>>, step_name: &str) -> Result<()> {
    println!("lib pause_step start");
    let crawler = state.read().await;
    println!("lib pause_step 1");
    crawler.pause_step(step_name.to_string(), true).await?;
    println!("lib pause_step end");
    Ok(())
}

#[tauri::command]
#[specta::specta]
async fn get_pause_step(state: State<'_, Arc<RwLock<Crawler>>>, step_name: &str) -> Result<bool> {
    let crawler = state.read().await;
    Ok(crawler.get_pause_step(step_name.to_string()).await?)
}




#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = Builder::<tauri::Wry>::new().commands(collect_commands![
        greet,
        get_arg_path,
        read_txt,
        load_crawler,
        run_step,
        // stop_step,
        save_setting,
        pause_step,
        get_pause_step,
        // run_output_html,
        // stop_output_html
    ]);

    #[cfg(debug_assertions)] // <- Only export on non-release builds
    {
        use specta_typescript::BigIntExportBehavior;
        use specta_typescript::Typescript;
        let ts = Typescript::default().bigint(BigIntExportBehavior::Number);
        builder
            .export(ts, "../src/bindings.ts")
            .expect("Failed to export typescript bindings");

        let schema = schemars::schema_for!(Setting);
        let json_schema = serde_json::to_string_pretty(&schema).unwrap();
        let _ =
            std::fs::write("../sample/setting.schema.json", json_schema).map_err(|e| println!("{:?}", e));
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
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
