use crate::parsers::common::{raw, StaticParser};
use crate::utils::{input_to_str, simple_table_parse};
use crate::{
    OptionSpec, OutputShape, ParseOptions, ParserCapabilities, ParserDescriptor, ParserOutput,
    ParserTag, Platform, ScocError, UpstreamParser,
};
use serde_json::Value;
const OPTIONS: [OptionSpec; 1] = [OptionSpec::bool("raw", false, "Return JC raw output")];
const PLATFORMS: [Platform; 1] = [Platform::Linux];
const TAGS: [ParserTag; 1] = [ParserTag::Command];
pub static DESCRIPTOR: ParserDescriptor = ParserDescriptor {
    name: "dpkg-l",
    aliases: &[],
    description: "`dpkg -l` command parser",
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
        standard_name: "dpkg-l",
        standard_version: "1.3",
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
    let mut work = Vec::new();
    let mut found = false;
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        if line.contains("Architecture") {
            found = true;
            work.push(line.to_ascii_lowercase().replace("||/", "codes"));
            continue;
        }
        if line.contains("=========") {
            continue;
        }
        if found {
            work.push(line.to_string());
        }
    }
    if work.is_empty() {
        return Ok(Value::Array(vec![]));
    }
    let refs: Vec<&str> = work.iter().map(String::as_str).collect();
    let mut rows = simple_table_parse(&refs)?;
    if !raw(options) {
        for row in &mut rows {
            let Some(o) = row.as_object_mut() else {
                continue;
            };
            let Some(Value::String(codes)) = o.get("codes") else {
                continue;
            };
            let chars: Vec<char> = codes.to_ascii_lowercase().chars().collect();
            if let Some(c) = chars.first() {
                let v = match c {
                    'u' => "unknown",
                    'i' => "install",
                    'r' => "remove",
                    'p' => "purge",
                    'h' => "hold",
                    _ => "",
                };
                if !v.is_empty() {
                    o.insert("desired".into(), Value::String(v.into()));
                }
            }
            if let Some(c) = chars.get(1) {
                let v = match c {
                    'n' => "not installed",
                    'i' => "installed",
                    'c' => "config-files",
                    'u' => "unpacked",
                    'f' => "failed config",
                    'h' => "half installed",
                    'w' => "trigger await",
                    't' => "trigger pending",
                    _ => "",
                };
                if !v.is_empty() {
                    o.insert("status".into(), Value::String(v.into()));
                }
            }
            if chars.get(2) == Some(&'r') {
                o.insert("error".into(), Value::String("reinstall required".into()));
            }
        }
    }
    Ok(Value::Array(rows))
}
pub static DPKG_L: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
