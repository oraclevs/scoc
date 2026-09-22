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
    name: "jobs",
    aliases: &[],
    description: "`jobs` command parser",
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
        standard_name: "jobs",
        standard_version: "1.6",
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
    let mut out = vec![];
    for (n, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let mut parts = line.split_whitespace();
        let Some(job) = parts.next() else { continue };
        let mut job = job
            .trim_start_matches('[')
            .trim_end_matches(']')
            .to_string();
        let hist = if job.ends_with('+') {
            job.pop();
            Some("current")
        } else if job.ends_with('-') {
            job.pop();
            Some("previous")
        } else {
            None
        };
        let mut next = parts
            .next()
            .ok_or_else(|| ScocError::parse(d.name, Some(n + 1), "missing job status"))?;
        let pid = if next.chars().all(|c| c.is_ascii_digit()) {
            let p = next.to_string();
            next = parts
                .next()
                .ok_or_else(|| ScocError::parse(d.name, Some(n + 1), "missing status"))?;
            Some(p)
        } else {
            None
        };
        let status = next;
        let command = parts.collect::<Vec<_>>().join(" ");
        let mut o = Map::new();
        o.insert(
            "job_number".into(),
            if r {
                Value::String(job.clone())
            } else {
                to_i64_value(&job)
            },
        );
        if let Some(p) = pid {
            o.insert(
                "pid".into(),
                if r {
                    Value::String(p.clone())
                } else {
                    to_i64_value(&p)
                },
            );
        }
        if let Some(h) = hist {
            o.insert("history".into(), Value::String(h.into()));
        }
        o.insert("status".into(), Value::String(status.into()));
        o.insert("command".into(), Value::String(command));
        out.push(Value::Object(o));
    }
    Ok(Value::Array(out))
}
pub static JOBS: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
