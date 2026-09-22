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
    name: "git-diff",
    aliases: &[],
    description: "`git diff --name-status` command parser",
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
        standard_name: "git-diff",
        standard_version: "1.0",
        streaming_name: None,
        streaming_version: None,
    }),
};
fn desc(c: char) -> Option<&'static str> {
    Some(match c {
        'A' => "Added",
        'C' => "Copied",
        'D' => "Deleted",
        'M' => "Modified",
        'R' => "Renamed",
        'T' => "Type changed",
        'U' => "Unmerged",
        'X' => "Unknown",
        'B' => "Pairing broken",
        _ => return None,
    })
}
fn parse(
    d: &'static ParserDescriptor,
    input: &[u8],
    opts: &ParseOptions,
) -> Result<Value, ScocError> {
    let text = input_to_str(d.name, input)?;
    let raw_mode = raw(opts);
    let mut out = vec![];
    for (n, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let f = line.split('\t').collect::<Vec<_>>();
        if f.len() < 2 {
            return Err(ScocError::parse(
                d.name,
                Some(n + 1),
                "expected tab-separated git --name-status output",
            ));
        }
        let status = f[0];
        let Some(code) = status.chars().next() else {
            continue;
        };
        let mut o = Map::new();
        if raw_mode {
            o.insert("status".into(), Value::String(status.into()));
            o.insert("similarity".into(), Value::Null);
            o.insert(
                "old_path".into(),
                if matches!(code, 'R' | 'C') && f.len() >= 3 {
                    Value::String(f[1].into())
                } else {
                    Value::Null
                },
            );
            o.insert(
                "path".into(),
                Value::String(
                    if matches!(code, 'R' | 'C') && f.len() >= 3 {
                        f[2]
                    } else {
                        f[1]
                    }
                    .into(),
                ),
            );
        } else {
            o.insert("status".into(), Value::String(code.to_string()));
            o.insert(
                "status_description".into(),
                desc(code)
                    .map(|x| Value::String(x.into()))
                    .unwrap_or(Value::Null),
            );
            o.insert(
                "similarity".into(),
                if matches!(code, 'R' | 'C') {
                    to_i64_value(&status[1..])
                } else {
                    Value::Null
                },
            );
            o.insert(
                "old_path".into(),
                if matches!(code, 'R' | 'C') && f.len() >= 3 {
                    Value::String(f[1].into())
                } else {
                    Value::Null
                },
            );
            o.insert(
                "path".into(),
                Value::String(
                    if matches!(code, 'R' | 'C') && f.len() >= 3 {
                        f[2]
                    } else {
                        f[1]
                    }
                    .into(),
                ),
            );
        }
        out.push(Value::Object(o));
    }
    Ok(Value::Array(out))
}
pub static GIT_DIFF: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
