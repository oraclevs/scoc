use serde_json::{Map, Value};

use crate::parsers::common::{normalize_key, raw, StaticParser};
use crate::utils::{input_to_str, to_i64_value};
use crate::{
    OptionSpec, OutputShape, ParseOptions, ParserCapabilities, ParserDescriptor, ParserOutput,
    ParserTag, Platform, ScocError, UpstreamParser,
};

const OPTIONS: [OptionSpec; 1] = [OptionSpec::bool("raw", false, "Return JC raw output")];
const PLATFORMS: [Platform; 1] = [Platform::Linux];
const TAGS: [ParserTag; 1] = [ParserTag::Command];

pub static DESCRIPTOR: ParserDescriptor = ParserDescriptor {
    name: "apt-cache-show",
    aliases: &[],
    description: "`apt-cache show` command parser",
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
        standard_name: "apt-cache-show",
        standard_version: "1.0",
        streaming_name: None,
        streaming_version: None,
    }),
};

fn finish_entry(entry: &mut Map<String, Value>, description: &mut Vec<String>, raw_mode: bool) {
    if !description.is_empty() {
        entry.insert("description".into(), Value::String(description.join(" ")));
        description.clear();
    }
    if raw_mode {
        return;
    }
    for key in ["epoch", "size", "installed_size"] {
        if let Some(Value::String(value)) = entry.get(key).cloned() {
            entry.insert(key.into(), to_i64_value(&value));
        }
    }
    for key in [
        "depends",
        "pre_depends",
        "recommends",
        "suggests",
        "conflicts",
        "breaks",
        "tag",
        "replaces",
    ] {
        if let Some(Value::String(value)) = entry.get(key).cloned() {
            let values = value
                .split(',')
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(|v| Value::String(v.to_string()))
                .collect();
            entry.insert(key.into(), Value::Array(values));
        }
    }
}

fn parse(
    descriptor: &'static ParserDescriptor,
    input: &[u8],
    options: &ParseOptions,
) -> Result<Value, ScocError> {
    let text = input_to_str(descriptor.name, input)?;
    let raw_mode = raw(options);
    let mut rows = Vec::new();
    let mut entry = Map::new();
    let mut description = Vec::new();
    let mut desc_mode = false;

    for line in text.lines().filter(|line| !line.is_empty()) {
        let split = line.split_once(": ");
        let starts_record = split
            .map(|(key, _)| key.starts_with("Name") || key == "Package")
            .unwrap_or(false);
        if starts_record && !entry.is_empty() {
            finish_entry(&mut entry, &mut description, raw_mode);
            rows.push(Value::Object(std::mem::take(&mut entry)));
            desc_mode = false;
        }
        if line.starts_with("Description :") {
            desc_mode = true;
            description.clear();
            continue;
        }
        if line.starts_with("Description-en:") {
            desc_mode = true;
            if let Some((_, value)) = split {
                description.push(value.trim().to_string());
            }
            continue;
        }
        if desc_mode && line.starts_with(' ') {
            description.push(line.to_string());
            continue;
        }
        if let Some((key, value)) = split {
            desc_mode = false;
            entry.insert(normalize_key(key), Value::String(value.trim().to_string()));
        } else if desc_mode {
            description.push(line.to_string());
        }
    }
    if !entry.is_empty() {
        finish_entry(&mut entry, &mut description, raw_mode);
        rows.push(Value::Object(entry));
    }
    Ok(Value::Array(rows))
}

pub static APT_CACHE_SHOW: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
