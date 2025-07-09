use std::collections::HashMap;
use std::pin::Pin;
use async_stream::stream;
use glob::glob;
use tokio_stream::Stream;
use crate::models::{ItemData, IterJsonRangePattern};
use crate::utils::{get_handlebars, get_json_val};

pub fn get_iter_glob_json_range_pattern(
    iter_glob_json_range_pattern: IterJsonRangePattern,
    env: HashMap<String, String>,
) -> Pin<Box<dyn Stream<Item = ItemData> + Send>> {
    Box::pin(stream! {
        let name = iter_glob_json_range_pattern.name;
        let file_pattern = iter_glob_json_range_pattern.file_pattern;
        let offset_pattern = iter_glob_json_range_pattern.offset_pattern;
        let take_pattern = iter_glob_json_range_pattern.take_pattern;

        let Ok(file_pattern) = get_handlebars(&file_pattern, &env) else { return ; };
        let Ok(offset_pattern) = get_handlebars(&offset_pattern, &env) else { return ;};
        let Ok(take_pattern) = get_handlebars(&take_pattern, &env) else { return; };

        let Ok(mut paths) = glob(&file_pattern) else { return ; };
        let Some(entry) = paths.next() else { return ; };
        let Ok(p) = entry else { return ; };
        let Ok(json_str) = std::fs::read_to_string(p) else { return; };
        let Ok(json) = serde_json::from_str(&json_str) else { return ; };
        let offset_str = get_json_val(&json, &offset_pattern).unwrap_or(offset_pattern);
        let take_str = get_json_val(&json, &take_pattern).unwrap_or(take_pattern);
        let Ok(offset) = offset_str.parse::<usize>() else { return ;};
        let Ok(take) = take_str.parse::<usize>() else {return ;};

        let start = offset;
        let end = offset + take;
        for i in start..end {
            yield HashMap::from([(name.to_string(), i.to_string())]);
        }
    })
}

