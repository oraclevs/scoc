use crate::parsers::common::{normalize_key, raw, StaticParser};
use crate::utils::{input_to_str, to_f64_value, to_i64_value};
use crate::{
    OptionSpec, OutputShape, ParseOptions, ParserCapabilities, ParserDescriptor, ParserOutput,
    ParserTag, Platform, ScocError, UpstreamParser,
};
use chrono::{NaiveDateTime, TimeZone, Utc};
use serde_json::{Map, Value};
const OPTIONS: [OptionSpec; 1] = [OptionSpec::bool("raw", false, "Return JC raw output")];
const PLATFORMS: [Platform; 1] = [Platform::Linux];
const TAGS: [ParserTag; 1] = [ParserTag::Command];
pub static DESCRIPTOR: ParserDescriptor = ParserDescriptor {
    name: "timedatectl",
    aliases: &[],
    description: "`timedatectl status` command parser",
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
        standard_name: "timedatectl",
        standard_version: "1.8",
        streaming_name: None,
        streaming_version: None,
    }),
};
fn truth(s: &str) -> bool {
    matches!(
        s.to_ascii_lowercase().as_str(),
        "yes" | "true" | "1" | "y" | "*"
    )
}
fn parse(
    d: &'static ParserDescriptor,
    input: &[u8],
    opts: &ParseOptions,
) -> Result<Value, ScocError> {
    let text = input_to_str(d.name, input)?;
    let r = raw(opts);
    let valid = [
        "local time",
        "universal time",
        "rtc time",
        "time zone",
        "ntp enabled",
        "ntp synchronized",
        "rtc in local tz",
        "dst active",
        "system clock synchronized",
        "ntp service",
        "systemd-timesyncd.service active",
        "server",
        "poll interval",
        "leap",
        "version",
        "stratum",
        "reference",
        "precision",
        "root distance",
        "offset",
        "delay",
        "jitter",
        "packet count",
        "frequency",
    ];
    let mut o = Map::new();
    for line in text.lines() {
        let Some((k, v)) = line.split_once(':') else {
            continue;
        };
        let kl = k.trim().to_ascii_lowercase();
        if !valid.contains(&kl.as_str()) {
            continue;
        }
        o.insert(normalize_key(&kl), Value::String(v.trim().into()));
    }
    if !r {
        for k in [
            "ntp_enabled",
            "ntp_synchronized",
            "rtc_in_local_tz",
            "dst_active",
            "system_clock_synchronized",
            "systemd-timesyncd.service_active",
        ] {
            if let Some(Value::String(s)) = o.get(k).cloned() {
                o.insert(k.into(), Value::Bool(truth(&s)));
            }
        }
        for k in ["version", "stratum", "packet_count"] {
            if let Some(Value::String(s)) = o.get(k).cloned() {
                o.insert(k.into(), to_i64_value(&s));
            }
        }
        for k in ["offset", "delay", "jitter", "frequency"] {
            if let Some(Value::String(s)) = o.get(k).cloned() {
                let unit = if k == "frequency" {
                    s.chars()
                        .rev()
                        .take(3)
                        .collect::<String>()
                        .chars()
                        .rev()
                        .collect::<String>()
                } else {
                    s.chars()
                        .rev()
                        .take(2)
                        .collect::<String>()
                        .chars()
                        .rev()
                        .collect::<String>()
                };
                o.insert(format!("{k}_unit"), Value::String(unit));
                o.insert(k.into(), to_f64_value(&s));
            }
        }
        if let Some(Value::String(s)) = o.get("universal_time") {
            let trimmed = s
                .split_whitespace()
                .skip(1)
                .take(2)
                .collect::<Vec<_>>()
                .join(" ");
            if let Ok(dt) = NaiveDateTime::parse_from_str(&trimmed, "%Y-%m-%d %H:%M:%S") {
                o.insert(
                    "epoch_utc".into(),
                    Value::from(Utc.from_utc_datetime(&dt).timestamp()),
                );
            }
        }
    }
    Ok(Value::Object(o))
}
pub static TIMEDATECTL: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
