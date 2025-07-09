use std::collections::HashMap;
use std::pin::Pin;
use async_stream::stream;
use glob::glob;
use tokio_stream::Stream;
use crate::models::{ItemData, IterGlobJsonPattern};
use crate::utils::{get_handlebars, get_json_val};

pub fn get_iter_glob_json_pattern(
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
