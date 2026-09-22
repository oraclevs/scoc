use crate::parsers::common::{raw, StaticParser};
use crate::utils::input_to_str;
use crate::{
    OptionSpec, OutputShape, ParseOptions, ParserCapabilities, ParserDescriptor, ParserOutput,
    ParserTag, Platform, ScocError, UpstreamParser,
};
use regex::Regex;
use serde_json::{Map, Value};
const OPTIONS: [OptionSpec; 1] = [OptionSpec::bool("raw", false, "Return JC raw output")];
const PLATFORMS: [Platform; 1] = [Platform::Linux];
const TAGS: [ParserTag; 1] = [ParserTag::Command];
pub static DESCRIPTOR: ParserDescriptor = ParserDescriptor {
    name: "lsattr",
    aliases: &[],
    description: "`lsattr` command parser",
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
        standard_name: "lsattr",
        standard_version: "1.1",
        streaming_name: None,
        streaming_version: None,
    }),
};
fn attr(c: char) -> Option<&'static str> {
    Some(match c {
        'B' => "compressed_file",
        'Z' => "compressed_dirty_file",
        'X' => "compression_raw_access",
        's' => "secure_deletion",
        'u' => "undelete",
        'S' => "synchronous_updates",
        'D' => "synchronous_directory_updates",
        'i' => "immutable",
        'a' => "append_only",
        'd' => "no_dump",
        'A' => "no_atime",
        'c' => "compression_requested",
        'E' => "encrypted",
        'j' => "journaled_data",
        'I' => "indexed_directory",
        't' => "no_tailmerging",
        'T' => "top_of_directory_hierarchies",
        'e' => "extents",
        'C' => "no_cow",
        'F' => "casefold",
        'N' => "inline_data",
        'P' => "project_hierarchy",
        'V' => "verity",
        _ => return None,
    })
}
fn parse(
    d: &'static ParserDescriptor,
    input: &[u8],
    opts: &ParseOptions,
) -> Result<Value, ScocError> {
    let text = input_to_str(d.name, input)?;
    let r = raw(opts);
    let re = Regex::new(r"^([BZXsuSDiadAcEjItTeCFNPV-]{20}) (.*)$")
        .map_err(|e| ScocError::parse(d.name, None, e.to_string()))?;
    let mut out = vec![];
    for line in text.lines() {
        let Some(c) = re.captures(line) else { continue };
        let attrs = c[1].to_string();
        let mut o = Map::new();
        o.insert("file".into(), Value::String(c[2].to_string()));
        if r {
            o.insert("attributes".into(), Value::String(attrs));
        } else {
            for a in attrs.chars() {
                if let Some(k) = attr(a) {
                    o.insert(k.into(), Value::Bool(true));
                }
            }
        }
        out.push(Value::Object(o));
    }
    Ok(Value::Array(out))
}
pub static LSATTR: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
