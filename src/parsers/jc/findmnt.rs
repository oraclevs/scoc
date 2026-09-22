use crate::parsers::common::{raw, StaticParser};
use crate::utils::{input_to_str, simple_table_parse};
use crate::{
    OptionSpec, OutputShape, ParseOptions, ParserCapabilities, ParserDescriptor, ParserOutput,
    ParserTag, Platform, ScocError, UpstreamParser,
};
use regex::Regex;
use serde_json::{Map, Value};
const OPTIONS: [OptionSpec; 1] = [OptionSpec::bool("raw", false, "Return JC raw output")];
const PLATFORMS: [Platform; 1] = [Platform::Linux];
const TAGS: [ParserTag; 1] = [ParserTag::Command];
pub static DESCRIPTOR: ParserDescriptor = ParserDescriptor {
    name: "findmnt",
    aliases: &[],
    description: "`findmnt` command parser",
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
        standard_name: "findmnt",
        standard_version: "1.1",
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
        return Ok(Value::Array(vec![]));
    }
    let re = Regex::new(r"^([│ ├─└─|`-]+)/")
        .map_err(|e| ScocError::parse(descriptor.name, None, e.to_string()))?;
    let mut lines: Vec<String> = text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            re.replace(l, |caps: &regex::Captures| {
                format!("{}/", " ".repeat(caps[1].len()))
            })
            .to_string()
        })
        .collect();
    if let Some(h) = lines.first_mut() {
        *h = h.to_ascii_lowercase();
    }
    let refs: Vec<&str> = lines.iter().map(String::as_str).collect();
    let mut rows = simple_table_parse(&refs)?;
    if !raw(options) {
        for row in &mut rows {
            let Some(o) = row.as_object_mut() else {
                continue;
            };
            if let Some(Value::String(v)) = o.get("options").cloned() {
                let mut regular = vec![];
                let mut kv = Map::new();
                for opt in v.split(',') {
                    if let Some((k, v)) = opt.split_once('=') {
                        kv.insert(k.into(), Value::String(v.into()));
                    } else {
                        regular.push(Value::String(opt.into()));
                    }
                }
                if !regular.is_empty() {
                    o.insert("options".into(), Value::Array(regular));
                }
                if !kv.is_empty() {
                    o.insert("kv_options".into(), Value::Object(kv));
                }
            }
        }
    }
    Ok(Value::Array(rows))
}
pub static FINDMNT: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
