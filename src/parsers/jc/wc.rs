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
    name: "wc",
    aliases: &[],
    description: "`wc` command parser",
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
        standard_name: "wc",
        standard_version: "1.4",
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
    let mut out = vec![];
    for (n, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let f = line.split_whitespace().collect::<Vec<_>>();
        if f.len() < 3 {
            return Err(ScocError::parse(
                d.name,
                Some(n + 1),
                "expected line word character counts",
            ));
        }
        let mut o = Map::new();
        o.insert(
            "filename".into(),
            f.get(3)
                .map(|x| Value::String((*x).into()))
                .unwrap_or(Value::Null),
        );
        for (i, k) in ["lines", "words", "characters"].iter().enumerate() {
            o.insert(
                (*k).into(),
                if r {
                    Value::String(f[i].into())
                } else {
                    to_i64_value(f[i])
                },
            );
        }
        out.push(Value::Object(o));
    }
    Ok(Value::Array(out))
}
pub static WC: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
