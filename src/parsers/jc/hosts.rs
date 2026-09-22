use crate::parsers::common::StaticParser;
use crate::utils::input_to_str;
use crate::{
    OptionSpec, OutputShape, ParseOptions, ParserCapabilities, ParserDescriptor, ParserOutput,
    ParserTag, Platform, ScocError, UpstreamParser,
};
use serde_json::{Map, Value};
const OPTIONS: [OptionSpec; 1] = [OptionSpec::bool("raw", false, "Return JC raw output")];
const PLATFORMS: [Platform; 2] = [Platform::Linux, Platform::MacOs];
const TAGS: [ParserTag; 1] = [ParserTag::File];
pub static DESCRIPTOR: ParserDescriptor = ParserDescriptor {
    name: "hosts",
    aliases: &[],
    description: "`/etc/hosts` file parser",
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
        standard_name: "hosts",
        standard_version: "1.5",
        streaming_name: None,
        streaming_version: None,
    }),
};
fn parse(d: &'static ParserDescriptor, input: &[u8], _: &ParseOptions) -> Result<Value, ScocError> {
    let text = input_to_str(d.name, input)?;
    let mut out = vec![];
    for (n, line) in text.lines().enumerate() {
        let s = line.trim();
        if s.is_empty() || s.starts_with('#') {
            continue;
        }
        let clean = s.split('#').next().unwrap_or("").trim();
        let mut fields = clean.split_whitespace();
        let Some(ip) = fields.next() else { continue };
        let hosts = fields.map(|x| Value::String(x.into())).collect::<Vec<_>>();
        if hosts.is_empty() {
            return Err(ScocError::parse(d.name, Some(n + 1), "missing hostname"));
        }
        let mut o = Map::new();
        o.insert("ip".into(), Value::String(ip.into()));
        o.insert("hostname".into(), Value::Array(hosts));
        out.push(Value::Object(o));
    }
    Ok(Value::Array(out))
}
pub static HOSTS: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
