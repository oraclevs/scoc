use serde_json::{Map, Value};

use crate::parsers::common::{raw, StaticParser};
use crate::utils::{input_to_str, simple_table_parse, to_i64_value};
use crate::{
    OptionSpec, OutputShape, ParseOptions, ParserCapabilities, ParserDescriptor, ParserOutput,
    ParserTag, Platform, ScocError, UpstreamParser,
};

const OPTIONS: [OptionSpec; 1] = [OptionSpec::bool("raw", false, "Return JC raw output")];
const PLATFORMS: [Platform; 2] = [Platform::Linux, Platform::MacOs];
const TAGS: [ParserTag; 1] = [ParserTag::Command];
pub static DESCRIPTOR: ParserDescriptor = ParserDescriptor {
    name: "arp",
    aliases: &[],
    description: "`arp` command parser",
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
        standard_name: "arp",
        standard_version: "1.12",
        streaming_name: None,
        streaming_version: None,
    }),
};

fn process(mut rows: Vec<Value>, raw_mode: bool) -> Vec<Value> {
    if raw_mode {
        return rows;
    }
    for row in &mut rows {
        let Some(obj) = row.as_object_mut() else {
            continue;
        };
        if matches!(obj.get("name"), Some(Value::String(name)) if name == "?") {
            obj.insert("name".into(), Value::Null);
        }
        if let Some(Value::String(value)) = obj.get("expires").cloned() {
            obj.insert("expires".into(), to_i64_value(&value));
        }
    }
    rows
}

fn parse(
    descriptor: &'static ParserDescriptor,
    input: &[u8],
    options: &ParseOptions,
) -> Result<Value, ScocError> {
    let text = input_to_str(descriptor.name, input)?;
    let mut lines: Vec<String> = text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(str::to_string)
        .collect();
    if lines
        .last()
        .is_some_and(|line| line.starts_with("Entries:"))
    {
        lines.pop();
    }
    if lines.is_empty() {
        return Ok(Value::Array(Vec::new()));
    }
    let mut rows = Vec::new();
    if lines[0].starts_with("Address") {
        lines[0] = lines[0]
            .replace("Flags Mask", "flags_mask")
            .to_ascii_lowercase();
        let refs: Vec<&str> = lines.iter().map(String::as_str).collect();
        rows = simple_table_parse(&refs)?;
    } else {
        for (index, line) in lines.iter().enumerate() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty()
                || parts[0].contains("bucket:")
                || (parts.len() > 1 && parts[0] == "There" && parts[1] == "are")
            {
                continue;
            }
            if parts.len() < 4 {
                return Err(ScocError::parse(
                    descriptor.name,
                    Some(index + 1),
                    "malformed arp row",
                ));
            }
            let mut obj = Map::new();
            obj.insert("name".into(), Value::String(parts[0].to_string()));
            obj.insert(
                "address".into(),
                Value::String(
                    parts
                        .get(1)
                        .unwrap_or(&"")
                        .trim_matches(|c| c == '(' || c == ')')
                        .to_string(),
                ),
            );
            let incomplete = parts.contains(&"<incomplete>") || parts.contains(&"(incomplete)");
            if incomplete {
                obj.insert("hwtype".into(), Value::Null);
                obj.insert("hwaddress".into(), Value::Null);
                if let Some(iface) = parts.get(5) {
                    obj.insert("iface".into(), Value::String((*iface).to_string()));
                }
            } else {
                obj.insert(
                    "hwaddress".into(),
                    Value::String(parts.get(3).unwrap_or(&"").to_string()),
                );
                let hwtype = if line.trim_end().ends_with(']') {
                    parts.last().copied().unwrap_or("")
                } else {
                    parts.get(4).copied().unwrap_or("")
                };
                obj.insert(
                    "hwtype".into(),
                    Value::String(hwtype.trim_matches(|c| c == '[' || c == ']').to_string()),
                );
                if let Some(pos) = parts.iter().position(|p| *p == "on") {
                    if let Some(iface) = parts.get(pos + 1) {
                        obj.insert("iface".into(), Value::String((*iface).to_string()));
                    }
                } else if let Some(iface) = parts.get(5) {
                    obj.insert("iface".into(), Value::String((*iface).to_string()));
                }
                if line.contains("permanent") {
                    obj.insert("permanent".into(), Value::Bool(true));
                } else if line.trim_end().ends_with(']') {
                    obj.insert("permanent".into(), Value::Bool(false));
                }
                if let Some(pos) = parts.iter().position(|p| *p == "expires") {
                    if pos > 0 {
                        obj.insert("expires".into(), Value::String(parts[pos - 1].to_string()));
                    }
                }
            }
            rows.push(Value::Object(obj));
        }
    }
    Ok(Value::Array(process(rows, raw(options))))
}
pub static ARP: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
