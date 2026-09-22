use serde_json::Value;

use crate::utils::{input_to_str, simple_table_parse, to_f64_value, to_i64_value};
use crate::{
    OptionSpec, OutputShape, ParseOptions, ParserCapabilities, ParserDescriptor, ParserOutput,
    ParserTag, Platform, ScocError, ScocParser, UpstreamParser,
};

const OPTIONS: [OptionSpec; 1] = [OptionSpec::bool("raw", false, "Return JC raw output")];
const PLATFORMS: [Platform; 2] = [Platform::Linux, Platform::MacOs];
const TAGS: [ParserTag; 1] = [ParserTag::Command];

pub struct PsParser;
pub static PS: PsParser = PsParser;

pub static DESCRIPTOR: ParserDescriptor = ParserDescriptor {
    name: "ps",
    aliases: &[],
    description: "`ps` command parser",
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
        standard_name: "ps",
        standard_version: "1.7",
        streaming_name: None,
        streaming_version: None,
    }),
};

impl ScocParser for PsParser {
    fn descriptor(&self) -> &'static ParserDescriptor {
        &DESCRIPTOR
    }

    fn parse(&self, input: &[u8], options: &ParseOptions) -> Result<Value, ScocError> {
        let text = input_to_str("ps", input)?;
        if text.trim().is_empty() {
            return Ok(Value::Array(Vec::new()));
        }
        let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
        lines[0] = lines[0].to_ascii_lowercase();
        let refs: Vec<&str> = lines.iter().map(String::as_str).collect();
        let mut rows = simple_table_parse(&refs)?;
        if options.bool("raw").unwrap_or(false) {
            return Ok(Value::Array(rows));
        }

        for row in &mut rows {
            let Some(object) = row.as_object_mut() else {
                continue;
            };
            if let Some(value) = object.remove("%cpu") {
                object.insert("cpu_percent".into(), value);
            }
            if let Some(value) = object.remove("%mem") {
                object.insert("mem_percent".into(), value);
            }
            for key in ["pid", "ppid", "c", "vsz", "rss"] {
                if let Some(Value::String(value)) = object.get(key) {
                    let converted = to_i64_value(value);
                    object.insert(key.into(), converted);
                }
            }
            for key in ["cpu_percent", "mem_percent"] {
                if let Some(Value::String(value)) = object.get(key) {
                    let converted = to_f64_value(value);
                    object.insert(key.into(), converted);
                }
            }
            if matches!(object.get("tty"), Some(Value::String(value)) if value == "?" || value == "??")
            {
                object.insert("tty".into(), Value::Null);
            }
            if matches!(object.get("tt"), Some(Value::String(value)) if value == "??") {
                object.insert("tt".into(), Value::Null);
            }
        }
        Ok(Value::Array(rows))
    }
}
