use serde::de::DeserializeOwned;

pub struct DynamicConfig<T: DeserializeOwned> {
    pub name: String,
    pub value: Option<T>,
    pub rule_id: String,
}
