use crate::parsers::common::{raw, StaticParser};
use crate::utils::{input_to_str, to_i64_value};
use crate::{
    OptionSpec, OutputShape, ParseOptions, ParserCapabilities, ParserDescriptor, ParserOutput,
    ParserTag, Platform, ScocError, UpstreamParser,
};
use serde_json::{Map, Value};
const OPTIONS: [OptionSpec; 1] = [OptionSpec::bool("raw", false, "Return JC raw output")];
const PLATFORMS: [Platform; 2] = [Platform::Linux, Platform::MacOs];
const TAGS: [ParserTag; 1] = [ParserTag::File];
pub static DESCRIPTOR: ParserDescriptor = ParserDescriptor {
    name: "group",
    aliases: &[],
    description: "`/etc/group` file parser",
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
        standard_name: "group",
        standard_version: "1.5",
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
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let f = line.split(':').collect::<Vec<_>>();
        if f.len() < 4 {
            return Err(ScocError::parse(
                d.name,
                Some(n + 1),
                "expected four colon-separated fields",
            ));
        }
        let mut o = Map::new();
        o.insert("group_name".into(), Value::String(f[0].into()));
        o.insert("password".into(), Value::String(f[1].into()));
        o.insert(
            "gid".into(),
            if r {
                Value::String(f[2].into())
            } else {
                to_i64_value(f[2])
            },
        );
        let members = f[3]
            .split(',')
            .map(|x| Value::String(x.into()))
            .collect::<Vec<_>>();
        o.insert(
            "members".into(),
            if !r && members.len() == 1 && members[0] == Value::String(String::new()) {
                Value::Array(vec![])
            } else {
                Value::Array(members)
            },
        );
        out.push(Value::Object(o));
    }
    Ok(Value::Array(out))
}
pub static GROUP: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
