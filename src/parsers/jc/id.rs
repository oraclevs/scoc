use serde_json::{Map, Value};

use crate::parsers::common::{raw, StaticParser};
use crate::utils::{input_to_str, to_i64_value};
use crate::{
    OptionSpec, OutputShape, ParseOptions, ParserCapabilities, ParserDescriptor, ParserOutput,
    ParserTag, Platform, ScocError, UpstreamParser,
};

const OPTIONS: [OptionSpec; 1] = [OptionSpec::bool("raw", false, "Return JC raw output")];
const PLATFORMS: [Platform; 2] = [Platform::Linux, Platform::MacOs];
const TAGS: [ParserTag; 1] = [ParserTag::Command];

pub static DESCRIPTOR: ParserDescriptor = ParserDescriptor {
    name: "id",
    aliases: &[],
    description: "`id` command parser",
    parser_version: "0.1.0",
    platforms: &PLATFORMS,
    tags: &TAGS,
    output: ParserOutput {
        normalized: OutputShape::Record,
        raw: Some(OutputShape::Record),
        stream_item: None,
    },
    capabilities: ParserCapabilities {
        raw: true,
        streaming: false,
        ignore_errors: false,
    },
    options: &OPTIONS,
    upstream: Some(UpstreamParser {
        standard_name: "id",
        standard_version: "1.7",
        streaming_name: None,
        streaming_version: None,
    }),
};

fn id_name(value: &str, raw_mode: bool) -> Value {
    let mut row = Map::new();
    let (id, name) = if let Some(open) = value.find('(') {
        let id = &value[..open];
        let name = value[open + 1..]
            .strip_suffix(')')
            .unwrap_or(&value[open + 1..]);
        (id, Some(name))
    } else {
        (value, None)
    };
    row.insert(
        "id".into(),
        if raw_mode {
            Value::String(id.into())
        } else {
            to_i64_value(id)
        },
    );
    row.insert(
        "name".into(),
        name.map_or(Value::Null, |name| Value::String(name.into())),
    );
    Value::Object(row)
}

fn parse(
    descriptor: &'static ParserDescriptor,
    input: &[u8],
    options: &ParseOptions,
) -> Result<Value, ScocError> {
    let text = input_to_str(descriptor.name, input)?.trim();
    if text.is_empty() {
        return Ok(Value::Object(Map::new()));
    }
    let raw_mode = raw(options);
    let mut object = Map::new();
    let markers = ["uid=", "gid=", "groups=", "context="];
    let mut positions = Vec::new();
    for marker in markers {
        if let Some(pos) = text.find(marker) {
            positions.push((pos, marker));
        }
    }
    positions.sort_by_key(|(pos, _)| *pos);
    for index in 0..positions.len() {
        let (start, marker) = positions[index];
        let value_start = start + marker.len();
        let end = positions
            .get(index + 1)
            .map(|(pos, _)| *pos)
            .unwrap_or(text.len());
        let value = text[value_start..end].trim();
        match marker {
            "uid=" => {
                object.insert("uid".into(), id_name(value, raw_mode));
            }
            "gid=" => {
                object.insert("gid".into(), id_name(value, raw_mode));
            }
            "groups=" => {
                let groups = value
                    .split(',')
                    .filter(|v| !v.is_empty())
                    .map(|v| id_name(v.trim(), raw_mode))
                    .collect();
                object.insert("groups".into(), Value::Array(groups));
            }
            "context=" => {
                let parts: Vec<&str> = value.splitn(4, ':').collect();
                if parts.len() == 4 {
                    let mut context = Map::new();
                    for (key, part) in ["user", "role", "type", "level"].into_iter().zip(parts) {
                        context.insert(key.into(), Value::String(part.into()));
                    }
                    object.insert("context".into(), Value::Object(context));
                }
            }
            _ => {}
        }
    }
    Ok(Value::Object(object))
}

pub static ID: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
