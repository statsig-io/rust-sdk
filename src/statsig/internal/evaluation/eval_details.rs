use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct EvalDetails {
    pub reason: EvaluationReason,
    pub config_sync_time: u64,
    pub init_time: u64,
    pub server_time: u64,
}

impl EvalDetails {
    pub fn default() -> Self {
        let curr_time = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(time) => time.as_secs() as u64,
            Err(_) => 0,
        };
        EvalDetails {
            reason: EvaluationReason::Uninitialized,
            config_sync_time: 0,
            init_time: 0,
            server_time: curr_time,
        }
    }
}
#[derive(Debug, Copy, Clone, PartialEq, Serialize)]
pub enum EvaluationReason {
    Network,
    DataAdapter,
    Uninitialized,
    Unrecognized,
    Unsupported,
}
