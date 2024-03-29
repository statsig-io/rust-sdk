use serde_json::Value;
use serde_json::Value::Null;

use crate::statsig::internal::helpers::UsizeExt;
use crate::StatsigUser;

pub struct CountryLookup {
    country_codes: Vec<String>,
    ip_ranges: Vec<i64>,
}

impl CountryLookup {
    pub fn new() -> Self {
        let bytes = include_bytes!("resources/ip_supalite.table");

        let mut raw_code_lookup: Vec<String> = vec![];
        let mut country_codes: Vec<String> = vec![];
        let mut ip_ranges: Vec<i64> = vec![];

        let mut i = 0;

        while i < bytes.len() {
            let c1 = bytes[i.post_inc()] as char;
            let c2 = bytes[i.post_inc()] as char;

            raw_code_lookup.push(format!("{}{}", c1, c2));

            if c1 == '*' {
                break;
            }
        }

        let longs = |index: usize| bytes[index] as i64;

        let mut last_end_range = 0_i64;
        while (i + 1) < bytes.len() {
            let mut count: i64 = 0;
            let n1 = longs(i.post_inc());
            if n1 < 240 {
                count = n1;
            } else if n1 == 242 {
                let n2 = longs(i.post_inc());
                let n3 = longs(i.post_inc());
                count = n2 | (n3 << 8);
            } else if n1 == 243 {
                let n2 = longs(i.post_inc());
                let n3 = longs(i.post_inc());
                let n4 = longs(i.post_inc());
                count = n2 | (n3 << 8) | (n4 << 16);
            }

            last_end_range += count * 256;

            let cc = bytes[i.post_inc()] as usize;
            ip_ranges.push(last_end_range);
            country_codes.push(raw_code_lookup[cc].clone())
        }

        Self {
            country_codes,
            ip_ranges,
        }
    }

    pub fn get_value_from_ip(&self, user: &StatsigUser, field: &Option<String>) -> Value {
        let unwrapped_field = match field {
            Some(f) => f.as_str(),
            _ => return Null,
        };

        if unwrapped_field != "country" {
            return Null;
        }

        let ip = match &user.ip {
            Some(ip) => ip,
            _ => return Null,
        };

        match self.lookup(ip) {
            Some(cc) => Value::String(cc),
            _ => Null,
        }
    }

    fn lookup(&self, ip_address: &str) -> Option<String> {
        let parts: Vec<&str> = ip_address.split('.').collect();
        if parts.len() != 4 {
            return None;
        }

        let nums: Vec<Option<i64>> = parts.iter().map(|&x| x.parse().ok()).collect();
        if let (Some(n0), Some(n1), Some(n2), Some(n3)) = (nums[0], nums[1], nums[2], nums[3]) {
            let ip_number = (n0 * 256_i64.pow(3)) + (n1 << 16) + (n2 << 8) + n3;
            return self.lookup_numeric(ip_number);
        }

        None
    }

    fn lookup_numeric(&self, ip_address: i64) -> Option<String> {
        let index = self.binary_search(ip_address);
        let cc = self.country_codes[index].clone();
        if cc == "--" {
            return None;
        }
        Some(cc)
    }

    fn binary_search(&self, value: i64) -> usize {
        let mut min = 0;
        let mut max = self.ip_ranges.len();

        while min < max {
            let mid = (min + max) >> 1;
            if self.ip_ranges[mid] <= value {
                min = mid + 1;
            } else {
                max = mid;
            }
        }

        min
    }
}
