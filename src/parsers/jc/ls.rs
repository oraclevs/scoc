use regex::Regex;
use serde_json::{Map, Value};
use std::sync::OnceLock;

use crate::parsers::shared::streaming::BufferedBatchStream;
use crate::utils::{input_to_str, parse_ls_timestamp, to_i64_value};
use crate::{
    OptionSpec, OutputShape, ParseOptions, ParserCapabilities, ParserDescriptor, ParserOutput,
    ParserTag, Platform, ScocError, ScocParser, ScocStreamParser, UpstreamParser,
};

const OPTIONS: [OptionSpec; 1] = [OptionSpec::bool("raw", false, "Return JC raw output")];
const PLATFORMS: [Platform; 2] = [Platform::Linux, Platform::MacOs];
const TAGS: [ParserTag; 1] = [ParserTag::Command];
const ALIASES: [&str; 2] = ["vdir", "ls-s"];

pub struct LsParser;
pub static LS: LsParser = LsParser;

pub static DESCRIPTOR: ParserDescriptor = ParserDescriptor {
    name: "ls",
    aliases: &ALIASES,
    description: "`ls` and `vdir` command parser",
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
        standard_name: "ls",
        standard_version: "1.13",
        streaming_name: Some("ls-s"),
        streaming_version: Some("1.4"),
    }),
};

fn perm_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"^[-dclpsbDCMnP?]([-r][-w][-xsS]){2}([-r][-w][-xtT])[+.]?")
            .expect("valid ls permission regex")
    })
}
fn default_date_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"^[A-Za-z]{3}\s{1,2}\d{1,2}\s{1,2}[0-9:]{4,5}$").expect("valid ls date regex")
    })
}
fn long_iso_date_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^\d{4}-\d{2}-\d{2}$").expect("valid ls ISO date regex"))
}
fn long_iso_time_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^\d{2}:\d{2}$").expect("valid ls ISO time regex"))
}

fn detect_date_width(sample: &str) -> usize {
    let device = matches!(sample.as_bytes().first(), Some(b'b' | b'c'));
    let tokens: Vec<&str> = sample.split_whitespace().collect();
    let start = if device { 6 } else { 5 };
    if tokens.len() > start + 1
        && long_iso_date_re().is_match(tokens[start])
        && long_iso_time_re().is_match(tokens[start + 1])
    {
        2
    } else {
        3
    }
}

fn split_fixed_fields(line: &str, count: usize) -> Vec<String> {
    let bytes = line.as_bytes();
    let mut out = Vec::with_capacity(count + 1);
    let mut pos = 0usize;
    for _ in 0..count {
        while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }
        if pos >= bytes.len() {
            return out;
        }
        let start = pos;
        while pos < bytes.len() && !bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }
        out.push(line[start..pos].to_string());
    }
    while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
        pos += 1;
    }
    if pos <= bytes.len() {
        out.push(line[pos..].to_string());
    }
    out
}

fn normalize_row(object: &mut Map<String, Value>) {
    for key in ["links", "size", "major_number", "minor_number"] {
        if let Some(Value::String(value)) = object.get(key).cloned() {
            object.insert(key.into(), to_i64_value(&value));
        }
    }
    if let Some(Value::String(date)) = object.get("date").cloned() {
        if !default_date_re().is_match(&date) {
            let (epoch, epoch_utc) = parse_ls_timestamp(&date);
            if let Some(epoch) = epoch {
                object.insert("epoch".into(), Value::from(epoch));
            }
            if let Some(epoch) = epoch_utc {
                object.insert("epoch_utc".into(), Value::from(epoch));
            }
        }
    }
}

