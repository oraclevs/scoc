use crate::parsers::common::{normalize_key, raw, StaticParser};
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
    name: "dmidecode",
    aliases: &[],
    description: "`dmidecode` command parser",
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
        standard_name: "dmidecode",
        standard_version: "1.5",
        streaming_name: None,
        streaming_version: None,
    }),
};
fn finish(
    out: &mut Vec<Value>,
    item: &mut Option<Map<String, Value>>,
    list_key: &mut Option<String>,
    list: &mut Vec<Value>,
    raw_mode: bool,
) {
    if let Some(mut o) = item.take() {
        if let Some(k) = list_key.take() {
            if let Some(Value::Object(vals)) = o.get_mut("values") {
                vals.insert(k, Value::Array(std::mem::take(list)));
            }
        }
        if !raw_mode {
            if let Some(Value::String(s)) = o.get("type").cloned() {
                o.insert("type".into(), to_i64_value(&s));
            }
            if let Some(Value::String(s)) = o.get("bytes").cloned() {
                o.insert("bytes".into(), to_i64_value(&s));
            }
            if let Some(Value::Object(v)) = o.get("values") {
                if v.is_empty() {
                    o.insert("values".into(), Value::Null);
                }
            }
        }
        out.push(Value::Object(o));
    }
}
fn parse(
    d: &'static ParserDescriptor,
    input: &[u8],
    opts: &ParseOptions,
) -> Result<Value, ScocError> {
    let text = input_to_str(d.name, input)?;
    let r = raw(opts);
    let mut out = vec![];
    let mut item: Option<Map<String, Value>> = None;
    let mut header: (String, String, String) = (String::new(), String::new(), String::new());
    let mut list_key = None;
    let mut list = vec![];
    for line in text.lines() {
        if line.starts_with("Handle ") && line.ends_with("bytes") {
            finish(&mut out, &mut item, &mut list_key, &mut list, r);
            let f = line
                .replace(',', " ")
                .split_whitespace()
                .map(str::to_string)
                .collect::<Vec<_>>();
            if f.len() >= 6 {
                header = (f[1].clone(), f[4].clone(), f[5].clone());
            }
            continue;
        }
        if !line.starts_with('\t') && !line.trim().is_empty() && !header.0.is_empty() {
            finish(&mut out, &mut item, &mut list_key, &mut list, r);
            let mut o = Map::new();
            o.insert("handle".into(), Value::String(header.0.clone()));
            o.insert("type".into(), Value::String(header.1.clone()));
            o.insert("bytes".into(), Value::String(header.2.clone()));
            o.insert("description".into(), Value::String(line.trim().into()));
            o.insert("values".into(), Value::Object(Map::new()));
            item = Some(o);
            continue;
        }
        let Some(o) = item.as_mut() else { continue };
        if line.starts_with("\t\t") {
            if list_key.is_some() {
                list.push(Value::String(line.trim().into()));
            }
            continue;
        }
        let s = line.trim();
        if s.ends_with(':') && !s.contains(": ") {
            if let Some(k) = list_key.take() {
                if let Some(Value::Object(v)) = o.get_mut("values") {
                    v.insert(k, Value::Array(std::mem::take(&mut list)));
                }
            }
            list_key = Some(normalize_key(s.trim_end_matches(':')));
            continue;
        }
        if let Some((k, v)) = s.split_once(':') {
            if let Some(lk) = list_key.take() {
                if let Some(Value::Object(vals)) = o.get_mut("values") {
                    vals.insert(lk, Value::Array(std::mem::take(&mut list)));
                }
            }
            if let Some(Value::Object(vals)) = o.get_mut("values") {
                vals.insert(normalize_key(k), Value::String(v.trim().into()));
            }
        }
    }
    finish(&mut out, &mut item, &mut list_key, &mut list, r);
    Ok(Value::Array(out))
}
pub static DMIDECODE: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
