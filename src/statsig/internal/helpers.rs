use std::sync::{Arc, Mutex};


pub fn make_arc<T>(value: T) -> Arc<Mutex<T>> {
    Arc::new(Mutex::new(value))
}

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