use std::collections::HashMap;
use std::pin::Pin;
use async_stream::stream;
use glob::glob;
use tokio_stream::Stream;
use crate::models::{ItemData, IterPattern};

pub fn get_iter_pattern(iter_pattern: IterPattern) -> Pin<Box<dyn Stream<Item = ItemData> + Send>> {
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
