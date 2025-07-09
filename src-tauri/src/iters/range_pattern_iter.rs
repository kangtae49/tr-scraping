use std::collections::HashMap;
use std::path::Path;
use std::pin::Pin;
use async_stream::stream;
use tokio_stream::Stream;
use crate::models::{ItemData, IterRangePattern};
use crate::utils::{get_handlebars, get_json_val};
pub fn get_iter_range_pattern(
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

