use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs::OpenOptions;
use std::io::{Write};

use sanitize_filename::sanitize;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::tasks::task::Task;
use crate::utils::{get_handlebars, get_handlebars_safe_dir};
use crate::models::Result;

#[derive(Type, Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct CsvJob {
    pub keys: Vec<String>,
    pub sep: String,
    pub filename: String,
    pub output: String,
}

impl CsvJob {
    pub fn pre_process(&mut self) -> Result<()> {
        Ok(())
    }

    pub async fn make_task(&self, cur_env: HashMap<String, String>) -> Result<Task> {

        let folder = get_handlebars_safe_dir(&self.output, &cur_env)?;
        let filename = sanitize(get_handlebars(&self.filename, &cur_env)?);
        let p: PathBuf = Path::new(&folder).join(filename);
        let save_path = p.to_string_lossy().to_string();
        Ok(Task::CsvTask(CsvTask {
            cur_env: cur_env.clone(),
            keys: self.keys.clone(),
            sep: self.sep.clone(),
            folder,
            save_path,
        }))
    }
}


#[derive(Clone, Debug)]
pub struct CsvTask {
    pub cur_env: HashMap<String, String>,
    pub keys: Vec<String>,
    pub sep: String,
    pub folder: String,
    pub save_path: String,
}


impl CsvTask {
    pub async fn run(&mut self) -> Result<()> {
        let folder = self.folder.clone();
        let p_folder = Path::new(&folder);
        if !p_folder.exists() {
            std::fs::create_dir_all(Path::new(&folder))?;
        }

        let save_path = self.save_path.clone();
        let p = Path::new(&save_path);

        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(p)?;

        let mut cols = Vec::<String>::new();
        for key in &self.keys {
            let default_value = "".to_string();
            let value = self.cur_env.get(key).unwrap_or(&default_value);
            let val = value.trim().to_string();
            cols.push(val);
        }
        let row = cols.join(&self.sep);
        writeln!(file, "{}", row)?;
        Ok(())
    }
}


