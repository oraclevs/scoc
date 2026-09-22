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
    name: "swapon",
    aliases: &[],
    description: "`swapon` command parser",
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
        standard_name: "swapon",
        standard_version: "1.0",
        streaming_name: None,
        streaming_version: None,
    }),
};
fn size(s: &str) -> Option<i64> {
    let (last, num) = s.split_at(s.len().saturating_sub(1));
    let (p, mul) = match num {
        "B" => (true, 1),
        "K" => (true, 1024),
        "M" => (true, 1024_i64.pow(2)),
        "G" => (true, 1024_i64.pow(3)),
        "T" => (true, 1024_i64.pow(4)),
        _ => (false, 1024),
    };
    let n = if p { last } else { s };
    n.parse::<i64>().ok().map(|v| v * mul)
}
fn parse(d: &'static ParserDescriptor, input: &[u8], _: &ParseOptions) -> Result<Value, ScocError> {
    let text = input_to_str(d.name, input)?;
    let mut lines = text.lines();
    let Some(header) = lines.next() else {
        return Ok(Value::Array(vec![]));
    };
    let cols = header.split_whitespace().collect::<Vec<_>>();
    let mut out = vec![];
    for (n, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let f = line.split_whitespace().collect::<Vec<_>>();
        if f.len() + 2 < cols.len() {
            return Err(ScocError::parse(
                d.name,
                Some(n + 2),
                "swapon column mismatch",
            ));
        }
        let mut o = Map::new();
        for (c, v) in cols.iter().zip(f.iter()) {
            let key = match *c {
                "NAME" | "Filename" => "name",
                "TYPE" | "Type" => "type",
                "SIZE" | "Size" => "size",
                "USED" | "Used" => "used",
                "PRIO" | "Priority" => "priority",
                "LABEL" => "label",
                "UUID" => "uuid",
                _ => continue,
            };
            let val = match key {
                "size" | "used" => size(v).map(Value::from).unwrap_or(Value::Null),
                "priority" => v.parse::<i64>().map(Value::from).unwrap_or(Value::Null),
                _ => Value::String((*v).into()),
            };
            o.insert(key.into(), val);
        }
        out.push(Value::Object(o));
    }
    Ok(Value::Array(out))
}
pub static SWAPON: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
