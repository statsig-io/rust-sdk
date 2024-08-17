use serde::Serialize;

use super::evaluation::eval_details::EvalDetails;
#[derive(Clone, Serialize)]
pub struct FeatureGate {
    pub name: String,
    pub value: bool,
    pub rule_id: String,
    pub evaluation_details: EvalDetails,
}
