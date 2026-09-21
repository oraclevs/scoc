use crate::parsers::common::{raw, StaticParser};
use crate::utils::{convert_size_to_int, input_to_str, to_i64_value};
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
    name: "iptables",
    aliases: &[],
    description: "`iptables` command parser",
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
        standard_name: "iptables",
        standard_version: "1.13",
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
    let stats = Regex::new(r"\(policy (.+) (\S+) packets, (\S+) bytes\)")
        .map_err(|e| ScocError::parse(d.name, None, e.to_string()))?;
    let mut chains = vec![];
    let mut chain: Option<Map<String, Value>> = None;
    let mut headers: Vec<String> = vec![];
    for (n, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        if line.starts_with("Chain ") {
            if let Some(c) = chain.take() {
                chains.push(Value::Object(c));
            }
            let f = line.split_whitespace().collect::<Vec<_>>();
            if f.len() < 2 {
                return Err(ScocError::parse(
                    d.name,
                    Some(n + 1),
                    "invalid chain header",
                ));
            }
            let mut c = Map::new();
            c.insert("chain".into(), Value::String(f[1].into()));
            if let Some(x) = stats.captures(line) {
                c.insert("default_policy".into(), Value::String(x[1].into()));
                c.insert(
                    "default_packets".into(),
                    if r {
                        Value::String(x[2].into())
                    } else {
                        to_i64_value(&x[2])
                    },
                );
                c.insert(
                    "default_bytes".into(),
                    if r {
                        Value::String(x[3].into())
                    } else {
                        convert_size_to_int(&x[3], false, false)
                            .map(Value::from)
                            .unwrap_or(Value::Null)
                    },
                );
            }
            c.insert("rules".into(), Value::Array(vec![]));
            chain = Some(c);
            headers.clear();
            continue;
        }
        let t = line.trim_start();
        if t.starts_with("target") || t.starts_with("pkts") || t.starts_with("num") {
            headers = t
                .to_ascii_lowercase()
                .split_whitespace()
                .map(str::to_string)
                .collect();
            headers.push("options".into());
            continue;
        }
        let Some(c) = chain.as_mut() else { continue };
        if headers.is_empty() {
            continue;
        }
        let mut tokens = line.split_whitespace().collect::<Vec<_>>();
        if headers.first().map(String::as_str) == Some("target") && line.starts_with(' ') {
            tokens.insert(0, "");
        } else if headers.first().map(String::as_str) == Some("pkts")
            && tokens.len() > 3
            && matches!(tokens[3], "--" | "-f" | "!f")
        {
            tokens.insert(2, "");
        }
        let mut o = Map::new();
        for (i, k) in headers.iter().enumerate() {
            if i >= tokens.len() {
                break;
            }
            let v = if i + 1 == headers.len() {
                tokens[i..].join(" ")
            } else {
                tokens[i].to_string()
            };
            let val = if !r && matches!(k.as_str(), "num" | "pkts") {
                to_i64_value(&v)
            } else if !r && k == "bytes" {
                convert_size_to_int(&v, false, false)
                    .map(Value::from)
                    .unwrap_or(Value::Null)
            } else if !r && ((k == "opt" && v == "--") || (k == "target" && v.is_empty())) {
                Value::Null
            } else {
                Value::String(v)
            };
            o.insert(k.clone(), val);
        }
        if let Some(Value::Array(a)) = c.get_mut("rules") {
            a.push(Value::Object(o));
        }
    }
    if let Some(c) = chain {
        chains.push(Value::Object(c));
    }
    Ok(Value::Array(chains))
}
pub static IPTABLES: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
