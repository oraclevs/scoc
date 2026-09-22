use crate::parsers::common::{raw, StaticParser};
use crate::utils::{input_to_str, to_i64_value};
use crate::{
    OptionSpec, OutputShape, ParseOptions, ParserCapabilities, ParserDescriptor, ParserOutput,
    ParserTag, Platform, ScocError, UpstreamParser,
};
use serde_json::{Map, Value};
const OPTIONS: [OptionSpec; 1] = [OptionSpec::bool("raw", false, "Return JC raw output")];
const PLATFORMS: [Platform; 2] = [Platform::Linux, Platform::MacOs];
const TAGS: [ParserTag; 1] = [ParserTag::Command];
pub static DESCRIPTOR: ParserDescriptor = ParserDescriptor {
    name: "cksum",
    aliases: &["sum"],
    description: "`cksum` and `sum` command parser",
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
        standard_name: "cksum",
        standard_version: "1.4",
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
    let raw_mode = raw(options);
    let mut rows = vec![];
    for (idx, line) in text.lines().filter(|l| !l.trim().is_empty()).enumerate() {
        let mut p = line
            .splitn(3, char::is_whitespace)
            .filter(|v| !v.is_empty());
        let Some(checksum) = p.next() else { continue };
        let Some(blocks) = p.next() else {
            return Err(ScocError::parse(
                descriptor.name,
                Some(idx + 1),
                "missing block count",
            ));
        };
        let Some(filename) = p.next() else {
            return Err(ScocError::parse(
                descriptor.name,
                Some(idx + 1),
                "missing filename",
            ));
        };
        let mut o = Map::new();
        o.insert("filename".into(), Value::String(filename.into()));
        o.insert(
            "checksum".into(),
            if raw_mode {
                Value::String(checksum.into())
            } else {
                to_i64_value(checksum)
            },
        );
        o.insert(
            "blocks".into(),
            if raw_mode {
                Value::String(blocks.into())
            } else {
                to_i64_value(blocks)
            },
        );
        rows.push(Value::Object(o));
    }
    Ok(Value::Array(rows))
}
pub static CKSUM: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
