use std::collections::HashMap;
use serde_json::json;

pub trait UsizeExt {
    fn post_inc(&mut self) -> Self;
}

impl UsizeExt for usize {
    fn post_inc(&mut self) -> Self {
        let was = self.clone();
        *self += 1;
        return was;
    }
}
