use serde_json::{json, Value};
use serde_json::Value::Null;
use uaparser::{Parser};
use crate::StatsigUser;

pub struct UserAgentParser {
    parser: uaparser::UserAgentParser,
}

impl UserAgentParser {
    pub fn new() -> Self {
        let ua_regex_bytes = include_bytes!("resources/ua_parser_regex.yaml");

        Self {
            parser: uaparser::UserAgentParser::from_bytes(ua_regex_bytes)
                .expect("UserAgentParser creation failed"),
        }
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

        let parsed = self.parser.parse(user_agent);
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
}