use std::collections::HashMap;
use std::pin::Pin;
use async_stream::stream;
use tokio_stream::Stream;
use crate::models::{ItemData, IterRange};
use crate::utils::{get_handlebars};

pub fn get_iter_range(iter_range: IterRange, env: HashMap<String, String>) -> Pin<Box<dyn Stream<Item = ItemData> + Send>> {
    Box::pin(stream! {
        let name = iter_range.name;
        let offset_pattern = iter_range.offset;
        let take_pattern = iter_range.take;
        let Ok(offset_str) = get_handlebars(&offset_pattern, &env) else { return ; };
        let Ok(take_str) = get_handlebars(&take_pattern, &env) else { return ; };
        let Ok(offset) = offset_str.parse::<usize>() else { return ; };
        let Ok(take) = take_str.parse::<usize>() else { return ;};
        let start = offset;
        let end = offset + take;
        for i in start..end {
            yield HashMap::from([(name.to_string(), i.to_string())]);
        }
    })
}

