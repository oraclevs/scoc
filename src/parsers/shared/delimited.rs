use super::normalize::scalar;
use crate::ScocError;
use serde_json::{Map, Value};

pub(crate) fn delimited_records(
    lines: &[&str],
    delimiter: char,
    fields: &[&str],
) -> Result<Vec<Map<String, Value>>, ScocError> {
    let mut out = Vec::new();
    for (idx, line) in lines.iter().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split(delimiter).collect();
        if parts.len() < fields.len() {
            return Err(ScocError::Parse {
                parser: "delimited".into(),
                line: Some(idx + 1),
                message: format!("expected {} fields, found {}", fields.len(), parts.len()),
            });
        }
        let mut row = Map::new();
        for (field, value) in fields.iter().zip(parts.iter()) {
            row.insert((*field).to_string(), scalar(value));
        }
        out.push(row);
    }
    Ok(out)
}
