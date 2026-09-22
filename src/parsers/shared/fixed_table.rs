use super::normalize::scalar;
use crate::ScocError;
use serde_json::{Map, Value};

#[derive(Clone, Copy, Debug)]
pub(crate) struct ColumnSpec {
    pub name: &'static str,
    pub start: usize,
}

pub(crate) fn fixed_table(
    lines: &[&str],
    columns: &[ColumnSpec],
) -> Result<Vec<Map<String, Value>>, ScocError> {
    if columns.is_empty() {
        return Err(ScocError::InvalidInput {
            parser: "table".into(),
            message: "no fixed columns".into(),
        });
    }
    let mut rows = Vec::new();
    for line in lines.iter().copied().filter(|line| !line.trim().is_empty()) {
        let mut row = Map::new();
        for (idx, col) in columns.iter().enumerate() {
            let end = columns
                .get(idx + 1)
                .map(|next| next.start)
                .unwrap_or(line.len());
            let cell = if col.start >= line.len() {
                ""
            } else {
                &line[col.start..end.min(line.len())]
            };
            row.insert(
                col.name.to_string(),
                if cell.trim().is_empty() {
                    Value::Null
                } else {
                    scalar(cell.trim())
                },
            );
        }
        rows.push(row);
    }
    Ok(rows)
}
