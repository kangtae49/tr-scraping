use std::collections::HashMap;
use std::pin::Pin;
use async_stream::stream;
use tokio_stream::Stream;
use crate::models::{IterList, ItemData};

pub fn get_iter_vec(iter_vec: IterList) -> Pin<Box<dyn Stream<Item = ItemData> + Send>> {
    Box::pin(stream! {
        let name = iter_vec.name;
        for v in iter_vec.val.iter() {
            yield HashMap::from([(name.clone(), v.clone())]);
        }
    })
}