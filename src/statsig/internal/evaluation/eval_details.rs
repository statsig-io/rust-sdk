use chrono::Utc;
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
        EvalDetails {
            reason: EvaluationReason::Uninitialized,
            config_sync_time: 0,
            init_time: 0,
            server_time: Utc::now().timestamp_millis() as u64,
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
