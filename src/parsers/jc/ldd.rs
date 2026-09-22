use crate::parsers::common::StaticParser;
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
    name: "ldd",
    aliases: &[],
    description: "`ldd` command parser",
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
        standard_name: "ldd",
        standard_version: "1.0",
        streaming_name: None,
        streaming_version: None,
    }),
};
fn new_file(name: Option<String>) -> Map<String, Value> {
    let mut o = Map::new();
    o.insert(
        "file".into(),
        name.map(Value::String).unwrap_or(Value::Null),
    );
    o.insert("dependencies".into(), Value::Array(vec![]));
    o.insert("version_info".into(), Value::Array(vec![]));
    o
}
fn parse(d: &'static ParserDescriptor, input: &[u8], _: &ParseOptions) -> Result<Value, ScocError> {
    let text = input_to_str(d.name, input)?;
    let addr = Regex::new(r"^(\S+)\s*\((0x[0-9a-fA-F]+)\)$")
        .map_err(|e| ScocError::parse(d.name, None, e.to_string()))?;
    let req = Regex::new(r"^(\S+)\s*\(([^)]+)\)\s*=>\s*(\S+)$")
        .map_err(|e| ScocError::parse(d.name, None, e.to_string()))?;
    let mut files: Vec<Map<String, Value>> = vec![];
    let mut current: Option<Map<String, Value>> = None;
    let mut version = false;
    let mut vobj: Option<Map<String, Value>> = None;
    for line in text.lines() {
        let s = line.trim();
        if s.is_empty() {
            continue;
        }
        let ind = line.starts_with(' ') || line.starts_with('\t');
        if !ind && s.ends_with(':') && s != "Version information:" {
            if let Some(mut f) = current.take() {
                if let Some(v) = vobj.take() {
                    if let Some(Value::Array(a)) = f.get_mut("version_info") {
                        a.push(Value::Object(v));
                    }
                }
                files.push(f)
            }
            current = Some(new_file(Some(s.trim_end_matches(':').into())));
            version = false;
            continue;
        }
        if s == "Version information:" {
            version = true;
            continue;
        }
        if version {
            if s.ends_with(':') {
                if let Some(v) = vobj.take() {
                    let f = current.get_or_insert_with(|| new_file(None));
                    if let Some(Value::Array(a)) = f.get_mut("version_info") {
                        a.push(Value::Object(v));
                    }
                }
                let mut v = Map::new();
                v.insert("for".into(), Value::String(s.trim_end_matches(':').into()));
                v.insert("requires".into(), Value::Array(vec![]));
                vobj = Some(v);
                continue;
            }
            if let Some(c) = req.captures(s) {
                let v = vobj.get_or_insert_with(|| {
                    let mut x = Map::new();
                    x.insert("for".into(), Value::Null);
                    x.insert("requires".into(), Value::Array(vec![]));
                    x
                });
                let mut r = Map::new();
                r.insert("name".into(), Value::String(c[1].into()));
                r.insert("version".into(), Value::String(c[2].into()));
                r.insert("path".into(), Value::String(c[3].into()));
                if let Some(Value::Array(a)) = v.get_mut("requires") {
                    a.push(Value::Object(r));
                }
                continue;
            }
        }
        let f = current.get_or_insert_with(|| new_file(None));
        let mut dep = Map::new();
        if let Some((name, rest)) = s.split_once("=>") {
            let name = name.trim();
            let rest = rest.trim();
            dep.insert("name".into(), Value::String(name.into()));
            if rest == "not found" {
                dep.insert("path".into(), Value::Null);
                dep.insert("address".into(), Value::Null);
            } else if let Some(c) = addr.captures(rest) {
                dep.insert("path".into(), Value::String(c[1].into()));
                dep.insert("address".into(), Value::String(c[2].into()));
            } else {
                continue;
            }
        } else if let Some(c) = addr.captures(s) {
            dep.insert("name".into(), Value::String(c[1].into()));
            dep.insert("path".into(), Value::Null);
            dep.insert("address".into(), Value::String(c[2].into()));
        } else {
            continue;
        }
        if let Some(Value::Array(a)) = f.get_mut("dependencies") {
            a.push(Value::Object(dep));
        }
    }
    if let Some(mut f) = current {
        if let Some(v) = vobj {
            if let Some(Value::Array(a)) = f.get_mut("version_info") {
                a.push(Value::Object(v));
            }
        }
        files.push(f)
    }
    Ok(Value::Array(files.into_iter().map(Value::Object).collect()))
}
pub static LDD: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
