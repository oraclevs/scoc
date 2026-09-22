use crate::parsers::common::{raw, StaticParser};
use crate::utils::input_to_str;
use crate::{
    OptionSpec, OutputShape, ParseOptions, ParserCapabilities, ParserDescriptor, ParserOutput,
    ParserTag, Platform, ScocError, UpstreamParser,
};
use regex::Regex;
use serde_json::{Map, Value};
const OPTIONS: [OptionSpec; 1] = [OptionSpec::bool("raw", false, "Return JC raw output")];
const PLATFORMS: [Platform; 2] = [Platform::Linux, Platform::MacOs];
const TAGS: [ParserTag; 1] = [ParserTag::Command];
pub static DESCRIPTOR: ParserDescriptor = ParserDescriptor {
    name: "hashsum",
    aliases: &[],
    description: "hashsum command parser (`md5sum`, `shasum`, etc.)",
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
        standard_name: "hashsum",
        standard_version: "1.3",
        streaming_name: None,
        streaming_version: None,
    }),
};
fn parse(
    d: &'static ParserDescriptor,
    input: &[u8],
    opts: &ParseOptions,
) -> Result<Value, ScocError> {
    let text = input_to_str(d.name, input)?;
    let raw_mode = raw(opts);
    let re = Regex::new(r"^([0-9a-fA-F]+) (.)(.*)$")
        .map_err(|e| ScocError::parse(d.name, None, e.to_string()))?;
    let mut out = vec![];
    for (n, line) in text.lines().enumerate() {
        if line.is_empty() {
            continue;
        }
        let (hash, mode, name) = if line.starts_with("MD5 (") {
            let Some((left, right)) = line.split_once('=') else {
                return Err(ScocError::parse(d.name, Some(n + 1), "invalid md5 output"));
            };
            (
                right.trim().to_string(),
                None,
                left.strip_prefix("MD5 (")
                    .and_then(|v| v.strip_suffix(") "))
                    .unwrap_or(left)
                    .to_string(),
            )
        } else {
            let Some(c) = re.captures(line) else {
                return Err(ScocError::parse(
                    d.name,
                    Some(n + 1),
                    "invalid hashsum line",
                ));
            };
            (c[1].to_string(), c[2].chars().next(), c[3].to_string())
        };
        let mode_value = if raw_mode {
            mode.map(|c| Value::String(c.to_string()))
                .unwrap_or(Value::Null)
        } else {
            Value::String(
                match mode {
                    Some(' ') => "text",
                    Some('*') => "binary",
                    Some('U') => "universal",
                    Some('^') => "bits",
                    None => "binary",
                    Some(_) => "unknown",
                }
                .into(),
            )
        };
        let mut o = Map::new();
        o.insert("filename".into(), Value::String(name));
        o.insert("mode".into(), mode_value);
        o.insert("hash".into(), Value::String(hash));
        out.push(Value::Object(o));
    }
    Ok(Value::Array(out))
}
pub static HASHSUM: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
