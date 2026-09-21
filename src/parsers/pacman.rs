use crate::parsers::common::{normalize_key, raw, StaticParser};
use crate::utils::{convert_size_to_int, input_to_str};
use crate::{
    OptionSpec, OutputShape, ParseOptions, ParserCapabilities, ParserDescriptor, ParserOutput,
    ParserTag, Platform, ScocError, UpstreamParser,
};
use chrono::{Local, NaiveDateTime, TimeZone};
use serde_json::{Map, Value};

/// JC drops the trailing ` TZ` abbreviation and reads the rest as local time.
fn local_epoch(text: &str) -> Option<i64> {
    let trimmed = text.get(..text.len().checked_sub(4)?)?;
    let naive = NaiveDateTime::parse_from_str(trimmed, "%a %d %b %Y %I:%M:%S %p").ok()?;
    Some(Local.from_local_datetime(&naive).earliest()?.timestamp())
}
const OPTIONS: [OptionSpec; 1] = [OptionSpec::bool("raw", false, "Return JC raw output")];
const PLATFORMS: [Platform; 1] = [Platform::Linux];
const TAGS: [ParserTag; 1] = [ParserTag::Command];
pub static DESCRIPTOR: ParserDescriptor = ParserDescriptor {
    name: "pacman",
    aliases: &["yay"],
    description: "`pacman` command parser",
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
        standard_name: "pacman",
        standard_version: "1.1",
        streaming_name: None,
        streaming_version: None,
    }),
};
fn flush_mult(entry: &mut Map<String, Value>, key: &mut Option<String>, vals: &mut Vec<Value>) {
    if let Some(k) = key.take() {
        let vals = std::mem::take(vals);
        // JC only stores a multi-line field that collected something.
        if !vals.is_empty() {
            entry.insert(k, Value::Array(vals));
        }
    }
}
fn parse(
    d: &'static ParserDescriptor,
    input: &[u8],
    opts: &ParseOptions,
) -> Result<Value, ScocError> {
    let text = input_to_str(d.name, input)?;
    let r = raw(opts);
    let mut rows = vec![];
    let mut entry = Map::new();
    let mut mkey = None;
    let mut mvals = vec![];
    for line in text.lines().filter(|l| !l.is_empty()) {
        if let Some((k, v)) = line.split_once(" : ") {
            let key = normalize_key(k.trim());
            let val = v.trim();
            if matches!(key.as_str(), "name" | "repository") && entry.len() > 2 {
                flush_mult(&mut entry, &mut mkey, &mut mvals);
                rows.push(Value::Object(std::mem::take(&mut entry)));
                entry.insert(key, Value::String(val.into()));
                continue;
            }
            if matches!(
                key.as_str(),
                "required_by" | "optional_deps" | "backup_files"
            ) {
                flush_mult(&mut entry, &mut mkey, &mut mvals);
                mkey = Some(key);
                if val != "None" {
                    mvals.push(Value::String(val.into()));
                }
                continue;
            }
            flush_mult(&mut entry, &mut mkey, &mut mvals);
            entry.insert(
                key,
                if val == "None" {
                    Value::Null
                } else {
                    Value::String(val.into())
                },
            );
        } else if mkey.is_some() {
            mvals.push(Value::String(line.trim().into()));
        }
    }
    flush_mult(&mut entry, &mut mkey, &mut mvals);
    if !entry.is_empty() {
        rows.push(Value::Object(entry));
    }
    if !r {
        for row in &mut rows {
            let Value::Object(o) = row else { continue };
            let words = |s: &str| {
                s.split_whitespace()
                    .map(|x| Value::String(x.into()))
                    .collect::<Vec<_>>()
            };
            // JC's `_process` keys off the raw value of each field.
            for key in [
                "licenses",
                "groups",
                "provides",
                "depends_on",
                "conflicts_with",
                "replaces",
                "optional_for",
                "required_by",
                "validated_by",
            ] {
                let Some(original) = o.get(key).cloned() else {
                    continue;
                };
                let split_field = !matches!(key, "required_by" | "validated_by");
                let space_split = !matches!(key, "licenses" | "optional_for");
                let two_space = matches!(key, "licenses" | "validated_by");
                let mut value = original.clone();
                if split_field {
                    value = match &original {
                        Value::Null => Value::Array(vec![]),
                        Value::String(s) => Value::Array(words(s)),
                        other => other.clone(),
                    };
                }
                if space_split {
                    if let Value::Array(items) = &original {
                        value = Value::Array(
                            items
                                .iter()
                                .flat_map(|item| match item {
                                    Value::String(s) => words(s),
                                    other => vec![other.clone()],
                                })
                                .collect(),
                        );
                    }
                }
                if two_space {
                    if let Value::String(s) = &original {
                        value =
                            Value::Array(s.split("  ").map(|x| Value::String(x.into())).collect());
                    }
                }
                o.insert(key.into(), value);
            }
            if let Some(Value::Array(a)) = o.get("optional_deps").cloned() {
                let mut n = vec![];
                for v in a {
                    if let Value::String(s) = v {
                        let mut parts = s.splitn(2, ": ");
                        let name = parts.next().unwrap_or("");
                        let desc = parts.next().unwrap_or("");
                        let mut x = Map::new();
                        x.insert("name".into(), Value::String(name.into()));
                        x.insert("description".into(), Value::String(desc.into()));
                        n.push(Value::Object(x));
                    }
                }
                o.insert("optional_deps".into(), Value::Array(n));
            }
            for key in ["build_date", "install_date"] {
                if let Some(Value::String(s)) = o.get(key) {
                    if let Some(epoch) = local_epoch(s) {
                        o.insert(format!("{key}_epoch"), Value::from(epoch));
                    }
                }
            }
            for key in ["download_size", "installed_size"] {
                if let Some(Value::String(s)) = o.get(key) {
                    if let Some(n) = convert_size_to_int(s, false, false) {
                        o.insert(format!("{key}_bytes"), Value::from(n));
                    }
                }
            }
        }
    }
    Ok(Value::Array(rows))
}
pub static PACMAN: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
