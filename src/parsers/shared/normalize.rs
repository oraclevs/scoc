use serde_json::Value;

pub(crate) fn normalized_key(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut underscore = false;
    for ch in value.trim().chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            underscore = false;
        } else if !underscore && !out.is_empty() {
            out.push('_');
            underscore = true;
        }
    }
    while out.ends_with('_') {
        out.pop();
    }
    if out.is_empty() {
        "value".to_string()
    } else {
        out
    }
}

#[allow(dead_code)]
pub(crate) fn nullable<'a>(value: &'a str, sentinels: &[&str]) -> Option<&'a str> {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || sentinels
            .iter()
            .any(|sentinel| trimmed.eq_ignore_ascii_case(sentinel))
    {
        None
    } else {
        Some(trimmed)
    }
}

pub(crate) fn percent(value: &str) -> Option<f64> {
    value
        .trim()
        .trim_end_matches('%')
        .replace(',', "")
        .parse::<f64>()
        .ok()
}

#[allow(dead_code)]
pub(crate) fn duration_seconds(value: &str) -> Option<f64> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    if let Ok(seconds) = value.parse::<f64>() {
        return Some(seconds);
    }
    let (number, factor) = value
        .strip_suffix("ms")
        .map(|v| (v, 0.001))
        .or_else(|| value.strip_suffix('s').map(|v| (v, 1.0)))
        .or_else(|| value.strip_suffix('m').map(|v| (v, 60.0)))
        .or_else(|| value.strip_suffix('h').map(|v| (v, 3600.0)))
        .or_else(|| value.strip_suffix('d').map(|v| (v, 86400.0)))?;
    number.trim().parse::<f64>().ok().map(|n| n * factor)
}

pub(crate) fn scalar(value: &str) -> Value {
    let trimmed = value.trim();
    if matches!(trimmed, "<none>" | "<no value>" | "N/A" | "n/a" | "-") {
        return Value::Null;
    }
    if let Ok(v) = trimmed.parse::<i64>() {
        return Value::from(v);
    }
    if let Ok(v) = trimmed.parse::<f64>() {
        return Value::from(v);
    }
    if trimmed.eq_ignore_ascii_case("true") {
        return Value::Bool(true);
    }
    if trimmed.eq_ignore_ascii_case("false") {
        return Value::Bool(false);
    }
    Value::String(trimmed.to_string())
}

pub(crate) fn parse_human_bytes(value: &str) -> Option<u64> {
    let compact = value.trim().replace(' ', "");
    if compact.is_empty() || compact == "--" || compact == "-" {
        return None;
    }
    let split = compact
        .find(|c: char| !(c.is_ascii_digit() || c == '.'))
        .unwrap_or(compact.len());
    let number = compact[..split].parse::<f64>().ok()?;
    let unit = compact[split..].to_ascii_lowercase();
    let factor = match unit.as_str() {
        "" | "b" => 1.0,
        "kb" | "kib" | "k" => 1024.0,
        "mb" | "mib" | "m" => 1024.0 * 1024.0,
        "gb" | "gib" | "g" => 1024.0 * 1024.0 * 1024.0,
        "tb" | "tib" | "t" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        _ => return None,
    };
    Some((number * factor) as u64)
}
