use crate::parsers::common::{raw, StaticParser};
use crate::utils::{convert_size_to_int, input_to_str, sparse_table_parse, to_i64_value};
use crate::{
    OptionSpec, OutputShape, ParseOptions, ParserCapabilities, ParserDescriptor, ParserOutput,
    ParserTag, Platform, ScocError, UpstreamParser,
};
use serde_json::Value;
const OPTIONS: [OptionSpec; 1] = [OptionSpec::bool("raw", false, "Return JC raw output")];
const PLATFORMS: [Platform; 1] = [Platform::Linux];
const TAGS: [ParserTag; 1] = [ParserTag::Command];
pub static DESCRIPTOR: ParserDescriptor = ParserDescriptor {
    name: "lsblk",
    aliases: &[],
    description: "`lsblk` command parser",
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
        standard_name: "lsblk",
        standard_version: "1.10",
        streaming_name: None,
        streaming_version: None,
    }),
};
fn truthy(s: &str) -> bool {
    matches!(s.trim(), "1" | "true" | "yes" | "y" | "*")
}
fn parse(
    d: &'static ParserDescriptor,
    input: &[u8],
    opts: &ParseOptions,
) -> Result<Value, ScocError> {
    let text = input_to_str(d.name, input)?;
    let mut lines = text.lines().collect::<Vec<_>>();
    if lines.is_empty() {
        return Ok(Value::Array(vec![]));
    }
    let header = lines[0].to_ascii_lowercase().replace([':', '-'], "_");
    lines[0] = &text[0..0];
    let body = text.lines().skip(1).collect::<Vec<_>>();
    let mut merged = Vec::with_capacity(body.len() + 1);
    merged.push(header.as_str());
    merged.extend(body);
    let raw_mode = raw(opts);
    let parsed = sparse_table_parse(&merged)?;
    let mut rows: Vec<Value> = vec![];
    for row in parsed {
        let Value::Object(mut o) = row else { continue };
        let Some(namev) = o.get("name").cloned() else {
            continue;
        };
        if namev.is_null() {
            if let Some(Value::String(m)) = o.get("mountpoints") {
                if let Some(Value::Object(last)) = rows.last_mut() {
                    if let Some(Value::Array(a)) = last.get_mut("mountpoints") {
                        a.push(Value::String(m.clone()));
                    }
                }
            }
            continue;
        }
        if let Some(Value::String(name)) = o.get_mut("name") {
            for p in ["`-", "|-", "├─", "└─"] {
                if name.starts_with(p) {
                    *name = name[p.len()..].to_string();
                    break;
                }
            }
        }
        if let Some(v) = o.get("mountpoints").cloned() {
            o.insert(
                "mountpoints".into(),
                match v {
                    Value::String(s) => Value::Array(vec![Value::String(s)]),
                    Value::Null => Value::Array(vec![]),
                    x => x,
                },
            );
        }
        if !raw_mode {
            for k in ["rm", "ro", "rota", "disc_zero", "rand"] {
                if let Some(Value::String(s)) = o.get(k).cloned() {
                    o.insert(k.into(), Value::Bool(truthy(&s)));
                }
            }
            for k in [
                "ra",
                "alignment",
                "min_io",
                "opt_io",
                "phy_sec",
                "log_sec",
                "rq_size",
                "disc_aln",
            ] {
                if let Some(Value::String(s)) = o.get(k).cloned() {
                    o.insert(k.into(), to_i64_value(&s));
                }
            }
        }
        for k in ["size", "disc_gran", "disc_max", "wsame"] {
            if let Some(Value::String(s)) = o.get(k).cloned() {
                if let Some(n) = convert_size_to_int(&s, false, true) {
                    o.insert(format!("{k}_bytes"), Value::from(n));
                }
            }
        }
        rows.push(Value::Object(o));
    }
    Ok(Value::Array(rows))
}
pub static LSBLK: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
