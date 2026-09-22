use regex::Regex;
use serde_json::{Map, Value};
use std::sync::OnceLock;

use crate::parsers::common::StaticParser;
use crate::utils::input_to_str;
use crate::{
    OptionSpec, OutputShape, ParseOptions, ParserCapabilities, ParserDescriptor, ParserOutput,
    ParserTag, Platform, ScocError, UpstreamParser,
};

const OPTIONS: [OptionSpec; 1] = [OptionSpec::bool("raw", false, "Return JC raw output")];
const PLATFORMS: [Platform; 2] = [Platform::Linux, Platform::MacOs];
const TAGS: [ParserTag; 1] = [ParserTag::Command];

pub static DESCRIPTOR: ParserDescriptor = ParserDescriptor {
    name: "mount",
    aliases: &[],
    description: "`mount` command parser",
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
        standard_name: "mount",
        standard_version: "1.11",
        streaming_name: None,
        streaming_version: None,
    }),
};

fn linux_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^(?P<filesystem>.*) on (?P<mount_point>.*?) type (?P<type>\S+) \((?P<options>.*?)\)\s*$").expect("valid mount regex"))
}

fn mac_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"^(?P<filesystem>.*) on (?P<mount_point>.*?) \((?P<options>.*?)\)\s*$")
            .expect("valid mount regex")
    })
}

fn parse(
    descriptor: &'static ParserDescriptor,
    input: &[u8],
    _options: &ParseOptions,
) -> Result<Value, ScocError> {
    let text = input_to_str(descriptor.name, input)?;
    let mut rows = Vec::new();
    for (line_number, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let captures = if line.contains(" type ") {
            linux_re().captures(line)
        } else {
            mac_re().captures(line)
        };
        let Some(captures) = captures else {
            return Err(ScocError::parse(
                descriptor.name,
                Some(line_number + 1),
                "unrecognized mount output",
            ));
        };
        let mut row = Map::new();
        let filesystem = captures
            .name("filesystem")
            .map(|m| m.as_str())
            .unwrap_or("");
        let mount_point = captures
            .name("mount_point")
            .map(|m| m.as_str())
            .unwrap_or("");
        row.insert("filesystem".into(), Value::String(filesystem.to_string()));
        row.insert("mount_point".into(), Value::String(mount_point.to_string()));
        if let Some(kind) = captures.name("type") {
            row.insert("type".into(), Value::String(kind.as_str().to_string()));
        }
        let options = captures
            .name("options")
            .map(|m| m.as_str())
            .unwrap_or("")
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| Value::String(value.to_string()))
            .collect();
        row.insert("options".into(), Value::Array(options));
        rows.push(Value::Object(row));
    }
    Ok(Value::Array(rows))
}

pub static MOUNT: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
