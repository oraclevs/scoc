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
    name: "lspci",
    aliases: &[],
    description: "`lspci -mmv` command parser",
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
        standard_name: "lspci",
        standard_version: "1.1",
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
    let bracket = Regex::new(r"^(.*) \[([0-9a-fA-F]{4})\]$")
        .map_err(|e| ScocError::parse(d.name, None, e.to_string()))?;
    let mut out = vec![];
    let mut o = Map::new();
    for line in text.lines().chain(std::iter::once("Slot:")) {
        if line.starts_with("Slot:") {
            if !o.is_empty() {
                out.push(Value::Object(std::mem::take(&mut o)));
            }
            let Some(slot) = line.split_whitespace().nth(1) else {
                continue;
            };
            o.insert("slot".into(), Value::String(slot.into()));
            let (left, fun) = slot.rsplit_once('.').unwrap_or((slot, "0"));
            let parts = left.split(':').collect::<Vec<_>>();
            let (domain, bus, dev) = if parts.len() == 3 {
                (parts[0], parts[1], parts[2])
            } else if parts.len() == 2 {
                ("00", parts[0], parts[1])
            } else {
                ("00", "00", left)
            };
            for (k, v) in [
                ("domain", domain),
                ("bus", bus),
                ("dev", dev),
                ("function", fun),
            ] {
                o.insert(k.into(), Value::String(v.into()));
                if !r {
                    if let Ok(n) = i64::from_str_radix(v, 16) {
                        o.insert(format!("{k}_int"), Value::from(n));
                    }
                }
            }
            continue;
        }
        let Some((key, val)) = line.split_once(char::is_whitespace) else {
            continue;
        };
        let key = key.trim_end_matches(':').to_ascii_lowercase();
        let val = val.trim();
        if val.len() == 4 && val.chars().all(|c| c.is_ascii_hexdigit()) {
            o.insert(format!("{key}_id"), Value::String(val.into()));
            if !r {
                if let Ok(n) = i64::from_str_radix(val, 16) {
                    o.insert(format!("{key}_id_int"), Value::from(n));
                }
            }
        } else if let Some(c) = bracket.captures(val) {
            o.insert(key.clone(), Value::String(c[1].into()));
            o.insert(format!("{key}_id"), Value::String(c[2].into()));
            if !r {
                if let Ok(n) = i64::from_str_radix(&c[2], 16) {
                    o.insert(format!("{key}_id_int"), Value::from(n));
                }
            }
        } else {
            if !r && key == "progif" {
                if let Ok(n) = i64::from_str_radix(val, 16) {
                    o.insert("progif_int".into(), Value::from(n));
                }
            }
            o.insert(key, Value::String(val.into()));
        }
    }
    Ok(Value::Array(out))
}
pub static LSPCI: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
