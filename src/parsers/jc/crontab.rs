use crate::parsers::common::{raw, StaticParser};
use crate::utils::input_to_str;
use crate::{
    OptionSpec, OutputShape, ParseOptions, ParserCapabilities, ParserDescriptor, ParserOutput,
    ParserTag, Platform, ScocError, UpstreamParser,
};
use serde_json::{Map, Value};
const OPTIONS: [OptionSpec; 1] = [OptionSpec::bool("raw", false, "Return JC raw output")];
const PLATFORMS: [Platform; 2] = [Platform::Linux, Platform::MacOs];
const TAGS: [ParserTag; 2] = [ParserTag::File, ParserTag::Command];
pub static DESCRIPTOR: ParserDescriptor = ParserDescriptor {
    name: "crontab",
    aliases: &[],
    description: "`crontab` command and file parser",
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
        standard_name: "crontab",
        standard_version: "1.9",
        streaming_name: None,
        streaming_version: None,
    }),
};
fn arr(v: &str) -> Value {
    Value::Array(v.split(',').map(|s| Value::String(s.to_string())).collect())
}
fn parse(
    descriptor: &'static ParserDescriptor,
    input: &[u8],
    options: &ParseOptions,
) -> Result<Value, ScocError> {
    let text = input_to_str(descriptor.name, input)?;
    let raw_mode = raw(options);
    if text.trim().is_empty() {
        return Ok(Value::Object(Map::new()));
    }
    let mut vars = vec![];
    let mut schedule = vec![];
    for (idx, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('@') {
            let mut p = line.splitn(2, char::is_whitespace);
            let occ = p.next().unwrap_or("").trim_start_matches('@');
            let cmd = p.next().unwrap_or("").trim();
            let mut o = Map::new();
            o.insert("occurrence".into(), Value::String(occ.into()));
            o.insert("command".into(), Value::String(cmd.into()));
            schedule.push(Value::Object(o));
            continue;
        }
        if line.contains('=') && !line.starts_with(|c: char| c.is_ascii_digit() || c == '*') {
            let Some((k, v)) = line.split_once('=') else {
                continue;
            };
            let mut o = Map::new();
            o.insert("name".into(), Value::String(k.trim().into()));
            o.insert("value".into(), Value::String(v.trim().into()));
            vars.push(Value::Object(o));
            continue;
        }
        let parts: Vec<&str> = line
            .splitn(6, char::is_whitespace)
            .filter(|v| !v.is_empty())
            .collect();
        if parts.len() < 6 {
            return Err(ScocError::parse(
                descriptor.name,
                Some(idx + 1),
                "malformed crontab schedule",
            ));
        }
        let mut o = Map::new();
        for (key, val) in ["minute", "hour", "day_of_month", "month", "day_of_week"]
            .into_iter()
            .zip(&parts[..5])
        {
            o.insert(
                key.into(),
                if raw_mode {
                    Value::String((*val).into())
                } else {
                    arr(val)
                },
            );
        }
        o.insert("command".into(), Value::String(parts[5].into()));
        schedule.push(Value::Object(o));
    }
    let mut root = Map::new();
    root.insert("variables".into(), Value::Array(vars));
    root.insert("schedule".into(), Value::Array(schedule));
    Ok(Value::Object(root))
}
pub static CRONTAB: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
