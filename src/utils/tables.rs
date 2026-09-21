use serde_json::{Map, Value};

use crate::ScocError;

pub fn simple_table_parse(lines: &[&str]) -> Result<Vec<Value>, ScocError> {
    let Some(header) = lines.first() else {
        return Err(ScocError::InvalidInput {
            parser: "table".into(),
            message: "table has no header".into(),
        });
    };
    let headers: Vec<&str> = header.split_whitespace().collect();
    if headers.is_empty() {
        return Err(ScocError::InvalidInput {
            parser: "table".into(),
            message: "table has an empty header".into(),
        });
    }
    let mut out = Vec::new();
    for line in lines.iter().skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        let mut rest = line.trim();
        let mut fields = Vec::with_capacity(headers.len());
        for index in 0..headers.len() {
            if index + 1 == headers.len() {
                fields.push(rest.trim());
                break;
            }
            let trimmed = rest.trim_start();
            let split = trimmed.find(char::is_whitespace).unwrap_or(trimmed.len());
            fields.push(&trimmed[..split]);
            rest = &trimmed[split..];
        }
        let mut object = Map::new();
        for (name, value) in headers.iter().zip(fields.iter()) {
            object.insert((*name).to_string(), Value::String((*value).to_string()));
        }
        out.push(Value::Object(object));
    }
    Ok(out)
}

/// Parse a table whose cells are aligned under header columns, including
/// tables with blank cells and values containing spaces.
///
/// This mirrors JC's `universal.sparse_table_parse`: boundaries start at the
/// next header and are moved left when a preceding value overflows its
/// nominal column. SCOC uses ranges instead of inserting a sentinel byte so
/// input data can never collide with the implementation delimiter.
pub fn sparse_table_parse(lines: &[&str]) -> Result<Vec<Value>, ScocError> {
    let Some(header) = lines.first() else {
        return Err(ScocError::InvalidInput {
            parser: "table".into(),
            message: "table has no header".into(),
        });
    };

    let headers: Vec<&str> = header.split_whitespace().collect();
    if headers.is_empty() {
        return Err(ScocError::InvalidInput {
            parser: "table".into(),
            message: "table has an empty header".into(),
        });
    }

    // JC locates every non-final boundary at the occurrence of the next
    // header surrounded by spaces. `find` returns byte offsets, which is
    // safe for slicing because the matching header token itself begins at a
    // UTF-8 boundary.
    let mut boundary_specs = Vec::with_capacity(headers.len().saturating_sub(1));
    for next_header in headers.iter().skip(1) {
        let needle = format!(" {next_header} ");
        let boundary =
            format!("{header} ")
                .find(&needle)
                .ok_or_else(|| ScocError::InvalidInput {
                    parser: "table".into(),
                    message: format!("could not locate sparse-table column `{next_header}`"),
                })?;
        boundary_specs.push(boundary);
    }

    // JC works on Python strings, so boundaries and widths are character
    // positions; `lsblk` tree glyphs such as `├─` are multi-byte.
    let boundary_specs: Vec<usize> = boundary_specs
        .into_iter()
        .map(|byte| format!("{header} ")[..byte].chars().count())
        .collect();
    let max_len = lines
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or_else(|| header.chars().count());
    let mut output = Vec::new();

    for line in lines.iter().skip(1) {
        if line.trim().is_empty() {
            continue;
        }

        let mut chars: Vec<char> = line.chars().collect();
        chars.resize(max_len, ' ');

        let mut boundaries = Vec::with_capacity(boundary_specs.len());
        for nominal in &boundary_specs {
            let mut boundary = (*nominal).min(chars.len().saturating_sub(1));
            // If this nominal boundary cuts through a value, JC moves left
            // until it finds whitespace.
            while boundary > 0 && !chars[boundary].is_whitespace() {
                boundary -= 1;
            }
            boundaries.push(boundary);
        }

        let mut object = Map::new();
        let mut start = 0usize;
        for (index, name) in headers.iter().enumerate() {
            let end = boundaries.get(index).copied().unwrap_or(chars.len());
            let end = end.max(start).min(chars.len());
            let cell: String = chars[start..end].iter().collect();
            let value = cell.trim();
            object.insert(
                (*name).to_string(),
                if value.is_empty() {
                    Value::Null
                } else {
                    Value::String(value.to_string())
                },
            );
            // JC replaces exactly one boundary-space char with its sentinel,
            // then splits on that sentinel. Skipping one char reproduces that
            // behavior while preserving any additional alignment spaces for
            // trimming in the next cell.
            start = if index < boundaries.len() {
                end.saturating_add(1).min(chars.len())
            } else {
                end
            };
        }
        output.push(Value::Object(object));
    }

    Ok(output)
}
