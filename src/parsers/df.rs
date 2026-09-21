use serde_json::{Map, Value};

use crate::utils::{convert_size_to_int, input_to_str, sparse_table_parse, to_i64_value};
use crate::{
    OptionSpec, OutputShape, ParseOptions, ParserCapabilities, ParserDescriptor, ParserOutput,
    ParserTag, Platform, ScocError, ScocParser, UpstreamParser,
};

const OPTIONS: [OptionSpec; 1] = [OptionSpec::bool("raw", false, "Return JC raw output")];
const PLATFORMS: [Platform; 2] = [Platform::Linux, Platform::MacOs];
const TAGS: [ParserTag; 1] = [ParserTag::Command];

pub struct DfParser;
pub static DF: DfParser = DfParser;

pub static DESCRIPTOR: ParserDescriptor = ParserDescriptor {
    name: "df",
    aliases: &[],
    description: "`df` command parser",
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
        standard_name: "df",
        standard_version: "2.1",
        streaming_name: None,
        streaming_version: None,
    }),
};

fn marker_for(index: usize, width: usize) -> String {
    let seed = format!("scoc{index:08x}");
    if seed.len() >= width {
        seed[..width].to_string()
    } else {
        let mut out = seed;
        out.push_str(&"0".repeat(width - out.len()));
        out
    }
}

fn prepare_lines(text: &str) -> (Vec<String>, bool, Vec<(String, String)>) {
    let mut lines: Vec<String> = text
        .lines()
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect();
    if lines.is_empty() {
        return (lines, false, Vec::new());
    }
    lines[0] = lines[0]
        .to_ascii_lowercase()
        .replace('-', "_")
        .replace("mounted on", "mounted_on");
    let posix_mode = lines[0].contains("use%");

    let header = lines[0].clone();
    let mut space_count = 0usize;
    for ch in header.chars().skip(10) {
        if ch == ' ' {
            space_count += 1;
        } else {
            break;
        }
    }
    let filesystem_col_len = space_count + 9;
    let mut replacements = Vec::new();
    if filesystem_col_len > 0 {
        for (index, line) in lines.iter_mut().enumerate().skip(1) {
            let Some(filesystem) = line.split_whitespace().next() else {
                continue;
            };
            if filesystem.len() > filesystem_col_len {
                let filesystem = filesystem.to_string();
                let marker = marker_for(index, filesystem_col_len);
                *line = line.replacen(&filesystem, &marker, 1);
                replacements.push((marker, filesystem));
            }
        }
    }
    (lines, posix_mode, replacements)
}

fn restore_filesystems(rows: &mut [Value], replacements: &[(String, String)]) {
    for row in rows {
        let Some(object) = row.as_object_mut() else {
            continue;
        };
        let Some(Value::String(filesystem)) = object.get("filesystem") else {
            continue;
        };
        if let Some((_, original)) = replacements.iter().find(|(marker, _)| marker == filesystem) {
            object.insert("filesystem".into(), Value::String(original.clone()));
        }
    }
}

fn normalize_row(object: &mut Map<String, Value>, posix_mode: bool) {
    for (old, new) in [
        ("avail", "available"),
        ("use%", "use_percent"),
        ("capacity", "capacity_percent"),
        ("%iused", "iused_percent"),
    ] {
        if let Some(value) = object.remove(old) {
            object.insert(new.into(), value);
        }
    }

    let keys: Vec<String> = object.keys().cloned().collect();
    for key in keys {
        let Some(Value::String(value)) = object.get(&key).cloned() else {
            continue;
        };
        if key.contains("_blocks") {
            object.insert(key, to_i64_value(&value));
            continue;
        }
        if matches!(
            key.as_str(),
            "use_percent" | "capacity_percent" | "iused_percent"
        ) {
            object.insert(key, to_i64_value(value.trim_end_matches('%')));
            continue;
        }
        if matches!(key.as_str(), "ifree" | "iused") {
            object.insert(key, to_i64_value(&value));
            continue;
        }
        if matches!(key.as_str(), "size" | "used" | "available") {
            let converted =
                convert_size_to_int(&value, false, posix_mode).map_or(Value::Null, Value::from);
            object.insert(key, converted);
        }
    }
}

impl ScocParser for DfParser {
    fn descriptor(&self) -> &'static ParserDescriptor {
        &DESCRIPTOR
    }

    fn parse(&self, input: &[u8], options: &ParseOptions) -> Result<Value, ScocError> {
        let text = input_to_str("df", input)?;
        if text.trim().is_empty() {
            return Ok(Value::Array(Vec::new()));
        }
        let (lines, posix_mode, replacements) = prepare_lines(text);
        let refs: Vec<&str> = lines.iter().map(String::as_str).collect();
        let mut rows = sparse_table_parse(&refs)?;
        restore_filesystems(&mut rows, &replacements);
        if !options.bool("raw").unwrap_or(false) {
            for row in &mut rows {
                if let Some(object) = row.as_object_mut() {
                    normalize_row(object, posix_mode);
                }
            }
        }
        Ok(Value::Array(rows))
    }
}
