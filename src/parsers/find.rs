use crate::parsers::common::{raw, StaticParser};
use crate::utils::input_to_str;
use crate::{
    OptionSpec, OutputShape, ParseOptions, ParserCapabilities, ParserDescriptor, ParserOutput,
    ParserTag, Platform, ScocError, UpstreamParser,
};
use serde_json::{Map, Value};
const OPTIONS: [OptionSpec; 1] = [OptionSpec::bool("raw", false, "Return JC raw output")];
const PLATFORMS: [Platform; 1] = [Platform::Linux];
const TAGS: [ParserTag; 1] = [ParserTag::Command];
pub static DESCRIPTOR: ParserDescriptor = ParserDescriptor {
    name: "find",
    aliases: &[],
    description: "`find` command parser",
    parser_version: "0.1.0",
    platforms: &PLATFORMS,
    tags: &TAGS,
    output: ParserOutput {
        normalized: OutputShape::Table,
        raw: Some(OutputShape::List),
        stream_item: None,
    },
    capabilities: ParserCapabilities {
        raw: true,
        streaming: false,
        ignore_errors: false,
    },
    options: &OPTIONS,
    upstream: Some(UpstreamParser {
        standard_name: "find",
        standard_version: "1.0",
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
    let lines: Vec<&str> = text.lines().collect();
    if raw(options) {
        return Ok(Value::Array(
            lines.into_iter().map(|v| Value::String(v.into())).collect(),
        ));
    }
    let mut rows = vec![];
    for line in lines {
        let mut o = Map::new();
        if line == "." {
            o.insert("path".into(), Value::Null);
            o.insert("node".into(), Value::String(".".into()));
        } else if line.starts_with("find: ") {
            o.insert("path".into(), Value::Null);
            o.insert("node".into(), Value::Null);
            o.insert("error".into(), Value::String(line.into()));
        } else if let Some((path, node)) = line.rsplit_once('/') {
            o.insert(
                "path".into(),
                if path.is_empty() {
                    Value::Null
                } else {
                    Value::String(path.into())
                },
            );
            o.insert(
                "node".into(),
                if node.is_empty() {
                    Value::Null
                } else {
                    Value::String(node.into())
                },
            );
        } else {
            o.insert("path".into(), Value::Null);
            o.insert("node".into(), Value::Null);
        }
        rows.push(Value::Object(o));
    }
    Ok(Value::Array(rows))
}
pub static FIND: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
