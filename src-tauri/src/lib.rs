mod scraping;
mod models;
mod tasks;
mod utils;
mod iters;

use std::sync::{Arc};
use tauri::State;
use tauri_specta::{collect_commands, Builder};
use crate::scraping::Scraping;
use crate::utils::save_file;
use crate::models::{ApiError, Setting, TextContent};
use tokio::sync::RwLock;

type Result<T> = std::result::Result<T, ApiError>;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
#[specta::specta]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
#[specta::specta]
async fn get_arg_path(state: State<'_, Arc<RwLock<Scraping>>>) -> Result<Option<String>> {
    let scraping = state.read().await;
    Ok(scraping.get_arg_path())
}

#[tauri::command]
#[specta::specta]
async fn load_setting(state: State<'_, Arc<RwLock<Scraping>>>, setting: Setting) -> Result<()> {
    let mut scraping = state.write().await;
    let _ = scraping.load(setting).await;
    Ok(())
}

#[tauri::command]
#[specta::specta]
async fn read_txt(state: State<'_, Arc<RwLock<Scraping>>>, path_str: &str) -> Result<TextContent> {
    let scraping = state.read().await;
    let text_content = scraping.read_txt(path_str).await?;
    Ok(text_content)
}

#[tauri::command]
#[specta::specta]
async fn run_step(state: State<'_, Arc<RwLock<Scraping>>>, window: tauri::Window, step_name: &str) -> Result<()> {
    println!("run_step: {}", step_name);
    let scraping = state.read().await;
    scraping.run_step(String::from(step_name), window).await?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
async fn save_setting(file_path: String, txt: String) -> Result<()> {
    Ok(save_file(file_path, txt).await?)
}

#[tauri::command]
#[specta::specta]
async fn update_state(state: State<'_, Arc<RwLock<Scraping>>>, step_name: &str, val: u8) -> Result<()> {
    let scraping = state.read().await;
    scraping.update_state(step_name.to_string(), val).await?;
    Ok(())
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = Builder::<tauri::Wry>::new().commands(collect_commands![
        greet,
        get_arg_path,
        read_txt,
        load_setting,
        run_step,
        save_setting,
        update_state,
    ]);

    #[cfg(debug_assertions)] // <- Only export on non-release builds
    {
        use specta_typescript::BigIntExportBehavior;
        use specta_typescript::Typescript;
        use specta::{TypeCollection};
        use crate::models::StepNotify;

        let mut types = TypeCollection::default();
        types.register::<StepNotify>();
        Typescript::default()
            .export_to("../src/bindings_etc.ts", &types)
            .unwrap();

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
        .manage(Arc::new(RwLock::new(Scraping::new())))
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
