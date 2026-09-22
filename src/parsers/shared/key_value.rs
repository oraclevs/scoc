use super::normalize::{normalized_key, scalar};
use serde_json::{Map, Value};

pub(crate) fn key_value_sections(lines: &[&str], separator: char) -> Vec<Map<String, Value>> {
    let mut sections = Vec::new();
    let mut current = Map::new();
    for line in lines {
        let line = line.trim_end();
        if line.trim().is_empty() {
            if !current.is_empty() {
                sections.push(std::mem::take(&mut current));
            }
            continue;
        }
        if let Some((key, value)) = line.split_once(separator) {
            let key = normalized_key(key);
            let value = scalar(value.trim());
            if let Some(existing) = current.remove(&key) {
                let mut values = match existing {
                    Value::Array(v) => v,
                    other => vec![other],
                };
                values.push(value);
                current.insert(key, Value::Array(values));
            } else {
                current.insert(key, value);
            }
        }
    }
    if !current.is_empty() {
        sections.push(current);
    }
    sections
}
