use crate::parsers::common::{raw, StaticParser};
use crate::utils::{input_to_str, sparse_table_parse, to_i64_value};
use crate::{
    OptionSpec, OutputShape, ParseOptions, ParserCapabilities, ParserDescriptor, ParserOutput,
    ParserTag, Platform, ScocError, UpstreamParser,
};
use serde_json::Value;
const OPTIONS: [OptionSpec; 1] = [OptionSpec::bool("raw", false, "Return JC raw output")];
const PLATFORMS: [Platform; 2] = [Platform::Linux, Platform::MacOs];
const TAGS: [ParserTag; 1] = [ParserTag::Command];
pub static DESCRIPTOR: ParserDescriptor = ParserDescriptor {
    name: "lsof",
    aliases: &[],
    description: "`lsof` command parser",
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
        standard_name: "lsof",
        standard_version: "1.6",
        streaming_name: None,
        streaming_version: None,
    }),
};
fn parse(
    d: &'static ParserDescriptor,
    input: &[u8],
    opts: &ParseOptions,
) -> Result<Value, ScocError> {
    let text = input_to_str(d.name, input)?;
    let mut lines = text
        .lines()
        .filter(|x| !x.trim().is_empty())
        .collect::<Vec<_>>();
    if lines.is_empty() {
        return Ok(Value::Array(vec![]));
    }
    let header = lines[0].to_ascii_lowercase().replace('/', "_");
    let owned = std::iter::once(header.as_str())
        .chain(lines.drain(1..))
        .collect::<Vec<_>>();
    let mut rows = sparse_table_parse(&owned)?;
    if !raw(opts) {
        for row in &mut rows {
            if let Value::Object(o) = row {
                for k in ["pid", "tid", "size_off", "node"] {
                    if let Some(Value::String(v)) = o.get(k).cloned() {
                        o.insert(k.into(), to_i64_value(&v));
                    }
                }
            }
        }
    }
    Ok(Value::Array(rows))
}
pub static LSOF: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
