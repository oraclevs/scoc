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
    name: "ip-route",
    aliases: &[],
    description: "`ip route` command parser",
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
        standard_name: "ip-route",
        standard_version: "1.1",
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
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let f = line.split_whitespace().collect::<Vec<_>>();
        if f.is_empty() {
            continue;
        }
        let mut o = Map::new();
        o.insert("ip".into(), Value::String(f[0].into()));
        let mut i = 1;
        while i < f.len() {
            match f[i] {
                "via" | "dev" | "proto" | "scope" | "src" | "status" if i + 1 < f.len() => {
                    let k = f[i];
                    let v = f[i + 1];
                    o.insert(k.into(), Value::String(v.into()));
                    i += 2
                }
                "metric" if i + 1 < f.len() => {
                    o.insert(
                        "metric".into(),
                        if r {
                            Value::String(f[i + 1].into())
                        } else {
                            to_i64_value(f[i + 1])
                        },
                    );
                    i += 2
                }
                "linkdown" => {
                    o.insert("status".into(), Value::String("linkdown".into()));
                    i += 1
                }
                _ => i += 1,
            }
        }
        out.push(Value::Object(o));
    }
    Ok(Value::Array(out))
}
pub static IP_ROUTE: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
