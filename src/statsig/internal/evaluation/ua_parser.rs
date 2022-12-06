use std::sync::{Arc, RwLock};
use serde_json::{json, Value};
use serde_json::Value::Null;
use uaparser::{Parser, UserAgentParser as ExtUserAgentParser};
use crate::{StatsigUser, unwrap_or_return};

pub struct UserAgentParser {
    parser: Arc<RwLock<Option<ExtUserAgentParser>>>,
}

impl UserAgentParser {
    pub fn new() -> Self {
        let mut inst = Self {
            parser: Arc::from(RwLock::from(None)),
        };
        inst.load_parser();
        inst
    }

    pub fn get_value_from_user_agent(&self, user: &StatsigUser, field: &Option<String>) -> Value {
        let field_lowered = match field {
            Some(f) => f.to_lowercase(),
            _ => return Null
        };

        let user_agent = match &user.user_agent {
            Some(ua) => ua,
            _ => return Null
        };

        if user_agent.len() > 1000 {
            return Null;
        }

        let lock = unwrap_or_return!(self.parser.read().ok(), Null);
        let parser = unwrap_or_return!(&*lock, Null);

        let parsed = parser.parse(user_agent);
        match field_lowered.as_str() {
            "os_name" | "osname" => json!(parsed.os.family),
            "os_version" | "osversion" => {
                let os = parsed.os;
                if let (Some(major), Some(minor), Some(patch)) = (os.major, os.minor, os.patch) {
                    return json!(format!("{}.{}.{}", major, minor, patch));
                }
                Null
            }
            "browser_name" | "browsername" => json!(parsed.user_agent.family),
            "browser_version" | "browserversion" => {
                let ua = parsed.user_agent;
                if let (Some(major), Some(minor), Some(patch)) = (ua.major, ua.minor, ua.patch) {
                    return json!(format!("{}.{}.{}", major, minor, patch));
                }
                Null
            }
            _ => Null
        }
    }

    fn load_parser(&mut self) {
        let parser = self.parser.clone();
        std::thread::spawn(move || {
            let mut lock = unwrap_or_return!(parser.write().ok(), ());
            *lock = Some(ExtUserAgentParser::from_bytes(include_bytes!("resources/ua_parser_regex.yaml")).expect("ua_parser"));
        });
    }
}