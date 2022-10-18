use std::sync::{Arc, Mutex};

pub fn make_arc<T>(value: T) -> Arc<Mutex<T>> {
    Arc::new(Mutex::new(value))
}
