use crate::parsers::common::{raw, StaticParser};
use crate::utils::{input_to_str, to_i64_value};
use crate::{
    OptionSpec, OutputShape, ParseOptions, ParserCapabilities, ParserDescriptor, ParserOutput,
    ParserTag, Platform, ScocError, UpstreamParser,
};
use chrono::{DateTime, Local, TimeZone, Utc};
use regex::Regex;
use serde_json::{Map, Value};
const OPTIONS: [OptionSpec; 1] = [OptionSpec::bool("raw", false, "Return JC raw output")];
const PLATFORMS: [Platform; 2] = [Platform::Linux, Platform::MacOs];
const TAGS: [ParserTag; 1] = [ParserTag::Command];
pub static DESCRIPTOR: ParserDescriptor = ParserDescriptor {
    name: "git-log",
    aliases: &["git-log-s"],
    description: "`git log` command parser",
    parser_version: "0.1.0",
    platforms: &PLATFORMS,
    tags: &TAGS,
    output: ParserOutput {
        normalized: OutputShape::Table,
        raw: Some(OutputShape::Table),
        stream_item: Some(OutputShape::Record),
    },
    capabilities: ParserCapabilities {
        raw: true,
        streaming: true,
        ignore_errors: false,
    },
    options: &OPTIONS,
    upstream: Some(UpstreamParser {
        standard_name: "git-log",
        standard_version: "1.7",
        streaming_name: Some("git-log-s"),
        streaming_version: Some("1.4"),
    }),
};
fn parse_ne(line: &str) -> (Value, Value) {
    let v = line.rsplitn(2, char::is_whitespace).collect::<Vec<_>>();
    if v.len() == 2 && v[0].starts_with('<') && v[0].ends_with('>') {
        (
            Value::String(v[1].trim().into()),
            Value::String(v[0][1..v[0].len() - 1].into()),
        )
    } else {
        (Value::String(line.trim().into()), Value::Null)
    }
}
fn finish(
    mut o: Map<String, Value>,
    msg: &mut Vec<String>,
    files: &mut Vec<Value>,
    stats: &mut Vec<Value>,
    patch: &mut Vec<String>,
    raw_mode: bool,
) -> Value {
    if !msg.is_empty() {
        o.insert(
            "message".into(),
            Value::String(std::mem::take(msg).join("\n")),
        );
    }
    if !files.is_empty() {
        let st = o
            .entry("stats")
            .or_insert_with(|| Value::Object(Map::new()));
        if let Value::Object(s) = st {
            s.insert("files".into(), Value::Array(std::mem::take(files)));
        }
    }
    if !stats.is_empty() {
        let st = o
            .entry("stats")
            .or_insert_with(|| Value::Object(Map::new()));
        if let Value::Object(s) = st {
            s.insert("file_stats".into(), Value::Array(std::mem::take(stats)));
        }
    }
    if !patch.is_empty() {
        o.insert(
            "patch".into(),
            Value::String(std::mem::take(patch).join("\n")),
        );
    }
    if !raw_mode {
        if let Some(Value::String(date)) = o.get("date").cloned() {
            if let Ok(dt) = DateTime::parse_from_str(&date, "%a %b %e %H:%M:%S %Y %z") {
                let naive = Local
                    .from_local_datetime(&dt.naive_local())
                    .single()
                    .map(|x| x.timestamp());
                let utc = (dt.offset().local_minus_utc() == 0)
                    .then(|| dt.with_timezone(&Utc).timestamp());
                o.insert(
                    "epoch".into(),
                    naive.map(Value::from).unwrap_or(Value::Null),
                );
                o.insert(
                    "epoch_utc".into(),
                    utc.map(Value::from).unwrap_or(Value::Null),
                );
            }
        }
        if let Some(Value::Object(s)) = o.get_mut("stats") {
            for k in ["files_changed", "insertions", "deletions"] {
                if let Some(Value::String(v)) = s.get(k).cloned() {
                    s.insert(k.into(), to_i64_value(&v));
                }
            }
            if let Some(Value::Array(a)) = s.get_mut("file_stats") {
                for x in a {
                    if let Value::Object(x) = x {
                        for k in ["lines_changed", "insertions", "deletions"] {
                            if let Some(Value::String(v)) = x.get(k).cloned() {
                                x.insert(k.into(), to_i64_value(&v));
                            }
                        }
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
    let hash =
        Regex::new(r"^[0-9a-f]{40}$").map_err(|e| ScocError::parse(d.name, None, e.to_string()))?;
    let num = Regex::new(r"^(\d+|-)\t(\d+|-)\t(.+)$")
        .map_err(|e| ScocError::parse(d.name, None, e.to_string()))?;
    let changes = Regex::new(
        r"(\d+) files? changed(?:, (\d+) insertions?\(\+\))?(?:, (\d+) deletions?\(-\))?",
    )
    .map_err(|e| ScocError::parse(d.name, None, e.to_string()))?;
    let mut out = vec![];
    let mut o = Map::new();
    let mut msg = vec![];
    let mut files = vec![];
    let mut fstats = vec![];
    let mut patch = vec![];
    let mut in_patch = false;
    for line in text.lines() {
        let first = line.split_whitespace().next().unwrap_or("");
        let new_commit =
            line.starts_with("commit ") || (hash.is_match(first) && !line.starts_with(' '));
        if new_commit {
            if !o.is_empty() {
                out.push(finish(
                    std::mem::take(&mut o),
                    &mut msg,
                    &mut files,
                    &mut fstats,
                    &mut patch,
                    raw_mode,
                ));
            }
            in_patch = false;
            if let Some(rest) = line.strip_prefix("commit ") {
                o.insert("commit".into(), Value::String(rest.trim().into()));
            } else {
                let mut sp = line.splitn(2, char::is_whitespace);
                o.insert(
                    "commit".into(),
                    Value::String(sp.next().unwrap_or("").into()),
                );
                o.insert(
                    "message".into(),
                    Value::String(sp.next().unwrap_or("").trim().into()),
                );
            }
            continue;
        }
        if line.starts_with("diff --git ")
            || line.starts_with("diff --cc ")
            || line.starts_with("diff --combined ")
        {
            in_patch = true;
        }
        if in_patch {
            if !line.is_empty() {
                patch.push(line.into());
            }
            continue;
        }
        if let Some(v) = line.strip_prefix("Author: ") {
            let (n, e) = parse_ne(v);
            o.insert("author".into(), n);
            o.insert("author_email".into(), e);
            continue;
        }
        if let Some(v) = line.strip_prefix("Commit: ") {
            let (n, e) = parse_ne(v);
            o.insert("commit_by".into(), n);
            o.insert("commit_by_email".into(), e);
            continue;
        }
        if let Some(v) = line.strip_prefix("Date:") {
            o.insert("date".into(), Value::String(v.trim().into()));
            continue;
        }
        if let Some(v) = line.strip_prefix("AuthorDate:") {
            o.insert("date".into(), Value::String(v.trim().into()));
            continue;
        }
        if let Some(v) = line.strip_prefix("CommitDate:") {
            o.insert("commit_by_date".into(), Value::String(v.trim().into()));
            continue;
        }
        if let Some(v) = line.strip_prefix("Merge: ") {
            o.insert("merge".into(), Value::String(v.into()));
            continue;
        }
        if let Some(c) = num.captures(line) {
            let mut s = Map::new();
            s.insert("name".into(), Value::String(c[3].into()));
            s.insert("insertions".into(), Value::String(c[1].into()));
            s.insert("deletions".into(), Value::String(c[2].into()));
            files.push(Value::String(c[3].into()));
            fstats.push(Value::Object(s));
            o.entry("stats")
                .or_insert_with(|| Value::Object(Map::new()));
            continue;
        }
        if line.starts_with("    ") {
            msg.push(line.trim().into());
            continue;
        }
        if line.starts_with(' ') && line.contains('|') {
            let mut sp = line.split('|');
            let name = sp.next().unwrap_or("").trim();
            let cnt = sp
                .next()
                .unwrap_or("")
                .split_whitespace()
                .next()
                .unwrap_or("0");
            let mut s = Map::new();
            s.insert("name".into(), Value::String(name.into()));
            s.insert("lines_changed".into(), Value::String(cnt.into()));
            files.push(Value::String(name.into()));
            fstats.push(Value::Object(s));
            continue;
        }
        if let Some(c) = changes.captures(line.trim()) {
            let mut s = Map::new();
            s.insert("files_changed".into(), Value::String(c[1].into()));
            s.insert(
                "insertions".into(),
                Value::String(c.get(2).map(|m| m.as_str()).unwrap_or("0").into()),
            );
            s.insert(
                "deletions".into(),
                Value::String(c.get(3).map(|m| m.as_str()).unwrap_or("0").into()),
            );
            o.insert("stats".into(), Value::Object(s));
        }
    }
    if !o.is_empty() {
        out.push(finish(
            o,
            &mut msg,
            &mut files,
            &mut fstats,
            &mut patch,
            raw_mode,
        ));
    }
    Ok(Value::Array(out))
}
pub static GIT_LOG: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
