use crate::parsers::common::{raw, StaticParser};
use crate::utils::{input_to_str, to_i64_value};
use crate::{
    OptionSpec, OutputShape, ParseOptions, ParserCapabilities, ParserDescriptor, ParserOutput,
    ParserTag, Platform, ScocError, UpstreamParser,
};
use serde_json::{Map, Value};
const OPTIONS: [OptionSpec; 1] = [OptionSpec::bool("raw", false, "Return JC raw output")];
const PLATFORMS: [Platform; 2] = [Platform::Linux, Platform::MacOs];
const TAGS: [ParserTag; 1] = [ParserTag::Command];
pub static DESCRIPTOR: ParserDescriptor = ParserDescriptor {
    name: "host",
    aliases: &[],
    description: "`host` command parser",
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
        standard_name: "host",
        standard_version: "1.0",
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
    let mut default = Map::new();
    let mut addrs = vec![];
    let mut v6 = vec![];
    let mut mail = vec![];
    let mut txt = vec![];
    let mut current_soa: Option<Map<String, Value>> = None;
    for (n, line) in text.lines().enumerate() {
        let s = line.trim();
        if s.is_empty() {
            continue;
        }
        if let Some(rest) = s.strip_prefix("Nameserver ") {
            if let Some(obj) = current_soa.take() {
                out.push(Value::Object(obj));
            }
            let mut o = Map::new();
            o.insert(
                "nameserver".into(),
                Value::String(rest.trim_end_matches(':').into()),
            );
            current_soa = Some(o);
            continue;
        }
        if s.contains(" has SOA record ") {
            let f = s.split_whitespace().collect::<Vec<_>>();
            if f.len() < 11 {
                return Err(ScocError::parse(d.name, Some(n + 1), "invalid SOA record"));
            }
            let o = current_soa.get_or_insert_with(Map::new);
            o.insert("zone".into(), Value::String(f[0].into()));
            o.insert("mname".into(), Value::String(f[4].into()));
            o.insert("rname".into(), Value::String(f[5].into()));
            for (i, k) in [
                (6, "serial"),
                (7, "refresh"),
                (8, "retry"),
                (9, "expire"),
                (10, "minimum"),
            ] {
                o.insert(
                    k.into(),
                    if r {
                        Value::String(f[i].into())
                    } else {
                        to_i64_value(f[i])
                    },
                );
            }
            continue;
        }
        if let Some((h, a)) = s.split_once(" has address ") {
            default.insert("hostname".into(), Value::String(h.into()));
            addrs.push(Value::String(a.into()));
            default.insert("address".into(), Value::Array(addrs.clone()));
            continue;
        }
        if let Some((h, a)) = s.split_once(" has IPv6 address ") {
            default.insert("hostname".into(), Value::String(h.into()));
            v6.push(Value::String(a.into()));
            default.insert("v6-address".into(), Value::Array(v6.clone()));
            continue;
        }
        if let Some((h, rest)) = s.split_once(" mail is handled by ") {
            default.insert("hostname".into(), Value::String(h.into()));
            let mx = rest.split_whitespace().last().unwrap_or(rest);
            mail.push(Value::String(mx.into()));
            default.insert("mail".into(), Value::Array(mail.clone()));
            continue;
        }
        if let Some((h, rest)) = s.split_once(" descriptive text ") {
            default.insert("hostname".into(), Value::String(h.trim().into()));
            txt.push(Value::String(rest.trim_matches('"').into()));
            default.insert("text".into(), Value::Array(txt.clone()));
        }
    }
    if let Some(obj) = current_soa {
        out.push(Value::Object(obj));
    } else if !default.is_empty() {
        out.push(Value::Object(default));
    }
    Ok(Value::Array(out))
}
pub static HOST: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
