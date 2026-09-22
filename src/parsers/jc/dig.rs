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
    name: "dig",
    aliases: &[],
    description: "`dig` command parser",
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
        standard_name: "dig",
        standard_version: "2.5",
        streaming_name: None,
        streaming_version: None,
    }),
};
fn rr(line: &str, raw_mode: bool) -> Option<Value> {
    let f = line.split_whitespace().collect::<Vec<_>>();
    if f.len() < 5 {
        return None;
    }
    let mut o = Map::new();
    o.insert("name".into(), Value::String(f[0].into()));
    o.insert("class".into(), Value::String(f[2].into()));
    o.insert("type".into(), Value::String(f[3].into()));
    o.insert(
        "ttl".into(),
        if raw_mode {
            Value::String(f[1].into())
        } else {
            to_i64_value(f[1])
        },
    );
    let data = f[4..].join(" ").trim_matches('"').to_string();
    o.insert("data".into(), Value::String(data));
    Some(Value::Object(o))
}
fn parse(
    d: &'static ParserDescriptor,
    input: &[u8],
    opts: &ParseOptions,
) -> Result<Value, ScocError> {
    let text = input_to_str(d.name, input)?;
    let r = raw(opts);
    let mut out = vec![];
    let mut cur = Map::new();
    let mut section = "";
    for line in text.lines().chain(std::iter::once(";; END")) {
        let s = line.trim();
        if s.starts_with(";; ->>HEADER<<-") {
            if !cur.is_empty() {
                out.push(Value::Object(std::mem::take(&mut cur)));
            }
            let f = s.split_whitespace().collect::<Vec<_>>();
            if f.len() >= 8 {
                for (k, v) in [
                    ("opcode", f[3].trim_end_matches(',')),
                    ("status", f[5].trim_end_matches(',')),
                ] {
                    cur.insert(k.into(), Value::String(v.into()));
                }
                cur.insert(
                    "id".into(),
                    if r {
                        Value::String(f[7].into())
                    } else {
                        to_i64_value(f[7])
                    },
                );
            }
            continue;
        }
        if s.starts_with(";; flags:") {
            let Some((left, right)) = s.split_once(';') else {
                continue;
            };
            let flags = left
                .trim_start_matches(";; flags:")
                .split_whitespace()
                .map(|x| Value::String(x.into()))
                .collect();
            cur.insert("flags".into(), Value::Array(flags));
            for part in right.split(',') {
                let f = part
                    .replace(':', " ")
                    .split_whitespace()
                    .map(str::to_string)
                    .collect::<Vec<_>>();
                if f.len() >= 2 {
                    let key = match f[0].as_str() {
                        "QUERY" => "query_num",
                        "ANSWER" => "answer_num",
                        "AUTHORITY" => "authority_num",
                        "ADDITIONAL" => "additional_num",
                        _ => continue,
                    };
                    cur.insert(
                        key.into(),
                        if r {
                            Value::String(f[1].clone())
                        } else {
                            to_i64_value(&f[1])
                        },
                    );
                }
            }
            continue;
        }
        if s.ends_with("SECTION:") {
            section = if s.contains("QUESTION") {
                "question"
            } else if s.contains("ANSWER") {
                "answer"
            } else if s.contains("AUTHORITY") {
                "authority"
            } else if s.contains("ADDITIONAL") {
                "additional"
            } else if s.contains("OPT PSEUDOSECTION") {
                "opt"
            } else {
                ""
            };
            continue;
        }
        if s.starts_with(";; Query time:") {
            let v = s
                .trim_start_matches(";; Query time:")
                .split_whitespace()
                .next()
                .unwrap_or("");
            cur.insert(
                "query_time".into(),
                if r {
                    Value::String(s.trim_start_matches(";; Query time:").trim().into())
                } else {
                    to_i64_value(v)
                },
            );
            continue;
        }
        if s.starts_with(";; SERVER:") {
            cur.insert(
                "server".into(),
                Value::String(s.trim_start_matches(";; SERVER:").trim().into()),
            );
            continue;
        }
        if s.starts_with(";; WHEN:") {
            cur.insert(
                "when".into(),
                Value::String(s.trim_start_matches(";; WHEN:").trim().into()),
            );
            continue;
        }
        if s.starts_with(";; MSG SIZE") || s.starts_with(";; MSG SIZE  rcvd:") {
            let v = s.split_whitespace().last().unwrap_or("");
            cur.insert(
                "rcvd".into(),
                if r {
                    Value::String(v.into())
                } else {
                    to_i64_value(v)
                },
            );
            continue;
        }
        if s.starts_with(";; END") {
            if !cur.is_empty() {
                out.push(Value::Object(std::mem::take(&mut cur)));
            }
            continue;
        }
        if s.is_empty() || s.starts_with(';') {
            if section == "question" && s.starts_with(';') && !s.starts_with(";;") {
                let f = s
                    .trim_start_matches(';')
                    .split_whitespace()
                    .collect::<Vec<_>>();
                if f.len() >= 3 {
                    let mut q = Map::new();
                    q.insert("name".into(), Value::String(f[0].into()));
                    q.insert("class".into(), Value::String(f[1].into()));
                    q.insert("type".into(), Value::String(f[2].into()));
                    cur.insert("question".into(), Value::Object(q));
                }
            }
            continue;
        }
        if matches!(section, "answer" | "authority" | "additional") {
            if let Some(v) = rr(s, r) {
                let e = cur.entry(section).or_insert_with(|| Value::Array(vec![]));
                if let Value::Array(a) = e {
                    a.push(v);
                }
            }
        } else if section.is_empty() && !s.starts_with(";;") {
            if let Some(v) = rr(s, r) {
                let e = cur.entry("answer").or_insert_with(|| Value::Array(vec![]));
                if let Value::Array(a) = e {
                    a.push(v);
                }
            }
        }
    }
    Ok(Value::Array(out))
}
pub static DIG: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
