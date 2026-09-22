use serde_json::{Map, Value};

use super::dynamic_table::{dynamic_table, HeaderPolicy};
use super::indented_tree::indented_tree;
use super::key_value::key_value_sections;
use super::native::parse_native;
use super::normalize::scalar;
use super::streaming::BufferedBatchStream;
use crate::utils::input_to_str;
use crate::{ParseOptions, ParserDescriptor, ScocError, ScocParser, ScocStreamParser};

#[derive(Clone, Copy, Debug)]
pub(crate) enum GenericKind {
    Auto,
    Table,
    KeyValue,
    Lines,
    Tree,
    Version,
}

pub(crate) struct GenericParser {
    descriptor: &'static ParserDescriptor,
    kind: GenericKind,
}

impl GenericParser {
    pub(crate) const fn new(descriptor: &'static ParserDescriptor, kind: GenericKind) -> Self {
        Self { descriptor, kind }
    }
}

fn table_value(lines: &[&str]) -> Result<Value, ScocError> {
    Ok(Value::Array(
        dynamic_table(lines, HeaderPolicy::FirstNonEmpty)?
            .into_iter()
            .map(Value::Object)
            .collect(),
    ))
}

fn key_value_value(lines: &[&str]) -> Value {
    let separator = if lines.iter().filter(|line| line.contains('=')).count()
        > lines.iter().filter(|line| line.contains(':')).count()
    {
        '='
    } else {
        ':'
    };
    let sections = key_value_sections(lines, separator);
    if sections.len() == 1 {
        Value::Object(sections.into_iter().next().unwrap_or_default())
    } else {
        Value::Array(sections.into_iter().map(Value::Object).collect())
    }
}

fn line_values(lines: &[&str]) -> Value {
    Value::Array(
        lines
            .iter()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                let mut row = Map::new();
                row.insert("value".into(), scalar(line.trim()));
                Value::Object(row)
            })
            .collect(),
    )
}

fn tree_value(lines: &[&str]) -> Value {
    Value::Array(
        indented_tree(lines)
            .into_iter()
            .map(|item| {
                let mut row = Map::new();
                row.insert("depth".into(), Value::from(item.depth as u64));
                row.insert("value".into(), Value::String(item.text));
                Value::Object(row)
            })
            .collect(),
    )
}

fn version_value(text: &str) -> Value {
    let mut row = Map::new();
    row.insert("raw".into(), Value::String(text.trim().to_string()));
    let tokens: Vec<&str> = text.split_whitespace().collect();
    if let Some(version) = tokens
        .iter()
        .find(|token| token.chars().next().is_some_and(|c| c.is_ascii_digit()))
    {
        row.insert(
            "version".into(),
            Value::String(
                version
                    .trim_matches(|c: char| c == ',' || c == '"')
                    .to_string(),
            ),
        );
    }
    Value::Object(row)
}

fn auto_value(lines: &[&str]) -> Result<Value, ScocError> {
    if lines.is_empty() {
        return Ok(Value::Array(Vec::new()));
    }
    let kv = lines
        .iter()
        .filter(|line| line.contains(':') || line.contains('='))
        .count();
    if kv * 2 >= lines.len().max(1) {
        return Ok(key_value_value(lines));
    }
    if lines.len() > 1 && lines[0].split_whitespace().count() > 1 {
        return table_value(lines);
    }
    Ok(line_values(lines))
}

impl ScocParser for GenericParser {
    fn descriptor(&self) -> &'static ParserDescriptor {
        self.descriptor
    }

    fn parse(&self, input: &[u8], _options: &ParseOptions) -> Result<Value, ScocError> {
        let text = input_to_str(self.descriptor.name, input)?;
        if self.descriptor.upstream.is_none() {
            if let Some(result) = parse_native(self.descriptor.name, text) {
                return result;
            }
        }
        let lines: Vec<&str> = text.lines().collect();
        match self.kind {
            GenericKind::Auto => auto_value(&lines),
            GenericKind::Table => table_value(&lines),
            GenericKind::KeyValue => Ok(key_value_value(&lines)),
            GenericKind::Lines => Ok(line_values(&lines)),
            GenericKind::Tree => Ok(tree_value(&lines)),
            GenericKind::Version => Ok(version_value(text)),
        }
    }

    fn stream_parser(
        &self,
        options: &ParseOptions,
    ) -> Result<Box<dyn ScocStreamParser>, ScocError> {
        if !self.descriptor.capabilities.streaming {
            return Err(ScocError::StreamingUnsupported {
                parser: self.descriptor.name.to_string(),
            });
        }
        Ok(Box::new(BufferedBatchStream::new(
            self.descriptor.name,
            options,
        )))
    }
}
