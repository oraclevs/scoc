use crate::parsers::common::{split_whitespace_max, StaticParser};
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
    name: "systemctl",
    aliases: &[],
    description: "`systemctl` command parser",
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
        standard_name: "systemctl",
        standard_version: "1.5",
        streaming_name: None,
        streaming_version: None,
    }),
};
fn parse(d: &'static ParserDescriptor, input: &[u8], _: &ParseOptions) -> Result<Value, ScocError> {
    let text = input_to_str(d.name, input)?;
    // JC drops non-ASCII glyphs (the `●` status marker) before splitting.
    let mut lines = text
        .lines()
        .map(|line| line.chars().filter(char::is_ascii).collect::<String>())
        .filter(|line| !line.is_empty());
    let Some(header) = lines.next() else {
        return Ok(Value::Array(vec![]));
    };
    let keys = header
        .split_whitespace()
        .map(|x| x.to_lowercase())
        .collect::<Vec<_>>();
    let mut out = vec![];
    for line in lines {
        if line.contains("LOAD   = ") {
            break;
        }
        let mut o = Map::new();
        for (key, value) in keys.iter().zip(split_whitespace_max(&line, 4)) {
            o.insert(key.clone(), Value::String(value.into()));
        }
        out.push(Value::Object(o));
    }
    Ok(Value::Array(out))
}
pub static SYSTEMCTL: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
