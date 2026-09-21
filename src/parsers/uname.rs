use crate::parsers::common::StaticParser;
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
    name: "uname",
    aliases: &[],
    description: "`uname -a` command parser",
    parser_version: "0.1.0",
    platforms: &PLATFORMS,
    tags: &TAGS,
    output: ParserOutput {
        normalized: OutputShape::Record,
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
        standard_name: "uname",
        standard_version: "1.8",
        streaming_name: None,
        streaming_version: None,
    }),
};
fn parse(d: &'static ParserDescriptor, input: &[u8], _: &ParseOptions) -> Result<Value, ScocError> {
    let text = input_to_str(d.name, input)?;
    let data = text.trim();
    if data.is_empty() {
        return Ok(Value::Object(Map::new()));
    }
    let f = data.split_whitespace().collect::<Vec<_>>();
    let mut o = Map::new();
    if data.starts_with("Darwin") || data.starts_with("FreeBSD") {
        if f.len() < 5 {
            return Err(ScocError::parse(
                d.name,
                None,
                "Could not parse uname output. Make sure to use uname -a",
            ));
        }
        o.insert("kernel_name".into(), Value::String(f[0].into()));
        o.insert("node_name".into(), Value::String(f[1].into()));
        o.insert("kernel_release".into(), Value::String(f[2].into()));
        o.insert(
            "kernel_version".into(),
            Value::String(f[3..f.len() - 1].join(" ")),
        );
        o.insert("machine".into(), Value::String(f[f.len() - 1].into()));
    } else {
        let mut data = data.to_string();
        if f.len() >= 4 {
            let mut set = vec![f[f.len() - 2], f[f.len() - 3], f[f.len() - 4]];
            set.sort_unstable();
            set.dedup();
            if set.len() > 2 {
                let mut fixup = f.clone();
                fixup.insert(fixup.len() - 1, "unknown");
                fixup.insert(fixup.len() - 1, "unknown");
                data = fixup.join(" ");
            }
        }
        let bad = || {
            ScocError::parse(
                d.name,
                None,
                "Could not parse uname output. Make sure to use uname -a",
            )
        };
        // Python `str.split(maxsplit=3)`: the remainder keeps its spacing.
        let mut rest = data.as_str();
        let mut head = Vec::new();
        for _ in 0..3 {
            rest = rest.trim_start();
            let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
            if end == 0 {
                return Err(bad());
            }
            head.push(&rest[..end]);
            rest = &rest[end..];
        }
        let rest = rest.trim_start();
        // Python `str.rsplit(maxsplit=4)`.
        let mut tail = Vec::new();
        let mut remaining = rest.trim_end();
        for _ in 0..4 {
            let Some(index) = remaining.rfind(char::is_whitespace) else {
                break;
            };
            tail.push(&remaining[index + 1..]);
            remaining = remaining[..index].trim_end();
        }
        if tail.len() < 4 {
            return Err(bad());
        }
        tail.push(remaining);
        o.insert("kernel_name".into(), Value::String(head[0].into()));
        o.insert("node_name".into(), Value::String(head[1].into()));
        o.insert("kernel_release".into(), Value::String(head[2].into()));
        o.insert("operating_system".into(), Value::String(tail[0].into()));
        o.insert("processor".into(), Value::String(tail[1].into()));
        o.insert("hardware_platform".into(), Value::String(tail[2].into()));
        o.insert("machine".into(), Value::String(tail[3].into()));
        o.insert("kernel_version".into(), Value::String(tail[4].into()));
    }
    Ok(Value::Object(o))
}
pub static UNAME: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
