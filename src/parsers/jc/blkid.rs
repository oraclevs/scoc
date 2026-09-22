use crate::parsers::common::{raw, shell_words, StaticParser};
use crate::utils::{input_to_str, to_i64_value};
use crate::{
    OptionSpec, OutputShape, ParseOptions, ParserCapabilities, ParserDescriptor, ParserOutput,
    ParserTag, Platform, ScocError, UpstreamParser,
};
use serde_json::{Map, Value};
const OPTIONS: [OptionSpec; 1] = [OptionSpec::bool("raw", false, "Return JC raw output")];
const PLATFORMS: [Platform; 1] = [Platform::Linux];
const TAGS: [ParserTag; 1] = [ParserTag::Command];
pub static DESCRIPTOR: ParserDescriptor = ParserDescriptor {
    name: "blkid",
    aliases: &[],
    description: "`blkid` command parser",
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
        standard_name: "blkid",
        standard_version: "1.6",
        streaming_name: None,
        streaming_version: None,
    }),
};
fn process(mut rows: Vec<Value>, raw_mode: bool) -> Vec<Value> {
    if raw_mode {
        return rows;
    }
    let ints = [
        "part_entry_number",
        "part_entry_offset",
        "part_entry_size",
        "id_part_entry_number",
        "id_part_entry_offset",
        "id_part_entry_size",
        "minimum_io_size",
        "physical_sector_size",
        "logical_sector_size",
        "id_iolimit_minimum_io_size",
        "id_iolimit_physical_sector_size",
        "id_iolimit_logical_sector_size",
    ];
    for row in &mut rows {
        let Some(obj) = row.as_object_mut() else {
            continue;
        };
        if let Some(v) = obj.remove("devname") {
            obj.insert("device".into(), v);
        }
        for key in ints {
            if let Some(Value::String(v)) = obj.get(key).cloned() {
                obj.insert(key.into(), to_i64_value(&v));
            }
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
    if text.trim().is_empty() {
        return Ok(Value::Array(vec![]));
    }
    let mut rows = Vec::new();
    let first = text.split_whitespace().next().unwrap_or("");
    if first.ends_with(':') {
        for (index, line) in text.lines().filter(|l| !l.trim().is_empty()).enumerate() {
            let mut entries = shell_words(descriptor.name, line)?;
            if entries.is_empty() {
                continue;
            }
            let device = entries.remove(0);
            let mut obj = Map::new();
            obj.insert(
                "device".into(),
                Value::String(device.trim_end_matches(':').into()),
            );
            for entry in entries {
                let Some((k, v)) = entry.split_once('=') else {
                    return Err(ScocError::parse(
                        descriptor.name,
                        Some(index + 1),
                        "malformed blkid field",
                    ));
                };
                obj.insert(k.to_ascii_lowercase(), Value::String(v.into()));
            }
            rows.push(Value::Object(obj));
        }
    } else {
        let mut obj = Map::new();
        for (index, line) in text.lines().enumerate() {
            if line.is_empty() {
                if !obj.is_empty() {
                    rows.push(Value::Object(std::mem::take(&mut obj)));
                }
                continue;
            }
            let Some((k, v)) = line.split_once('=') else {
                return Err(ScocError::parse(
                    descriptor.name,
                    Some(index + 1),
                    "malformed blkid key/value",
                ));
            };
            obj.insert(k.to_ascii_lowercase(), Value::String(v.into()));
        }
        if !obj.is_empty() {
            rows.push(Value::Object(obj));
        }
    }
    Ok(Value::Array(process(rows, raw(options))))
}
pub static BLKID: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
