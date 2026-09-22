use serde_json::Value;

use crate::parsers::common::{raw, StaticParser};
use crate::utils::{convert_size_to_int, input_to_str, simple_table_parse};
use crate::{
    OptionSpec, OutputShape, ParseOptions, ParserCapabilities, ParserDescriptor, ParserOutput,
    ParserTag, Platform, ScocError, UpstreamParser,
};

const OPTIONS: [OptionSpec; 1] = [OptionSpec::bool("raw", false, "Return JC raw output")];
const PLATFORMS: [Platform; 1] = [Platform::Linux];
const TAGS: [ParserTag; 1] = [ParserTag::Command];

pub static DESCRIPTOR: ParserDescriptor = ParserDescriptor {
    name: "free",
    aliases: &[],
    description: "`free` command parser",
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
        standard_name: "free",
        standard_version: "1.8",
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
    if text.trim().is_empty() {
        return Ok(Value::Array(Vec::new()));
    }
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    let Some(header) = lines.first_mut() else {
        return Ok(Value::Array(Vec::new()));
    };
    *header = format!(
        "type {}",
        header
            .to_ascii_lowercase()
            .replace("buff/cache", "buff_cache")
    );
    let refs: Vec<&str> = lines.iter().map(String::as_str).collect();
    let mut rows = simple_table_parse(&refs)?;
    for row in &mut rows {
        let Some(object) = row.as_object_mut() else {
            continue;
        };
        if let Some(Value::String(kind)) = object.get_mut("type") {
            *kind = kind.trim_end_matches(':').to_string();
        }
        if raw(options) {
            continue;
        }
        for field in [
            "total",
            "used",
            "free",
            "shared",
            "buff_cache",
            "buffers",
            "cache",
            "available",
        ] {
            let Some(Value::String(value)) = object.get(field).cloned() else {
                continue;
            };
            object.insert(
                field.into(),
                convert_size_to_int(&value, false, false).map_or(Value::Null, Value::from),
            );
        }
    }
    Ok(Value::Array(rows))
}

pub static FREE: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
