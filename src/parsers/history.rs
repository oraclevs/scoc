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
    name: "history",
    aliases: &[],
    description: "`history` command parser",
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
        standard_name: "history",
        standard_version: "1.7",
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
    let r = raw(opts);
    let mut obj = Map::new();
    let mut rows = vec![];
    for line in text.lines().filter(|l| !l.is_empty()) {
        let Some(pos) = line.trim_start().find(char::is_whitespace) else {
            continue;
        };
        let s = line.trim_start();
        let num = &s[..pos];
        let cmd = s[pos..].trim_start();
        if r {
            obj.insert(num.into(), Value::String(cmd.into()));
        } else {
            let mut o = Map::new();
            o.insert("line".into(), to_i64_value(num));
            o.insert("command".into(), Value::String(cmd.into()));
            rows.push(Value::Object(o));
        }
    }
    Ok(if r {
        Value::Object(obj)
    } else {
        Value::Array(rows)
    })
}
pub static HISTORY: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
