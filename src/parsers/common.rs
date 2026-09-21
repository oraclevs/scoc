use serde_json::{Map, Value};

use crate::utils::{input_to_str, to_i64_value};
use crate::{ParseOptions, ParserDescriptor, ScocError, ScocParser, ScocStreamParser};

pub type ParseFn = fn(&'static ParserDescriptor, &[u8], &ParseOptions) -> Result<Value, ScocError>;

pub struct StaticParser {
    descriptor: &'static ParserDescriptor,
    parse_fn: ParseFn,
}

impl StaticParser {
    pub const fn new(descriptor: &'static ParserDescriptor, parse_fn: ParseFn) -> Self {
        Self {
            descriptor,
            parse_fn,
        }
    }
}

impl ScocParser for StaticParser {
    fn descriptor(&self) -> &'static ParserDescriptor {
        self.descriptor
    }

    fn parse(&self, input: &[u8], options: &ParseOptions) -> Result<Value, ScocError> {
        (self.parse_fn)(self.descriptor, input, options)
    }

    fn stream_parser(
        &self,
        _options: &ParseOptions,
    ) -> Result<Box<dyn ScocStreamParser>, ScocError> {
        Err(ScocError::StreamingUnsupported {
            parser: self.descriptor.name.into(),
        })
    }
}

pub fn raw(options: &ParseOptions) -> bool {
    options.bool("raw").unwrap_or(false)
}

pub fn parse_first_rest(
    descriptor: &'static ParserDescriptor,
    input: &[u8],
    options: &ParseOptions,
    first_name: &str,
    rest_name: &str,
    first_numeric: bool,
) -> Result<Value, ScocError> {
    let text = input_to_str(descriptor.name, input)?;
    let raw_mode = raw(options);
    let mut rows = Vec::new();

    for (index, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let trimmed = line.trim();
        let split = trimmed.find(char::is_whitespace).ok_or_else(|| {
            ScocError::parse(
                descriptor.name,
                Some(index + 1),
                "expected at least two whitespace-separated fields",
            )
        })?;
        let first = &trimmed[..split];
        let rest = trimmed[split..].trim_start();
        let mut row = Map::new();
        row.insert(
            first_name.into(),
            if first_numeric && !raw_mode {
                to_i64_value(first)
            } else {
                Value::String(first.into())
            },
        );
        row.insert(rest_name.into(), Value::String(rest.into()));
        rows.push(Value::Object(row));
    }

    Ok(Value::Array(rows))
}

/// Python's `str.split(maxsplit=n)`: runs of whitespace separate fields, and the
/// remainder after `n` splits keeps its inner spacing.
pub fn split_whitespace_max(input: &str, max_splits: usize) -> Vec<&str> {
    let mut out = Vec::new();
    let mut rest = input.trim_start();
    while !rest.is_empty() {
        if out.len() == max_splits {
            out.push(rest.trim_end());
            break;
        }
        let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
        out.push(&rest[..end]);
        rest = rest[end..].trim_start();
    }
    out
}

pub fn normalize_key(input: &str) -> String {
    const SPECIAL: &str = "!\"#$%&'()*+,-./:;<=>?@[\\]^`{|}~ ";
    let mut value = input.trim().to_ascii_lowercase();
    let initial_underscore = value.starts_with('_');
    for ch in SPECIAL.chars() {
        value = value.replace(ch, "_");
    }
    value = value.trim_matches('_').replace('_', " ");
    let mut value = value.split_whitespace().collect::<Vec<_>>().join("_");
    if initial_underscore {
        value.insert(0, '_');
    }
    value
}

pub fn shell_words(parser: &str, line: &str) -> Result<Vec<String>, ScocError> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    let mut escaped = false;
    for ch in line.chars() {
        if escaped {
            current.push(ch);
            escaped = false;
            continue;
        }
        if ch == '\\' && quote != Some('\'') {
            escaped = true;
            continue;
        }
        if let Some(active) = quote {
            if ch == active {
                quote = None;
            } else {
                current.push(ch);
            }
            continue;
        }
        if ch == '\'' || ch == '"' {
            quote = Some(ch);
        } else if ch.is_whitespace() {
            if !current.is_empty() {
                out.push(std::mem::take(&mut current));
            }
        } else {
            current.push(ch);
        }
    }
    if escaped {
        current.push('\\');
    }
    if quote.is_some() {
        return Err(ScocError::parse(parser, None, "unterminated quote"));
    }
    if !current.is_empty() {
        out.push(current);
    }
    Ok(out)
}
