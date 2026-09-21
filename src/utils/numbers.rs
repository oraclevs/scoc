use serde_json::Value;

fn numeric_text(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_digit() || *ch == '-' || *ch == '.')
        .collect()
}

pub fn to_i64(value: &str) -> Option<i64> {
    let cleaned = numeric_text(value);
    cleaned
        .parse::<i64>()
        .ok()
        .or_else(|| cleaned.parse::<f64>().ok().map(|value| value as i64))
}

pub fn to_f64(value: &str) -> Option<f64> {
    numeric_text(value).parse::<f64>().ok()
}

pub fn to_i64_value(value: &str) -> Value {
    to_i64(value).map_or(Value::Null, Value::from)
}

pub fn to_f64_value(value: &str) -> Value {
    to_f64(value)
        .and_then(serde_json::Number::from_f64)
        .map_or(Value::Null, Value::Number)
}
