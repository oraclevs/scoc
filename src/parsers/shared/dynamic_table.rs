use regex::Regex;
use serde_json::{Map, Value};
use std::sync::OnceLock;

use super::normalize::{normalized_key, scalar};
use crate::ScocError;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub(crate) enum HeaderPolicy {
    Required,
    FirstNonEmpty,
}

fn gap_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?:\t+| {2,})").expect("valid dynamic-table regex"))
}

fn char_index(header: &str, byte_index: usize) -> usize {
    header[..byte_index].chars().count()
}

fn split_header(header: &str) -> Vec<(String, usize)> {
    let mut out = Vec::new();
    let mut start = 0usize;
    for m in gap_re().find_iter(header) {
        let cell = header[start..m.start()].trim();
        if !cell.is_empty() {
            out.push((cell.to_string(), char_index(header, start)));
        }
        start = m.end();
    }
    let cell = header[start..].trim();
    if !cell.is_empty() {
        out.push((cell.to_string(), char_index(header, start)));
    }
    if out.len() <= 1 {
        out.clear();
        let mut search = 0usize;
        for token in header.split_whitespace() {
            if let Some(rel) = header[search..].find(token) {
                let pos = search + rel;
                out.push((token.to_string(), char_index(header, pos)));
                search = pos + token.len();
            }
        }
    }
    out
}

pub(crate) fn dynamic_table(
    lines: &[&str],
    _header: HeaderPolicy,
) -> Result<Vec<Map<String, Value>>, ScocError> {
    let nonempty: Vec<&str> = lines
        .iter()
        .copied()
        .filter(|line| !line.trim().is_empty())
        .collect();
    let Some(header) = nonempty.first().copied() else {
        return Ok(Vec::new());
    };
    let columns = split_header(header);
    if columns.is_empty() {
        return Err(ScocError::InvalidInput {
            parser: "table".into(),
            message: "table has no columns".into(),
        });
    }
    let mut keys = Vec::with_capacity(columns.len());
    for (title, _) in &columns {
        let key = normalized_key(title);
        if keys.contains(&key) {
            return Err(ScocError::unsupported_variant(
                "table",
                format!("header collision for `{title}`"),
            ));
        }
        keys.push(key);
    }
    let starts: Vec<usize> = columns.iter().map(|(_, start)| *start).collect();
    let mut rows = Vec::new();
    for line in nonempty.iter().skip(1) {
        let mut row = Map::new();
        let chars: Vec<char> = line.chars().collect();
        for (idx, key) in keys.iter().enumerate() {
            let start = *starts.get(idx).unwrap_or(&0);
            let end = starts.get(idx + 1).copied().unwrap_or(chars.len());
            let cell: String = if start >= chars.len() {
                String::new()
            } else {
                chars[start..end.min(chars.len())].iter().collect()
            };
            let trimmed = cell.trim();
            row.insert(
                key.clone(),
                if trimmed.is_empty() {
                    Value::Null
                } else {
                    scalar(trimmed)
                },
            );
        }
        rows.push(row);
    }
    Ok(rows)
}