impl ScocParser for LsParser {
    fn descriptor(&self) -> &'static ParserDescriptor {
        &DESCRIPTOR
    }

    fn parse(&self, input: &[u8], options: &ParseOptions) -> Result<Value, ScocError> {
        let text = input_to_str("ls", input)?;
        if text.is_empty() {
            return Ok(Value::Array(Vec::new()));
        }
        let raw_mode = options.bool("raw").unwrap_or(false);
        let mut lines: Vec<&str> = text.split('\n').collect();
        if lines.last() == Some(&"") {
            lines.pop();
        }
        if lines.is_empty() {
            return Ok(Value::Array(Vec::new()));
        }
        if lines.first().is_some_and(|line| {
            line.starts_with("total ") && line[6..].chars().all(|c| c.is_ascii_digit())
        }) {
            lines.remove(0);
        }
        if lines.is_empty() {
            return Ok(Value::Array(Vec::new()));
        }

        let long_mode = lines.iter().any(|line| perm_re().is_match(line));
        let mut rows: Vec<Value> = Vec::new();
        let mut parent = String::new();

        if !long_mode {
            let mut next_is_parent = false;
            for entry in lines {
                if entry.is_empty() {
                    next_is_parent = true;
                    continue;
                }
                if next_is_parent && entry.ends_with(':') {
                    parent = entry.trim_end_matches(':').to_string();
                    next_is_parent = false;
                    continue;
                }
                next_is_parent = false;
                let mut row = Map::new();
                row.insert("filename".into(), Value::String(entry.to_string()));
                if !parent.is_empty() {
                    row.insert("parent".into(), Value::String(parent.clone()));
                }
                rows.push(Value::Object(row));
            }
            return Ok(Value::Array(rows));
        }

        if !perm_re().is_match(lines[0]) && lines[0].ends_with(':') {
            parent = lines.remove(0).trim_end_matches(':').to_string();
            if lines.first().is_some_and(|line| line.starts_with("total ")) {
                lines.remove(0);
            }
        }
        if lines.is_empty() {
            return Ok(Value::Array(Vec::new()));
        }
        let date_width = lines
            .iter()
            .find(|line| perm_re().is_match(line))
            .map_or(3, |line| detect_date_width(line));
        let mut new_section = false;

        for (line_index, entry) in lines.into_iter().enumerate() {
            if !perm_re().is_match(entry) && entry.ends_with(':') {
                parent = entry.trim_end_matches(':').to_string();
                new_section = true;
                continue;
            }
            if entry.starts_with("total ") {
                new_section = false;
                continue;
            }
            if new_section && entry.is_empty() {
                new_section = false;
                continue;
            }
            if !new_section && !perm_re().is_match(entry) {
                if let Some(Value::Object(previous)) = rows.last_mut() {
                    if let Some(Value::String(filename)) = previous.get_mut("filename") {
                        filename.push('\n');
                        filename.push_str(entry);
                        continue;
                    }
                }
                return Err(ScocError::parse(
                    "ls",
                    Some(line_index + 1),
                    "malformed long listing row",
                ));
            }
            if !perm_re().is_match(entry) {
                continue;
            }
            new_section = false;

            let device = matches!(entry.as_bytes().first(), Some(b'b' | b'c'));
            let size_width = if device { 2 } else { 1 };
            let fixed_count = 4 + size_width + date_width;
            let parsed = split_fixed_fields(entry, fixed_count);
            if parsed.len() < fixed_count + 1 {
                return Err(ScocError::parse(
                    "ls",
                    Some(line_index + 1),
                    "long listing row has too few fields",
                ));
            }
            let filename_field = &parsed[fixed_count];
            let (filename, link_to) = filename_field
                .split_once(" -> ")
                .map_or((filename_field.as_str(), None), |(name, target)| {
                    (name, Some(target))
                });

            let mut row = Map::new();
            row.insert("filename".into(), Value::String(filename.to_string()));
            if let Some(target) = link_to {
                row.insert("link_to".into(), Value::String(target.to_string()));
            }
            if !parent.is_empty() {
                row.insert("parent".into(), Value::String(parent.clone()));
            }
            row.insert("flags".into(), Value::String(parsed[0].clone()));
            row.insert("links".into(), Value::String(parsed[1].clone()));
            row.insert("owner".into(), Value::String(parsed[2].clone()));
            row.insert("group".into(), Value::String(parsed[3].clone()));
            if device {
                row.insert(
                    "major_number".into(),
                    Value::String(parsed[4].trim_end_matches(',').to_string()),
                );
                row.insert("minor_number".into(), Value::String(parsed[5].clone()));
            } else {
                row.insert("size".into(), Value::String(parsed[4].clone()));
            }
            let date_start = 4 + size_width;
            row.insert(
                "date".into(),
                Value::String(parsed[date_start..date_start + date_width].join(" ")),
            );
            if !raw_mode {
                normalize_row(&mut row);
            }
            rows.push(Value::Object(row));
        }
        Ok(Value::Array(rows))
    }

    fn stream_parser(
        &self,
        options: &ParseOptions,
    ) -> Result<Box<dyn ScocStreamParser>, ScocError> {
        Ok(Box::new(BufferedBatchStream::new("ls", options)))
    }
}
