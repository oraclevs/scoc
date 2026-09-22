use crate::parsers::common::{raw, StaticParser};
use crate::utils::{input_to_str, to_i64_value};
use crate::{
    OptionSpec, OutputShape, ParseOptions, ParserCapabilities, ParserDescriptor, ParserOutput,
    ParserTag, Platform, ScocError, UpstreamParser,
};
use regex::Regex;
use serde_json::{Map, Value};
const OPTIONS: [OptionSpec; 1] = [OptionSpec::bool("raw", false, "Return JC raw output")];
const PLATFORMS: [Platform; 2] = [Platform::Linux, Platform::MacOs];
const TAGS: [ParserTag; 1] = [ParserTag::Command];
pub static DESCRIPTOR: ParserDescriptor = ParserDescriptor {
    name: "ifconfig",
    aliases: &[],
    description: "`ifconfig` command parser",
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
        standard_name: "ifconfig",
        standard_version: "2.5",
        streaming_name: None,
        streaming_version: None,
    }),
};
fn finalize(
    mut o: Map<String, Value>,
    raw_mode: bool,
    ipv4: Vec<Value>,
    ipv6: Vec<Value>,
) -> Value {
    if !ipv4.is_empty() {
        o.insert("ipv4".into(), Value::Array(ipv4));
    }
    if !ipv6.is_empty() {
        o.insert("ipv6".into(), Value::Array(ipv6));
    }
    if !raw_mode {
        for k in [
            "flags",
            "mtu",
            "ipv6_mask",
            "rx_packets",
            "rx_bytes",
            "rx_errors",
            "rx_dropped",
            "rx_overruns",
            "rx_frame",
            "tx_packets",
            "tx_bytes",
            "tx_errors",
            "tx_dropped",
            "tx_overruns",
            "tx_carrier",
            "tx_collisions",
            "metric",
        ] {
            if let Some(Value::String(s)) = o.get(k).cloned() {
                o.insert(k.into(), to_i64_value(&s));
            }
        }
        if let Some(Value::String(s)) = o.get("state").cloned() {
            o.insert(
                "state".into(),
                Value::Array(s.split(',').map(|x| Value::String(x.into())).collect()),
            );
        }
        if let Some(Value::Array(a)) = o.get_mut("ipv6") {
            for v in a {
                if let Value::Object(x) = v {
                    if let Some(Value::String(s)) = x.get("mask").cloned() {
                        x.insert("mask".into(), to_i64_value(&s));
                    }
                }
            }
        }
    }
    Value::Object(o)
}
fn parse(
    d: &'static ParserDescriptor,
    input: &[u8],
    opts: &ParseOptions,
) -> Result<Value, ScocError> {
    let text = input_to_str(d.name, input)?;
    let raw_mode = raw(opts);
    let linux=Regex::new(r"^([A-Za-z0-9:._-]+):?\s+(?:flags=(\d+)<([^>]*)>|Link encap:([^ ]+(?: [^ ]+)?))(?:.*?mtu\s+(\d+))?").map_err(|e|ScocError::parse(d.name,None,e.to_string()))?;
    let bsd = Regex::new(r"^([A-Za-z0-9:._-]+): flags=(\d+)<([^>]*)> mtu (\d+)")
        .map_err(|e| ScocError::parse(d.name, None, e.to_string()))?;
    let mut out = vec![];
    let mut cur = Map::new();
    let mut v4 = vec![];
    let mut v6 = vec![];
    for line in text.lines().chain(std::iter::once("__END__")) {
        if line == "__END__"
            || (!line.starts_with(char::is_whitespace) && (!line.trim().is_empty()))
        {
            if !cur.is_empty() {
                out.push(finalize(
                    std::mem::take(&mut cur),
                    raw_mode,
                    std::mem::take(&mut v4),
                    std::mem::take(&mut v6),
                ));
            }
            if line == "__END__" {
                break;
            }
            if let Some(c) = bsd.captures(line) {
                cur.insert("name".into(), Value::String(c[1].into()));
                cur.insert("flags".into(), Value::String(c[2].into()));
                cur.insert("state".into(), Value::String(c[3].into()));
                cur.insert("mtu".into(), Value::String(c[4].into()));
            } else if let Some(c) = linux.captures(line) {
                cur.insert("name".into(), Value::String(c[1].into()));
                if let Some(x) = c.get(2) {
                    cur.insert("flags".into(), Value::String(x.as_str().into()));
                }
                if let Some(x) = c.get(3) {
                    cur.insert("state".into(), Value::String(x.as_str().into()));
                }
                if let Some(x) = c.get(4) {
                    cur.insert("type".into(), Value::String(x.as_str().into()));
                }
                if let Some(x) = c.get(5) {
                    cur.insert("mtu".into(), Value::String(x.as_str().into()));
                }
            }
            continue;
        }
        let s = line.trim();
        if s.starts_with("inet ") || s.starts_with("inet addr:") {
            let clean = s
                .replace("addr:", "")
                .replace("Bcast:", "")
                .replace("Mask:", "");
            let f = clean.split_whitespace().collect::<Vec<_>>();
            let address = f.get(1).copied().unwrap_or("");
            let mut mask = None;
            let mut broad = None;
            for i in 0..f.len() {
                if f[i] == "netmask" {
                    mask = f.get(i + 1).copied();
                }
                if f[i] == "broadcast" {
                    broad = f.get(i + 1).copied();
                }
            }
            if mask.is_none() && f.len() >= 4 {
                broad = f.get(2).copied();
                mask = f.get(3).copied();
            }
            cur.insert("ipv4_addr".into(), Value::String(address.into()));
            cur.insert(
                "ipv4_mask".into(),
                mask.map(|x| Value::String(x.into())).unwrap_or(Value::Null),
            );
            cur.insert(
                "ipv4_bcast".into(),
                broad
                    .map(|x| Value::String(x.into()))
                    .unwrap_or(Value::Null),
            );
            let mut x = Map::new();
            x.insert("address".into(), Value::String(address.into()));
            x.insert(
                "mask".into(),
                mask.map(|x| Value::String(x.into())).unwrap_or(Value::Null),
            );
            x.insert(
                "broadcast".into(),
                broad
                    .map(|x| Value::String(x.into()))
                    .unwrap_or(Value::Null),
            );
            v4.push(Value::Object(x));
            continue;
        }
        if s.starts_with("inet6 ") {
            let f = s.split_whitespace().collect::<Vec<_>>();
            let addr = f.get(1).copied().unwrap_or("");
            let mut mask = None;
            let mut scope = None;
            for i in 0..f.len() {
                if f[i] == "prefixlen" {
                    mask = f.get(i + 1).copied();
                }
                if f[i] == "scopeid" {
                    scope = f.get(i + 1).copied();
                }
            }
            if mask.is_none() {
                if let Some(x) = f.iter().find(|x| x.starts_with("prefixlen")) {
                    mask = Some(x.trim_start_matches("prefixlen"));
                }
            }
            cur.insert("ipv6_addr".into(), Value::String(addr.into()));
            cur.insert(
                "ipv6_mask".into(),
                mask.map(|x| Value::String(x.into())).unwrap_or(Value::Null),
            );
            cur.insert(
                "ipv6_scope".into(),
                scope
                    .map(|x| Value::String(x.into()))
                    .unwrap_or(Value::Null),
            );
            let mut x = Map::new();
            x.insert("address".into(), Value::String(addr.into()));
            x.insert(
                "scope_id".into(),
                scope
                    .map(|x| Value::String(x.into()))
                    .unwrap_or(Value::Null),
            );
            x.insert(
                "mask".into(),
                mask.map(|x| Value::String(x.into())).unwrap_or(Value::Null),
            );
            v6.push(Value::Object(x));
            continue;
        }
        if s.starts_with("ether ") || s.starts_with("HWaddr ") {
            let mac = s.split_whitespace().nth(1).unwrap_or("");
            cur.insert("mac_addr".into(), Value::String(mac.into()));
            continue;
        }
        let normalized = s.replace(':', " ");
        let f = normalized.split_whitespace().collect::<Vec<_>>();
        for (i, k) in [
            ("RX packets", "rx_packets"),
            ("TX packets", "tx_packets"),
            ("RX bytes", "rx_bytes"),
            ("TX bytes", "tx_bytes"),
        ] {
            if let Some(pos) = s.find(i) {
                let rest = &s[pos + i.len()..];
                let val = rest
                    .trim_start_matches([' ', ':'])
                    .split_whitespace()
                    .next()
                    .unwrap_or("");
                cur.insert(k.into(), Value::String(val.into()));
            }
        }
        if f.first() == Some(&"status") {
            if let Some(v) = f.get(1) {
                cur.insert("status".into(), Value::String((*v).into()));
            }
        }
    }
    Ok(Value::Array(out))
}
pub static IFCONFIG: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
