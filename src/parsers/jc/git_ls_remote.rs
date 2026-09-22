use crate::parsers::common::{raw, StaticParser};
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
    name: "git-ls-remote",
    aliases: &[],
    description: "`git ls-remote` command parser",
    parser_version: "0.1.0",
    platforms: &PLATFORMS,
    tags: &TAGS,
    output: ParserOutput {
        normalized: OutputShape::Record,
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
        standard_name: "git-ls-remote",
        standard_version: "1.0",
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
    let raw_mode = raw(opts);
    let mut obj = Map::new();
    let mut rows = Vec::new();
    for (n, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let mut it = line.split_whitespace();
        let Some(commit) = it.next() else { continue };
        let Some(reference) = it.next() else {
            return Err(ScocError::parse(
                d.name,
                Some(n + 1),
                "expected commit and reference",
            ));
        };
        if raw_mode {
            let mut o = Map::new();
            o.insert("reference".into(), Value::String(reference.into()));
            o.insert("commit".into(), Value::String(commit.into()));
            rows.push(Value::Object(o));
        } else {
            obj.insert(reference.into(), Value::String(commit.into()));
        }
    }
    Ok(if raw_mode {
        Value::Array(rows)
    } else {
        Value::Object(obj)
    })
}
pub static GIT_LS_REMOTE: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
