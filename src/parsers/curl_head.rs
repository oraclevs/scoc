use crate::parsers::common::{raw, StaticParser};
use crate::utils::{input_to_str, to_f64_value, to_i64_value};
use crate::{
    OptionSpec, OutputShape, ParseOptions, ParserCapabilities, ParserDescriptor, ParserOutput,
    ParserTag, Platform, ScocError, UpstreamParser,
};
use chrono::DateTime;
use serde_json::{Map, Value};
const OPTIONS: [OptionSpec; 1] = [OptionSpec::bool("raw", false, "Return JC raw output")];
const PLATFORMS: [Platform; 2] = [Platform::Linux, Platform::MacOs];
const TAGS: [ParserTag; 1] = [ParserTag::Command];
pub static DESCRIPTOR: ParserDescriptor = ParserDescriptor {
    name: "curl-head",
    aliases: &[],
    description: "`curl --head` command parser",
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
        standard_name: "curl-head",
        standard_version: "1.0",
        streaming_name: None,
        streaming_version: None,
    }),
};
fn epoch(v: &str) -> Option<i64> {
    DateTime::parse_from_rfc2822(v).ok().map(|d| d.timestamp())
}
fn parse(
    descriptor: &'static ParserDescriptor,
    input: &[u8],
    options: &ParseOptions,
) -> Result<Value, ScocError> {
    let text = input_to_str(descriptor.name, input)?;
    if text.trim().is_empty() {
        return Ok(Value::Array(vec![]));
    }
    let verbose = text.lines().next().is_some_and(|l| l.starts_with("* "));
    let mut clean = Vec::new();
    for line in text.lines() {
        if line.starts_with("> ") || line.starts_with("< ") {
            clean.push(line[2..].to_string());
        } else if line.starts_with("* ") || verbose {
            if line.trim().is_empty() {
                clean.push(String::new());
            }
        } else {
            clean.push(line.to_string());
        }
    }
    let raw_mode = raw(options);
    let multi = [
        "content-security-policy",
        "content-security-policy-report-only",
        "cookie",
        "set-cookie",
    ];
    let split_multi = [
        "accept",
        "accept-ch",
        "accept-encoding",
        "accept-language",
        "accept-patch",
        "accept-post",
        "accept-ranges",
        "access-control-allow-headers",
        "access-control-allow-methods",
        "access-control-expose-headers",
        "access-control-request-headers",
        "allow",
        "alt-svc",
        "cache-control",
        "clear-site-data",
        "connection",
        "content-encoding",
        "content-language",
        "critical-ch",
        "expect-ct",
        "forwarded",
        "if-match",
        "if-none-match",
        "im",
        "keep-alive",
        "link",
        "permissions-policy",
        "permissions-policy-report-only",
        "pragma",
        "proxy-authenticate",
        "reporting-endpoints",
        "sec-ch-ua",
        "sec-ch-ua-full-version-list",
        "server",
        "server-timing",
        "timing-allow-origin",
        "trailer",
        "transfer-encoding",
        "upgrade",
        "vary",
        "via",
        "warning",
        "www-authenticate",
        "x-cache-hits",
    ];
    let ints = [
        "accept-ch-lifetime",
        "access-control-max-age",
        "age",
        "content-dpr",
        "content-length",
        "device-memory",
        "downlink",
        "dpr",
        "large-allocation",
        "max-forwards",
        "rtt",
        "upgrade-insecure-requests",
    ];
    let dates = [
        "date",
        "if-modified-since",
        "if-unmodified-since",
        "last-modified",
        "memento-datetime",
        "expires",
        "retry-after",
        "if-range",
    ];
    let mut rows = Vec::new();
    let mut obj = Map::new();
    for (idx, line) in clean.iter().filter(|l| !l.trim().is_empty()).enumerate() {
        let first = line
            .split_whitespace()
            .next()
            .unwrap_or("")
            .trim_end_matches(':')
            .to_ascii_lowercase();
        let method = [
            "connect", "delete", "get", "head", "options", "patch", "post", "put", "trace",
        ]
        .contains(&first.as_str());
        if method || first.starts_with("http/") {
            if !obj.is_empty() {
                rows.push(Value::Object(std::mem::take(&mut obj)));
            }
            if method {
                let p: Vec<&str> = line
                    .splitn(3, char::is_whitespace)
                    .filter(|v| !v.is_empty())
                    .collect();
                if p.len() < 3 {
                    return Err(ScocError::parse(
                        descriptor.name,
                        Some(idx + 1),
                        "malformed HTTP request line",
                    ));
                }
                obj.insert("_type".into(), Value::String("request".into()));
                obj.insert("_request_method".into(), Value::String(p[0].into()));
                obj.insert("_request_uri".into(), Value::String(p[1].into()));
                obj.insert("_request_version".into(), Value::String(p[2].into()));
            } else {
                let p: Vec<&str> = line
                    .splitn(3, char::is_whitespace)
                    .filter(|v| !v.is_empty())
                    .collect();
                if p.len() < 2 {
                    return Err(ScocError::parse(
                        descriptor.name,
                        Some(idx + 1),
                        "malformed HTTP response line",
                    ));
                }
                obj.insert("_type".into(), Value::String("response".into()));
                obj.insert("_response_version".into(), Value::String(p[0].into()));
                obj.insert("_response_status".into(), to_i64_value(p[1]));
                obj.insert(
                    "_response_reason".into(),
                    p.get(2).map_or(Value::Null, |v| {
                        Value::Array(vec![Value::String((*v).into())])
                    }),
                );
            }
            continue;
        }
        let Some((key, value)) = line.split_once(": ") else {
            return Err(ScocError::parse(
                descriptor.name,
                Some(idx + 1),
                "malformed HTTP header",
            ));
        };
        let key = key.to_ascii_lowercase();
        if split_multi.contains(&key.as_str()) {
            let vals = value
                .split(',')
                .map(|v| Value::String(v.trim().into()))
                .collect::<Vec<_>>();
            match obj.get_mut(&key) {
                Some(Value::Array(existing)) => existing.extend(vals),
                _ => {
                    obj.insert(key, Value::Array(vals));
                }
            }
        } else if multi.contains(&key.as_str()) {
            match obj.get_mut(&key) {
                Some(Value::Array(existing)) => existing.push(Value::String(value.into())),
                _ => {
                    obj.insert(key, Value::Array(vec![Value::String(value.into())]));
                }
            }
        } else {
            obj.insert(key, Value::String(value.into()));
        }
    }
    if !obj.is_empty() {
        rows.push(Value::Object(obj));
    }
    if !raw_mode {
        for row in &mut rows {
            let Some(o) = row.as_object_mut() else {
                continue;
            };
            for k in ints {
                if let Some(Value::String(v)) = o.get(k).cloned() {
                    o.insert(k.into(), to_i64_value(&v));
                }
            }
            if let Some(Value::String(v)) = o.get("x-content-duration").cloned() {
                o.insert("x-content-duration".into(), to_f64_value(&v));
            }
            for k in dates {
                if let Some(Value::String(v)) = o.get(k).cloned() {
                    if let Some(ts) = epoch(&v) {
                        o.insert(format!("{k}_epoch_utc"), Value::from(ts));
                    }
                    if ["expires", "retry-after"].contains(&k)
                        && v.chars().all(|c| c.is_ascii_digit())
                    {
                        o.insert(k.into(), to_i64_value(&v));
                    }
                }
            }
            if let Some(Value::Array(vals)) = o.get("x-cache-hits").cloned() {
                o.insert(
                    "x-cache-hits".into(),
                    Value::Array(
                        vals.into_iter()
                            .map(|v| match v {
                                Value::String(s) => to_i64_value(&s),
                                other => other,
                            })
                            .collect(),
                    ),
                );
            }
        }
    }
    Ok(Value::Array(rows))
}
pub static CURL_HEAD: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
