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
    name: "netstat",
    aliases: &[],
    description: "`netstat` command parser",
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
        standard_name: "netstat",
        standard_version: "1.16",
        streaming_name: None,
        streaming_version: None,
    }),
};
fn endpoint(s: &str) -> (String, String) {
    if s.starts_with('[') {
        if let Some(pos) = s.rfind("]: ") {
            return (s[..pos + 1].into(), s[pos + 3..].into());
        }
        if let Some(pos) = s.rfind("]: ") {
            return (s[..pos + 1].into(), s[pos + 2..].into());
        }
    }
    if let Some((a, p)) = s.rsplit_once(':') {
        return (a.into(), p.into());
    }
    (s.into(), String::new())
}
fn parse(
    d: &'static ParserDescriptor,
    input: &[u8],
    opts: &ParseOptions,
) -> Result<Value, ScocError> {
    let text = input_to_str(d.name, input)?;
    let r = raw(opts);
    let lines = text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .collect::<Vec<_>>();
    let mut out = vec![];
    if lines
        .iter()
        .any(|l| l.contains("Destination") && l.contains("Gateway") && l.contains("Genmask"))
    {
        let mut start = 0;
        for (i, l) in lines.iter().enumerate() {
            if l.contains("Destination") && l.contains("Gateway") {
                start = i + 1;
                break;
            }
        }
        for line in &lines[start..] {
            let f = line.split_whitespace().collect::<Vec<_>>();
            if f.len() < 8 {
                continue;
            }
            let mut o = Map::new();
            for (k, v) in [
                ("destination", f[0]),
                ("gateway", f[1]),
                ("genmask", f[2]),
                ("route_flags", f[3]),
            ] {
                o.insert(k.into(), Value::String(v.into()));
            }
            for (i, k) in [(4, "metric"), (5, "route_refs"), (6, "use")] {
                o.insert(
                    k.into(),
                    if r {
                        Value::String(f[i].into())
                    } else {
                        to_i64_value(f[i])
                    },
                );
            }
            o.insert("iface".into(), Value::String(f[f.len() - 1].into()));
            o.insert("kind".into(), Value::String("route".into()));
            out.push(Value::Object(o));
        }
        return Ok(Value::Array(out));
    }
    for line in lines {
        let s = line.trim();
        if !(s.starts_with("tcp") || s.starts_with("udp") || s.starts_with("raw")) {
            continue;
        }
        let f = s.split_whitespace().collect::<Vec<_>>();
        if f.len() < 5 {
            continue;
        }
        let mut o = Map::new();
        o.insert("proto".into(), Value::String(f[0].into()));
        o.insert(
            "recv_q".into(),
            if r {
                Value::String(f[1].into())
            } else {
                to_i64_value(f[1])
            },
        );
        o.insert(
            "send_q".into(),
            if r {
                Value::String(f[2].into())
            } else {
                to_i64_value(f[2])
            },
        );
        let (la, lp) = endpoint(f[3]);
        let (fa, fp) = endpoint(f[4]);
        o.insert("local_address".into(), Value::String(la));
        o.insert("local_port".into(), Value::String(lp.clone()));
        o.insert("foreign_address".into(), Value::String(fa));
        o.insert("foreign_port".into(), Value::String(fp.clone()));
        if let Some(state) = f.get(5) {
            o.insert("state".into(), Value::String((*state).into()));
        }
        o.insert("kind".into(), Value::String("network".into()));
        let proto = f[0];
        o.insert(
            "transport_protocol".into(),
            if proto.starts_with("tcp") {
                Value::String("tcp".into())
            } else if proto.starts_with("udp") {
                Value::String("udp".into())
            } else {
                Value::Null
            },
        );
        o.insert(
            "network_protocol".into(),
            Value::String(if proto.ends_with('6') { "ipv6" } else { "ipv4" }.into()),
        );
        if !r {
            if let Ok(n) = lp.parse::<i64>() {
                o.insert("local_port_num".into(), Value::from(n));
            }
            if let Ok(n) = fp.parse::<i64>() {
                o.insert("foreign_port_num".into(), Value::from(n));
            }
        }
        out.push(Value::Object(o));
    }
    Ok(Value::Array(out))
}
pub static NETSTAT: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
