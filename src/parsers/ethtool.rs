use crate::parsers::common::{normalize_key, raw, StaticParser};
use crate::utils::{convert_size_to_int, input_to_str, to_f64_value};
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
    name: "ethtool",
    aliases: &[],
    description: "`ethtool` command parser",
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
        standard_name: "ethtool",
        standard_version: "1.1",
        streaming_name: None,
        streaming_version: None,
    }),
};
fn truthy(v: &str) -> bool {
    matches!(
        v.to_ascii_lowercase().as_str(),
        "yes" | "on" | "true" | "1" | "y" | "*"
    )
}
fn parse_default(text: &str) -> Result<Map<String, Value>, ScocError> {
    let mut out = Map::new();
    let mut mode = "";
    let mut lists: std::collections::HashMap<&str, Vec<Value>> = std::collections::HashMap::new();
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        if line.starts_with("Settings for ") {
            out.insert(
                "name".into(),
                Value::String(
                    line.split_whitespace()
                        .nth(2)
                        .unwrap_or("")
                        .trim_end_matches(':')
                        .into(),
                ),
            );
            continue;
        }
        let data = line.replace('\t', "    ");
        if !data.starts_with("         ") {
            mode = "";
        }
        for (needle, key) in [
            ("Supported ports:", "supported_ports"),
            ("Supported link modes:", "supported_link_modes"),
            ("Supported FEC modes:", "supported_fec_modes"),
            ("Advertised link modes:", "advertised_link_modes"),
            (
                "Link partner advertised link modes:",
                "link_partner_advertised_link_modes",
            ),
            ("Advertised FEC modes:", "advertised_fec_modes"),
            ("Current message level:", "current_message_level"),
        ] {
            if line.contains(needle) && !line.contains("Not reported") {
                let val = line.split_once(':').map(|(_, v)| v.trim()).unwrap_or("");
                lists.entry(key).or_default().extend(
                    val.trim_matches(|c| c == '[' || c == ']')
                        .split_whitespace()
                        .map(|v| Value::String(v.into())),
                );
                mode = key;
                break;
            }
        }
        if !mode.is_empty() && data.starts_with("         ") {
            lists
                .entry(mode)
                .or_default()
                .extend(line.split_whitespace().map(|v| Value::String(v.into())));
            continue;
        }
        if let Some((k, v)) = line.split_once(':') {
            out.insert(normalize_key(k), Value::String(v.trim().into()));
        }
    }
    for (k, v) in lists {
        if !v.is_empty() {
            out.insert(k.into(), Value::Array(v));
        }
    }
    Ok(out)
}
fn parse_module(text: &str) -> Map<String, Value> {
    let mut out = Map::new();
    let mut last = String::new();
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        if let Some((k, v)) = line.trim().split_once(':') {
            let key = normalize_key(k);
            let val = Value::String(v.trim().into());
            if key == last {
                match out.get_mut(&key) {
                    Some(Value::Array(a)) => a.push(val),
                    Some(existing) => {
                        let first = existing.take();
                        *existing = Value::Array(vec![first, val]);
                    }
                    None => {
                        out.insert(key.clone(), Value::Array(vec![val]));
                    }
                }
            } else {
                out.insert(key.clone(), val);
            }
            last = key;
        }
    }
    out
}
fn parse(
    descriptor: &'static ParserDescriptor,
    input: &[u8],
    options: &ParseOptions,
) -> Result<Value, ScocError> {
    let text = input_to_str(descriptor.name, input)?;
    let mut out = if text.trim_start().starts_with("Settings for ") {
        parse_default(text)?
    } else if text.trim_start().starts_with("Identifier ") {
        parse_module(text)
    } else {
        Map::new()
    };
    if !raw(options) {
        if let Some(Value::String(v)) = out.get("speed").cloned() {
            out.insert(
                "speed_bps".into(),
                convert_size_to_int(&v, false, false).map_or(Value::Null, Value::from),
            );
        }
        for k in [
            "supports_auto_negotiation",
            "advertised_auto_negotiation",
            "auto_negotiation",
            "link_detected",
            "advertised_pause_frame_use",
        ] {
            if let Some(Value::String(v)) = out.get(k).cloned() {
                out.insert(k.into(), Value::Bool(truthy(&v)));
            }
        }
        let deg = Regex::new(r"^(.*?) degrees C / (.*?) degrees F$").unwrap();
        let pow = Regex::new(r"^(.*?) mW / (.*?) dBm$").unwrap();
        for (k, v) in out.clone() {
            let Value::String(s) = v else {
                continue;
            };
            if let Some(c) = deg.captures(&s) {
                out.remove(&k);
                out.insert(format!("{k}_celsius"), to_f64_value(&c[1]));
                out.insert(format!("{k}_farenheit"), to_f64_value(&c[2]));
            } else if let Some(c) = pow.captures(&s) {
                out.remove(&k);
                out.insert(format!("{k}_mw"), to_f64_value(&c[1]));
                out.insert(format!("{k}_dbm"), to_f64_value(&c[2]));
            } else if s.ends_with(" V") {
                out.remove(&k);
                out.insert(format!("{k}_v"), to_f64_value(&s));
            } else if s.ends_with(" mA") {
                out.remove(&k);
                out.insert(format!("{k}_ma"), to_f64_value(&s));
            }
        }
    }
    Ok(Value::Object(out))
}
pub static ETHTOOL: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
