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
    name: "shadow",
    aliases: &[],
    description: "`/etc/shadow` file parser",
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
        standard_name: "shadow",
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
        if f.len() < 8 {
            return Err(ScocError::parse(
                d.name,
                Some(n + 1),
                "expected at least eight shadow fields",
            ));
        }
        let mut o = Map::new();
        o.insert("username".into(), Value::String(f[0].into()));
        o.insert("password".into(), Value::String(f[1].into()));
        for (i, k) in [
            "last_changed",
            "minimum",
            "maximum",
            "warn",
            "inactive",
            "expire",
        ]
        .iter()
        .enumerate()
        {
            let v = f[i + 2];
            o.insert(
                (*k).into(),
                if r {
                    Value::String(v.into())
                } else {
                    to_i64_value(v)
                },
            );
        }
        out.push(Value::Object(o));
    }
    Ok(Value::Array(out))
}
pub static SHADOW: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
