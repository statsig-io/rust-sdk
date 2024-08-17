use serde::de::DeserializeOwned;

use super::evaluation::eval_details::EvalDetails;

pub struct DynamicConfig<T: DeserializeOwned> {
    pub name: String,
    pub value: Option<T>,
    pub rule_id: String,
    pub evaluation_details: EvalDetails
}
