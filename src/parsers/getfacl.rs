use crate::parsers::common::StaticParser;
use crate::utils::input_to_str;
use crate::{
    OptionSpec, OutputShape, ParseOptions, ParserCapabilities, ParserDescriptor, ParserOutput,
    ParserTag, Platform, ScocError, UpstreamParser,
};
use serde_json::{Map, Value};
const OPTIONS: [OptionSpec; 1] = [OptionSpec::bool("raw", false, "Return JC raw output")];
const PLATFORMS: [Platform; 1] = [Platform::Linux];
const TAGS: [ParserTag; 1] = [ParserTag::Command];
pub static DESCRIPTOR: ParserDescriptor = ParserDescriptor {
    name: "getfacl",
    aliases: &[],
    description: "`getfacl` command parser",
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
        standard_name: "getfacl",
        standard_version: "1.0",
        streaming_name: None,
        streaming_version: None,
    }),
};
fn parse(d: &'static ParserDescriptor, input: &[u8], _: &ParseOptions) -> Result<Value, ScocError> {
    let text = input_to_str(d.name, input)?;
    let mut out = Vec::new();
    let mut file = None;
    let mut owner = None;
    let mut group = None;
    let mut flags = None;
    for line in text.lines() {
        let s = line.trim();
        if s.is_empty() {
            continue;
        }
        if let Some(rest) = s.strip_prefix('#') {
            if let Some((k, v)) = rest.split_once(':') {
                let val = if v.trim().is_empty() {
                    None
                } else {
                    Some(v.trim().to_string())
                };
                match k.trim() {
                    "file" => {
                        file = val;
                        owner = None;
                        group = None;
                        flags = None
                    }
                    "owner" => owner = val,
                    "group" => group = val,
                    "flags" => flags = val,
                    _ => {}
                }
            }
            continue;
        }
        let (entry, comment) = s.split_once('#').unwrap_or((s, ""));
        let mut parts = entry.trim_end().split(':').collect::<Vec<_>>();
        let default = parts.first().copied() == Some("default");
        if default {
            parts.remove(0);
        }
        if parts.len() != 3 {
            continue;
        }
        if !matches!(parts[0], "user" | "group" | "mask" | "other") {
            continue;
        }
        let effective = comment
            .trim()
            .strip_prefix("effective:")
            .map(str::to_string);
        let mut o = Map::new();
        o.insert(
            "file".into(),
            file.clone().map(Value::String).unwrap_or(Value::Null),
        );
        o.insert(
            "owner".into(),
            owner.clone().map(Value::String).unwrap_or(Value::Null),
        );
        o.insert(
            "group".into(),
            group.clone().map(Value::String).unwrap_or(Value::Null),
        );
        o.insert(
            "flags".into(),
            flags.clone().map(Value::String).unwrap_or(Value::Null),
        );
        o.insert("type".into(), Value::String(parts[0].into()));
        o.insert(
            "name".into(),
            if parts[1].is_empty() {
                Value::Null
            } else {
                Value::String(parts[1].into())
            },
        );
        o.insert("permissions".into(), Value::String(parts[2].into()));
        o.insert(
            "effective".into(),
            effective.map(Value::String).unwrap_or(Value::Null),
        );
        o.insert("default".into(), Value::Bool(default));
        out.push(Value::Object(o));
    }
    Ok(Value::Array(out))
}
pub static GETFACL: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
