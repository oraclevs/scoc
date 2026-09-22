use crate::parsers::common::{raw, StaticParser};
use crate::utils::{input_to_str, simple_table_parse, to_i64_value};
use crate::{
    OptionSpec, OutputShape, ParseOptions, ParserCapabilities, ParserDescriptor, ParserOutput,
    ParserTag, Platform, ScocError, UpstreamParser,
};
use serde_json::Value;
const OPTIONS: [OptionSpec; 1] = [OptionSpec::bool("raw", false, "Return JC raw output")];
const PLATFORMS: [Platform; 1] = [Platform::Linux];
const TAGS: [ParserTag; 1] = [ParserTag::Command];
pub static DESCRIPTOR: ParserDescriptor = ParserDescriptor {
    name: "route",
    aliases: &[],
    description: "`route` command parser",
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
        standard_name: "route",
        standard_version: "1.9",
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
    let mut lines = text.lines().filter(|x| !x.trim().is_empty());
    let _ = lines.next();
    let Some(h) = lines.next() else {
        return Ok(Value::Array(vec![]));
    };
    let h = h
        .replace(" Next Hop ", " Next_Hop ")
        .replace(" Flag ", " Flags ")
        .replace(" Met ", " Metric ")
        .replace(" If", " Iface")
        .to_ascii_lowercase();
    let body = lines.collect::<Vec<_>>();
    let mut all = vec![h.as_str()];
    all.extend(body);
    let mut rows = simple_table_parse(&all)?;
    if !raw(opts) {
        for row in &mut rows {
            if let Value::Object(o) = row {
                for k in ["metric", "ref", "use", "mss", "window", "irtt"] {
                    if let Some(Value::String(s)) = o.get(k).cloned() {
                        o.insert(k.into(), to_i64_value(&s));
                    }
                }
                if let Some(Value::String(flags)) = o.get("flags").cloned() {
                    let mut a = vec![];
                    for c in flags.chars() {
                        let n = match c {
                            'U' => Some("UP"),
                            'H' => Some("HOST"),
                            'G' => Some("GATEWAY"),
                            'R' => Some("REINSTATE"),
                            'D' => Some("DYNAMIC"),
                            'M' => Some("MODIFIED"),
                            'A' => Some("ADDRCONF"),
                            'C' => Some("CACHE"),
                            '!' => Some("REJECT"),
                            _ => None,
                        };
                        if let Some(n) = n {
                            a.push(Value::String(n.into()));
                        }
                    }
                    o.insert("flags_pretty".into(), Value::Array(a));
                }
            }
        }
    }
    Ok(Value::Array(rows))
}
pub static ROUTE: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
