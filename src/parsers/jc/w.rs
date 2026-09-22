use serde_json::{Map, Value};

use crate::parsers::common::{raw, StaticParser};
use crate::utils::input_to_str;
use crate::{
    OptionSpec, OutputShape, ParseOptions, ParserCapabilities, ParserDescriptor, ParserOutput,
    ParserTag, Platform, ScocError, UpstreamParser,
};

const OPTIONS: [OptionSpec; 1] = [OptionSpec::bool("raw", false, "Return JC raw output")];
const PLATFORMS: [Platform; 2] = [Platform::Linux, Platform::MacOs];
const TAGS: [ParserTag; 1] = [ParserTag::Command];

pub static DESCRIPTOR: ParserDescriptor = ParserDescriptor {
    name: "w",
    aliases: &[],
    description: "`w` command parser",
    parser_version: "0.1.0",
    platforms: &PLATFORMS,
    tags: &TAGS,
    output: ParserOutput {
        normalized: OutputShape::Table,
        raw: Some(OutputShape::Table),
        stream_item: None,
    },
    capabilities: ParserCapabilities {
        raw: true,
        streaming: false,
        ignore_errors: false,
    },
    options: &OPTIONS,
    upstream: Some(UpstreamParser {
        standard_name: "w",
        standard_version: "1.6",
        streaming_name: None,
        streaming_version: None,
    }),
};

fn parse(
    descriptor: &'static ParserDescriptor,
    input: &[u8],
    options: &ParseOptions,
) -> Result<Value, ScocError> {
    let text = input_to_str(descriptor.name, input)?;
    let mut lines = text.lines();
    let _summary = lines.next();
    let Some(header) = lines.next() else {
        return Ok(Value::Array(Vec::new()));
    };
    let header_text = header.to_ascii_lowercase().replace("login@", "login_at");
    let headers: Vec<&str> = header_text.split_whitespace().collect();
    let from_column = header_text.find("from");
    let mut rows = Vec::new();
    for (line_number, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let mut fields = Vec::with_capacity(headers.len());
        let mut rest = line.trim();
        for index in 0..headers.len() {
            let trimmed = rest.trim_start();
            if trimmed.is_empty() {
                break;
            }
            if index + 1 == headers.len() {
                fields.push(trimmed.to_string());
                break;
            }
            let split = trimmed.find(char::is_whitespace).unwrap_or(trimmed.len());
            fields.push(trimmed[..split].to_string());
            rest = &trimmed[split..];
        }
        let missing_from = from_column
            .filter(|_| headers.contains(&"from"))
            .is_some_and(|column| {
                line.as_bytes()
                    .get(column)
                    .is_some_and(|byte| byte.is_ascii_whitespace())
                    && fields.len() + 1 == headers.len()
            });
        if missing_from {
            fields.insert(2, "-".into());
        }
        if fields.len() < headers.len() {
            return Err(ScocError::parse(
                descriptor.name,
                Some(line_number + 3),
                "not enough columns in w output",
            ));
        }
        let mut row = Map::new();
        for (header, value) in headers.iter().zip(fields.iter()) {
            let value = if !raw(options)
                && matches!(
                    *header,
                    "user" | "tty" | "from" | "login_at" | "idle" | "what"
                )
                && value == "-"
            {
                Value::Null
            } else {
                Value::String(value.clone())
            };
            row.insert((*header).into(), value);
        }
        rows.push(Value::Object(row));
    }
    Ok(Value::Array(rows))
}

pub static W: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
