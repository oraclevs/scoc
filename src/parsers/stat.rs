use crate::parsers::common::{raw, shell_words, StaticParser};
use crate::utils::{input_to_str, to_i64_value};
use crate::{
    OptionSpec, OutputShape, ParseOptions, ParserCapabilities, ParserDescriptor, ParserOutput,
    ParserTag, Platform, ScocError, UpstreamParser,
};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use serde_json::{Map, Value};
const OPTIONS: [OptionSpec; 1] = [OptionSpec::bool("raw", false, "Return JC raw output")];
const PLATFORMS: [Platform; 2] = [Platform::Linux, Platform::MacOs];
const TAGS: [ParserTag; 1] = [ParserTag::Command];
pub static DESCRIPTOR: ParserDescriptor = ParserDescriptor {
    name: "stat",
    aliases: &[],
    description: "`stat` command parser",
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
        standard_name: "stat",
        standard_version: "1.13",
        streaming_name: Some("stat-s"),
        streaming_version: Some("1.1"),
    }),
};
fn add_time(o: &mut Map<String, Value>, key: &str, raw_mode: bool) {
    if raw_mode {
        return;
    }
    let Some(Value::String(s)) = o.get(key).cloned() else {
        return;
    };
    if s == "-" {
        o.insert(key.into(), Value::Null);
        o.insert(format!("{key}_epoch"), Value::Null);
        o.insert(format!("{key}_epoch_utc"), Value::Null);
        return;
    }
    let mut naive = None;
    let mut utc = None;
    if let Ok(dt) = DateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S%.f %z") {
        naive = Local
            .from_local_datetime(&dt.naive_local())
            .single()
            .map(|x| x.timestamp());
        if dt.offset().local_minus_utc() == 0 {
            utc = Some(dt.with_timezone(&Utc).timestamp());
        }
    } else if let Ok(dt) = NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S%.f") {
        naive = Local
            .from_local_datetime(&dt)
            .single()
            .map(|x| x.timestamp());
    }
    o.insert(
        format!("{key}_epoch"),
        naive.map(Value::from).unwrap_or(Value::Null),
    );
    o.insert(
        format!("{key}_epoch_utc"),
        utc.map(Value::from).unwrap_or(Value::Null),
    );
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
        .filter(|x| !x.trim().is_empty())
        .collect::<Vec<_>>();
    if lines.is_empty() {
        return Ok(Value::Array(vec![]));
    }
    let mut rows = vec![];
    if lines[0].starts_with("  File: ") {
        let mut o = Map::new();
        for line in lines {
            if line.find("File:") == Some(2) {
                if !o.is_empty() {
                    rows.push(Value::Object(std::mem::take(&mut o)));
                }
                let value = line.split_once("File:").map(|x| x.1.trim()).unwrap_or("");
                let clean = value.trim_matches(|c| c == '‘' || c == '’' || c == '\'' || c == '"');
                if let Some((f, l)) = clean.split_once(" -> ") {
                    o.insert(
                        "file".into(),
                        Value::String(f.trim_matches(|c| c == '‘' || c == '’' || c == '\'').into()),
                    );
                    o.insert(
                        "link_to".into(),
                        Value::String(l.trim_matches(|c| c == '‘' || c == '’' || c == '\'').into()),
                    );
                } else {
                    o.insert("file".into(), Value::String(clean.into()));
                }
                continue;
            }
            if line.starts_with("  Size:") {
                let f = line.split_whitespace().collect::<Vec<_>>();
                if f.len() >= 8 {
                    o.insert("size".into(), Value::String(f[1].into()));
                    o.insert("blocks".into(), Value::String(f[3].into()));
                    o.insert("io_blocks".into(), Value::String(f[6].into()));
                    o.insert("type".into(), Value::String(f[7..].join(" ")));
                }
                continue;
            }
            if line.starts_with("Device:") {
                let f = line.split_whitespace().collect::<Vec<_>>();
                if f.len() >= 6 {
                    o.insert("device".into(), Value::String(f[1].into()));
                    o.insert("inode".into(), Value::String(f[3].into()));
                    o.insert("links".into(), Value::String(f[5].into()));
                }
                continue;
            }
            if line.starts_with("Access: (") {
                let x = line.replace(['(', ')', '/'], " ");
                let f = x.split_whitespace().collect::<Vec<_>>();
                if f.len() >= 9 {
                    for (k, v) in [
                        ("access", f[1]),
                        ("flags", f[2]),
                        ("uid", f[4]),
                        ("user", f[5]),
                        ("gid", f[7]),
                        ("group", f[8]),
                    ] {
                        o.insert(k.into(), Value::String(v.into()));
                    }
                }
                continue;
            }
            for (prefix, key) in [
                ("Access: 2", "access_time"),
                ("Modify:", "modify_time"),
                ("Change:", "change_time"),
                (" Birth:", "birth_time"),
            ] {
                if line.starts_with(prefix) {
                    if let Some((_, v)) = line.split_once(':') {
                        o.insert(key.into(), Value::String(v.trim().into()));
                    }
                    break;
                }
            }
        }
        if !o.is_empty() {
            rows.push(Value::Object(o));
        }
    } else {
        for (n, line) in lines.iter().enumerate() {
            let f = shell_words(d.name, line)?;
            if f.len() < 16 {
                return Err(ScocError::parse(
                    d.name,
                    Some(n + 1),
                    "invalid BSD stat output",
                ));
            }
            let mut o = Map::new();
            for (k, v) in [
                ("unix_device", &f[0]),
                ("inode", &f[1]),
                ("flags", &f[2]),
                ("links", &f[3]),
                ("user", &f[4]),
                ("group", &f[5]),
                ("rdev", &f[6]),
                ("size", &f[7]),
                ("access_time", &f[8]),
                ("modify_time", &f[9]),
                ("change_time", &f[10]),
                ("birth_time", &f[11]),
                ("block_size", &f[12]),
                ("blocks", &f[13]),
                ("unix_flags", &f[14]),
            ] {
                o.insert(k.into(), Value::String(v.clone()));
            }
            o.insert("file".into(), Value::String(f[15..].join(" ")));
            rows.push(Value::Object(o));
        }
    }
    if !r {
        for row in &mut rows {
            let Value::Object(o) = row else { continue };
            for k in [
                "size",
                "blocks",
                "io_blocks",
                "inode",
                "links",
                "uid",
                "gid",
                "unix_device",
                "rdev",
                "block_size",
            ] {
                if let Some(Value::String(s)) = o.get(k).cloned() {
                    o.insert(k.into(), to_i64_value(&s));
                }
            }
            for k in ["access_time", "modify_time", "change_time", "birth_time"] {
                add_time(o, k, false);
            }
        }
    }
    Ok(Value::Array(rows))
}
pub static STAT: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
