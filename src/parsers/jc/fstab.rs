use serde_json::{Map, Value};

use crate::utils::{input_to_str, to_i64_value};
use crate::{
    OptionSpec, OutputShape, ParseOptions, ParserCapabilities, ParserDescriptor, ParserOutput,
    ParserTag, Platform, ScocError, ScocParser, UpstreamParser,
};

const OPTIONS: [OptionSpec; 1] = [OptionSpec::bool("raw", false, "Return JC raw output")];
const PLATFORMS: [Platform; 1] = [Platform::Linux];
const TAGS: [ParserTag; 1] = [ParserTag::File];

pub struct FstabParser;
pub static FSTAB: FstabParser = FstabParser;

pub static DESCRIPTOR: ParserDescriptor = ParserDescriptor {
    name: "fstab",
    aliases: &[],
    description: "`/etc/fstab` file parser",
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
        standard_name: "fstab",
        standard_version: "1.8",
        streaming_name: None,
        streaming_version: None,
    }),
};

impl ScocParser for FstabParser {
    fn descriptor(&self) -> &'static ParserDescriptor {
        &DESCRIPTOR
    }

    fn parse(&self, input: &[u8], options: &ParseOptions) -> Result<Value, ScocError> {
        let text = input_to_str("fstab", input)?;
        let raw_mode = options.bool("raw").unwrap_or(false);
        let mut rows = Vec::new();
        for (index, line) in text.lines().enumerate() {
            if line.trim().is_empty() || line.trim_start().starts_with('#') {
                continue;
            }
            let mut fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() < 4 {
                return Err(ScocError::parse(
                    "fstab",
                    Some(index + 1),
                    "expected at least four fields",
                ));
            }
            if fields.len() == 4 {
                fields.extend(["0", "0"]);
            }
            if fields.len() == 5 {
                fields.push("0");
            }
            let mut row = Map::new();
            row.insert("fs_spec".into(), Value::String(fields[0].into()));
            row.insert("fs_file".into(), Value::String(fields[1].into()));
            row.insert("fs_vfstype".into(), Value::String(fields[2].into()));
            row.insert("fs_mntops".into(), Value::String(fields[3].into()));
            if raw_mode {
                row.insert("fs_freq".into(), Value::String(fields[4].into()));
                row.insert("fs_passno".into(), Value::String(fields[5].into()));
            } else {
                row.insert("fs_freq".into(), to_i64_value(fields[4]));
                row.insert("fs_passno".into(), to_i64_value(fields[5]));
            }
            rows.push(Value::Object(row));
        }
        Ok(Value::Array(rows))
    }
}
