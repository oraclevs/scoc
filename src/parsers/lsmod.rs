use crate::parsers::common::{raw, StaticParser};
use crate::utils::{input_to_str, to_i64_value};
use crate::{
    OptionSpec, OutputShape, ParseOptions, ParserCapabilities, ParserDescriptor, ParserOutput,
    ParserTag, Platform, ScocError, UpstreamParser,
};
use serde_json::{Map, Value};
const OPTIONS: [OptionSpec; 1] = [OptionSpec::bool("raw", false, "Return JC raw output")];
const PLATFORMS: [Platform; 1] = [Platform::Linux];
const TAGS: [ParserTag; 1] = [ParserTag::Command];
pub static DESCRIPTOR: ParserDescriptor = ParserDescriptor {
    name: "lsmod",
    aliases: &[],
    description: "`lsmod` command parser",
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
        standard_name: "lsmod",
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
    let mut out = vec![];
    for (n, line) in text.lines().enumerate().skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        let f = line.split_whitespace().collect::<Vec<_>>();
        if f.len() < 3 {
            return Err(ScocError::parse(
                d.name,
                Some(n + 1),
                "expected Module Size Used [by]",
            ));
        }
        let mut o = Map::new();
        o.insert("module".into(), Value::String(f[0].into()));
        o.insert(
            "size".into(),
            if r {
                Value::String(f[1].into())
            } else {
                to_i64_value(f[1])
            },
        );
        o.insert(
            "used".into(),
            if r {
                Value::String(f[2].into())
            } else {
                to_i64_value(f[2])
            },
        );
        if let Some(by) = f.get(3) {
            o.insert(
                "by".into(),
                Value::Array(
                    by.trim_end_matches(',')
                        .split(',')
                        .filter(|x| !x.is_empty())
                        .map(|x| Value::String(x.into()))
                        .collect(),
                ),
            );
        }
        out.push(Value::Object(o));
    }
    Ok(Value::Array(out))
}
pub static LSMOD: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
