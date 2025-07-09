use std::collections::HashMap;
use std::io::Write;
use chrono::{DateTime, Local, Utc};
use handlebars::Handlebars;
use sanitize_filename::sanitize;
use serde_json::Value;

pub fn get_json_val(json: &Value, path: &str) -> Option<String> {
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

pub fn get_handlebars(s: &str, env: &HashMap<String, String>) -> crate::models::Result<String> {
    let mut handlebars = Handlebars::new();
    handlebars.register_template_string("output", s)?;
    Ok(handlebars.render("output", &env)?)
}

pub fn get_handlebars_safe_dir(s: &str, env: &HashMap<String, String>) -> crate::models::Result<String> {
    let mut new_env = env.clone();
    for (_k, v) in new_env.iter_mut() {
        *v = sanitize(v.clone());
    }
    let mut handlebars = Handlebars::new();
    handlebars.register_template_string("output", s)?;
    Ok(handlebars.render("output", &new_env)?)
}

pub fn from_unix_time(s: String) -> crate::models::Result<String> {
    let timestamp_ms: i64 = s.parse::<i64>()?;
    let timestamp_sec = timestamp_ms / 1000;
    let timestamp_nano = (timestamp_ms % 1000) * 1_000_000;

    let Some(datetime_utc) = DateTime::<Utc>::from_timestamp(timestamp_sec, timestamp_nano as u32) else { return Ok(s) } ;

    let datetime_local = datetime_utc.with_timezone(&Local);
    Ok(datetime_local.format("%Y-%m-%d %H:%M:%S").to_string())
}

pub async fn save_file(file_path: String, txt: String) -> crate::models::Result<()> {
    let mut file = std::fs::File::create(file_path)?;
    file.write_all(txt.as_bytes())?;
    Ok(())
}