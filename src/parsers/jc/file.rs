use crate::parsers::common::StaticParser;
use crate::utils::input_to_str;
use crate::{
    OptionSpec, OutputShape, ParseOptions, ParserCapabilities, ParserDescriptor, ParserOutput,
    ParserTag, Platform, ScocError, UpstreamParser,
};
use serde_json::{Map, Value};
const OPTIONS: [OptionSpec; 1] = [OptionSpec::bool("raw", false, "Return JC raw output")];
const PLATFORMS: [Platform; 2] = [Platform::Linux, Platform::MacOs];
const TAGS: [ParserTag; 1] = [ParserTag::Command];
pub static DESCRIPTOR: ParserDescriptor = ParserDescriptor {
    name: "file",
    aliases: &[],
    description: "`file` command parser",
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
        standard_name: "file",
        standard_version: "1.5",
        streaming_name: None,
        streaming_version: None,
    }),
};
fn parse(
    descriptor: &'static ParserDescriptor,
    input: &[u8],
    _: &ParseOptions,
) -> Result<Value, ScocError> {
    let text = input_to_str(descriptor.name, input)?;
    let mut rows = vec![];
    for line in text.lines().filter(|l| !l.is_empty()) {
        let pair = if line.contains("gzip compressed data, last modified: ") {
            line.split_once(": ")
        } else {
            line.rsplit_once(": ")
        };
        let Some((filename, kind)) = pair else {
            continue;
        };
        let mut o = Map::new();
        o.insert("filename".into(), Value::String(filename.trim().into()));
        o.insert("type".into(), Value::String(kind.trim().into()));
        rows.push(Value::Object(o));
    }
    Ok(Value::Array(rows))
}
pub static FILE: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
