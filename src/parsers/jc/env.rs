use regex::Regex;
use serde_json::{Map, Value};
use std::sync::OnceLock;

use crate::utils::input_to_str;
use crate::{
    OptionSpec, OutputShape, ParseOptions, ParserCapabilities, ParserDescriptor, ParserOutput,
    ParserTag, Platform, ScocError, ScocParser, UpstreamParser,
};

const OPTIONS: [OptionSpec; 1] = [OptionSpec::bool("raw", false, "Return JC raw output")];
const PLATFORMS: [Platform; 2] = [Platform::Linux, Platform::MacOs];
const TAGS: [ParserTag; 1] = [ParserTag::Command];
const ALIASES: [&str; 1] = ["printenv"];

pub struct EnvParser;
pub static ENV: EnvParser = EnvParser;

pub static DESCRIPTOR: ParserDescriptor = ParserDescriptor {
    name: "env",
    aliases: &ALIASES,
    description: "`env` command parser",
    parser_version: "0.1.0",
    platforms: &PLATFORMS,
    tags: &TAGS,
    output: ParserOutput {
        normalized: OutputShape::Table,
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
        standard_name: "env",
        standard_version: "1.5",
        streaming_name: None,
        streaming_version: None,
    }),
};

fn var_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*=\S*.*$").expect("valid env regex"))
}

impl ScocParser for EnvParser {
    fn descriptor(&self) -> &'static ParserDescriptor {
        &DESCRIPTOR
    }

    fn parse(&self, input: &[u8], options: &ParseOptions) -> Result<Value, ScocError> {
        let text = input_to_str("env", input)?;
        let mut raw = Map::new();
        let mut key = String::new();
        let mut current: Option<String> = None;

        for line in text.lines() {
            if var_pattern().is_match(line) {
                if let Some(value) = current.take() {
                    raw.insert(key.clone(), Value::String(value));
                }
                let (name, value) = line.split_once('=').expect("regex guaranteed '='");
                key = name.to_string();
                current = Some(value.to_string());
            } else if let Some(value) = current.as_mut() {
                value.push('\n');
                value.push_str(line);
            }
        }
        if let Some(value) = current {
            raw.insert(key, Value::String(value));
        }

        if options.bool("raw").unwrap_or(false) {
            return Ok(Value::Object(raw));
        }
        let table = raw
            .into_iter()
            .map(|(name, value)| {
                let mut row = Map::new();
                row.insert("name".into(), Value::String(name));
                row.insert("value".into(), value);
                Value::Object(row)
            })
            .collect();
        Ok(Value::Array(table))
    }
}
