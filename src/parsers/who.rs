use chrono::{Local, NaiveDateTime, TimeZone};
use regex::Regex;
use serde_json::{Map, Value};
use std::sync::OnceLock;

use crate::parsers::common::{raw, StaticParser};
use crate::utils::{input_to_str, to_i64_value};
use crate::{
    OptionSpec, OutputShape, ParseOptions, ParserCapabilities, ParserDescriptor, ParserOutput,
    ParserTag, Platform, ScocError, UpstreamParser,
};

const OPTIONS: [OptionSpec; 1] = [OptionSpec::bool("raw", false, "Return JC raw output")];
const PLATFORMS: [Platform; 2] = [Platform::Linux, Platform::MacOs];
const TAGS: [ParserTag; 1] = [ParserTag::Command];

pub static DESCRIPTOR: ParserDescriptor = ParserDescriptor {
    name: "who",
    aliases: &[],
    description: "`who` command parser",
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
        standard_name: "who",
        standard_version: "1.9",
        streaming_name: None,
        streaming_version: None,
    }),
};

fn mac_month_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^[JFMASOND][aepuco][nbrynlgptvc]$").expect("valid month regex"))
}

fn who_epoch(text: &str) -> Option<i64> {
    for format in ["%Y-%m-%d %H:%M", "%Y-%m-%d %H:%M:%S"] {
        if let Ok(value) = NaiveDateTime::parse_from_str(text, format) {
            if let Some(local) = Local.from_local_datetime(&value).single() {
                return Some(local.timestamp());
            }
        }
    }
    None
}

fn finish_row(mut row: Map<String, Value>, raw_mode: bool) -> Value {
    if !raw_mode {
        if let Some(Value::String(pid)) = row.get("pid").cloned() {
            row.insert("pid".into(), to_i64_value(&pid));
        }
        if let Some(Value::String(time)) = row.get("time").cloned() {
            row.insert(
                "epoch".into(),
                who_epoch(&time).map_or(Value::Null, Value::from),
            );
        }
    }
    Value::Object(row)
}

fn parse(
    descriptor: &'static ParserDescriptor,
    input: &[u8],
    options: &ParseOptions,
) -> Result<Value, ScocError> {
    let text = input_to_str(descriptor.name, input)?;
    let raw_mode = raw(options);
    let mut rows = Vec::new();
    for (line_number, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let mut fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() >= 3
            && (fields[..3].join("") == "NAMELINETIME" || fields[..3].join("") == "USERLINEWHEN")
        {
            continue;
        }
        if fields.is_empty() {
            continue;
        }
        let mut row = Map::new();
        if fields[0] == "reboot" && fields.len() >= 7 {
            row.insert("event".into(), Value::String("reboot".into()));
            row.insert("time".into(), Value::String(fields[2..5].join(" ")));
            row.insert("pid".into(), Value::String(fields[6].into()));
            rows.push(finish_row(row, raw_mode));
            continue;
        }
        if fields.len() >= 2 && fields[0] == "system" && fields[1] == "boot" {
            row.insert("event".into(), Value::String("reboot".into()));
            if fields.len() >= 4 {
                row.insert("time".into(), Value::String(fields[2..4].join(" ")));
            }
            rows.push(finish_row(row, raw_mode));
            continue;
        }
        if fields[0] == "LOGIN" && fields.len() >= 5 {
            row.insert("event".into(), Value::String("login".into()));
            row.insert("tty".into(), Value::String(fields[1].into()));
            row.insert("time".into(), Value::String(fields[2..4].join(" ")));
            row.insert("pid".into(), Value::String(fields[4].into()));
            if fields.len() > 5 {
                row.insert("comment".into(), Value::String(fields[5..].join(" ")));
            }
            rows.push(finish_row(row, raw_mode));
            continue;
        }
        if fields[0] == "run-level" {
            row.insert(
                "event".into(),
                Value::String(fields.iter().take(2).copied().collect::<Vec<_>>().join(" ")),
            );
            if fields.len() >= 4 {
                row.insert("time".into(), Value::String(fields[2..4].join(" ")));
            }
            rows.push(finish_row(row, raw_mode));
            continue;
        }
        if fields.len() > 1 && fields[1] == "run-level" {
            continue;
        }
        if fields[0].starts_with("pts/") && fields.len() >= 4 {
            row.insert("tty".into(), Value::String(fields[0].into()));
            row.insert("time".into(), Value::String(fields[1..3].join(" ")));
            row.insert("pid".into(), Value::String(fields[3].into()));
            if fields.len() > 4 {
                row.insert("comment".into(), Value::String(fields[4..].join(" ")));
            }
            rows.push(finish_row(row, raw_mode));
            continue;
        }
        if fields.len() < 3 {
            return Err(ScocError::parse(
                descriptor.name,
                Some(line_number + 1),
                "unrecognized who output",
            ));
        }
        let process = if fields.len() >= 3
            && !matches!(fields[1], "+" | "-" | "?")
            && fields[2].starts_with("pts/")
        {
            Some(fields.remove(1).to_string())
        } else {
            None
        };
        row.insert("user".into(), Value::String(fields.remove(0).into()));
        if fields
            .first()
            .is_some_and(|v| matches!(*v, "+" | "-" | "?"))
        {
            row.insert(
                "writeable_tty".into(),
                Value::String(fields.remove(0).into()),
            );
        }
        if fields.is_empty() {
            rows.push(finish_row(row, raw_mode));
            continue;
        }
        row.insert("tty".into(), Value::String(fields.remove(0).into()));
        if let Some(process) = process {
            row.insert("process".into(), Value::String(process));
        }
        if fields.is_empty() {
            rows.push(finish_row(row, raw_mode));
            continue;
        }
        let time_width = if mac_month_re().is_match(fields[0]) {
            3
        } else {
            2
        };
        if fields.len() >= time_width {
            row.insert(
                "time".into(),
                Value::String(fields.drain(..time_width).collect::<Vec<_>>().join(" ")),
            );
        }
        if fields.len() == 1 {
            row.insert(
                "from".into(),
                Value::String(fields[0].trim_matches(|c| c == '(' || c == ')').into()),
            );
        } else if !fields.is_empty() {
            row.insert("idle".into(), Value::String(fields.remove(0).into()));
            if !fields.is_empty() {
                row.insert("pid".into(), Value::String(fields.remove(0).into()));
            }
            if !fields.is_empty() {
                let rest = fields.join(" ");
                if rest.starts_with('(') && rest.ends_with(')') {
                    row.insert("from".into(), Value::String(rest[1..rest.len() - 1].into()));
                } else {
                    row.insert("comment".into(), Value::String(rest));
                }
            }
        }
        rows.push(finish_row(row, raw_mode));
    }
    Ok(Value::Array(rows))
}

pub static WHO: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
