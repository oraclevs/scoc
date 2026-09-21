use regex::Regex;
use serde_json::{Map, Value};
use std::sync::OnceLock;

use crate::utils::{input_to_str, to_f64_value, to_i64_value, LineBuffer};
use crate::{
    OptionSpec, OutputShape, ParseOptions, ParserCapabilities, ParserDescriptor, ParserOutput,
    ParserTag, Platform, ScocError, ScocParser, ScocStreamParser, UpstreamParser,
};

const OPTIONS: [OptionSpec; 2] = [
    OptionSpec::bool("raw", false, "Return JC raw output"),
    OptionSpec::bool(
        "ignoreErrors",
        false,
        "Emit JC-compatible metadata for malformed streaming records",
    ),
];
const PLATFORMS: [Platform; 2] = [Platform::Linux, Platform::MacOs];
const TAGS: [ParserTag; 1] = [ParserTag::Command];
const ALIASES: [&str; 1] = ["ping6"];

pub struct PingParser;
pub static PING: PingParser = PingParser;

pub static DESCRIPTOR: ParserDescriptor = ParserDescriptor {
    name: "ping",
    aliases: &ALIASES,
    description: "`ping` and `ping6` command parser",
    parser_version: "0.1.0",
    platforms: &PLATFORMS,
    tags: &TAGS,
    output: ParserOutput {
        normalized: OutputShape::Record,
        raw: Some(OutputShape::Record),
        stream_item: Some(OutputShape::Record),
    },
    capabilities: ParserCapabilities {
        raw: true,
        streaming: true,
        ignore_errors: true,
    },
    options: &OPTIONS,
    upstream: Some(UpstreamParser {
        standard_name: "ping",
        standard_version: "1.11",
        streaming_name: Some("ping-s"),
        streaming_version: Some("1.6"),
    }),
};

fn header_bytes_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(\d+)(?:\(\d+\))?\s+(?:bytes of data|data bytes)")
            .expect("valid ping header regex")
    })
}
fn reply_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(
        r"^(?:\[([0-9.]+)\]\s+)?(\d+) bytes from ([^\s:]+)(?::|\s).*?icmp_seq[=\s](\d+).*?(?:ttl|hlim)[=\s](\d+).*?time[=<\s]([0-9.]+)\s*ms(?:.*?(DUP!))?",
    ).expect("valid ping reply regex"))
}
fn stats_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(\d+) packets transmitted,\s*(\d+) (?:packets )?received,.*?([0-9.]+)% packet loss",
        )
        .expect("valid ping stats regex")
    })
}
fn rtt_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"=\s*([0-9.]+)/([0-9.]+)/([0-9.]+)/([0-9.]+)\s*ms")
            .expect("valid ping rtt regex")
    })
}
fn linux_timeout_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"no answer yet for icmp_seq[=\s](\d+)").expect("valid ping timeout regex")
    })
}
fn bsd_timeout_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"Request timeout for icmp_seq\s+(\d+)").expect("valid ping timeout regex")
    })
}

fn error_type(line: &str) -> Option<&'static str> {
    const MAP: [(&str, &str); 12] = [
        ("Destination Net Unreachable", "destination_net_unreachable"),
        (
            "Destination Host Unreachable",
            "destination_host_unreachable",
        ),
        (
            "Destination Protocol Unreachable",
            "destination_protocol_unreachable",
        ),
        (
            "Destination Port Unreachable",
            "destination_port_unreachable",
        ),
        ("Packet filtered", "packet_filtered"),
        ("Time to live exceeded", "time_to_live_exceeded"),
        ("Destination unreachable", "destination_unreachable"),
        ("Packet too big", "packet_too_big"),
        ("Time exceeded:", "time_exceeded"),
        ("Parameter problem:", "parameter_problem"),
        ("Network is unreachable", "network_unreachable"),
        ("Name or service not known", "name_or_service_not_known"),
    ];
    MAP.iter()
        .find_map(|(needle, code)| line.contains(needle).then_some(*code))
}

fn parse_header(line: &str) -> Option<(String, String)> {
    if !(line.starts_with("PING ") || line.starts_with("PING6(")) {
        return None;
    }
    let destination_ip = if let Some(start) = line.find('(') {
        let rest = &line[start + 1..];
        rest.find(')').map(|end| rest[..end].to_string())
    } else {
        line.split_whitespace()
            .nth(1)
            .map(|token| token.trim_matches(':').to_string())
    }?;
    let bytes = header_bytes_re()
        .captures(line)?
        .get(1)?
        .as_str()
        .to_string();
    Some((destination_ip, bytes))
}

fn int_or_string(value: &str, raw: bool) -> Value {
    if raw {
        Value::String(value.to_string())
    } else {
        to_i64_value(value)
    }
}
fn float_or_string(value: &str, raw: bool) -> Value {
    if raw {
        Value::String(value.to_string())
    } else {
        to_f64_value(value)
    }
}

fn parse_reply(
    line: &str,
    raw: bool,
    destination_ip: Option<&str>,
    sent_bytes: Option<&str>,
    pattern: Option<&str>,
) -> Option<Value> {
    let captures = reply_re().captures(line)?;
    let mut row = Map::new();
    row.insert("type".into(), Value::String("reply".into()));
    if let Some(destination) = destination_ip {
        row.insert("destination_ip".into(), Value::String(destination.into()));
    }
    if let Some(bytes) = sent_bytes {
        row.insert("sent_bytes".into(), int_or_string(bytes, raw));
    }
    row.insert(
        "pattern".into(),
        pattern.map_or(Value::Null, |value| Value::String(value.into())),
    );
    row.insert(
        "timestamp".into(),
        captures.get(1).map_or(Value::Null, |timestamp| {
            float_or_string(timestamp.as_str(), raw)
        }),
    );
    row.insert(
        "response_bytes".into(),
        int_or_string(captures.get(2)?.as_str(), raw),
    );
    row.insert(
        "response_ip".into(),
        Value::String(captures.get(3)?.as_str().trim_matches(':').into()),
    );
    row.insert(
        "icmp_seq".into(),
        int_or_string(captures.get(4)?.as_str(), raw),
    );
    row.insert("ttl".into(), int_or_string(captures.get(5)?.as_str(), raw));
    row.insert(
        "time_ms".into(),
        float_or_string(captures.get(6)?.as_str(), raw),
    );
    row.insert("duplicate".into(), Value::Bool(captures.get(7).is_some()));
    Some(Value::Object(row))
}

fn parse_timeout(
    line: &str,
    raw: bool,
    destination_ip: Option<&str>,
    sent_bytes: Option<&str>,
    pattern: Option<&str>,
) -> Option<Value> {
    let sequence = linux_timeout_re()
        .captures(line)
        .and_then(|c| c.get(1))
        .or_else(|| bsd_timeout_re().captures(line).and_then(|c| c.get(1)))?;
    let mut row = Map::new();
    row.insert("type".into(), Value::String("timeout".into()));
    if let Some(destination) = destination_ip {
        row.insert("destination_ip".into(), Value::String(destination.into()));
    }
    if let Some(bytes) = sent_bytes {
        row.insert("sent_bytes".into(), int_or_string(bytes, raw));
    }
    row.insert(
        "pattern".into(),
        pattern.map_or(Value::Null, |value| Value::String(value.into())),
    );
    row.insert("timestamp".into(), Value::Null);
    row.insert("icmp_seq".into(), int_or_string(sequence.as_str(), raw));
    Some(Value::Object(row))
}

fn batch_response(value: Value) -> Value {
    let Value::Object(mut source) = value else {
        return value;
    };
    let mut row = Map::new();
    for key in [
        "type",
        "timestamp",
        "response_bytes",
        "response_ip",
        "icmp_seq",
        "ttl",
        "time_ms",
        "duplicate",
        "vr",
        "hl",
        "tos",
        "len",
        "id",
        "flg",
        "off",
        "pro",
        "cks",
        "src",
        "dst",
        "unparsed_line",
    ] {
        if let Some(value) = source.remove(key) {
            let output_key = if key == "response_bytes" {
                "bytes"
            } else {
                key
            };
            row.insert(output_key.into(), value);
        }
    }
    Value::Object(row)
}

fn add_success_meta(mut value: Value, enabled: bool) -> Value {
    if !enabled {
        return value;
    }
    if let Value::Object(row) = &mut value {
        let mut meta = Map::new();
        meta.insert("success".into(), Value::Bool(true));
        row.insert("_jc_meta".into(), Value::Object(meta));
    }
    value
}

#[derive(Default, Clone)]
struct PingState {
    destination_ip: Option<String>,
    sent_bytes: Option<String>,
    pattern: Option<String>,
    destination: Option<String>,
    footer: bool,
    packets_transmitted: Option<String>,
    packets_received: Option<String>,
    packet_loss_percent: Option<String>,
    duplicates: Option<String>,
    errors: Option<String>,
    corrupted: Option<String>,
    round_trip_ms_min: Option<String>,
    round_trip_ms_avg: Option<String>,
    round_trip_ms_max: Option<String>,
    round_trip_ms_stddev: Option<String>,
    time_ms: Option<String>,
}

fn parse_stats_line(line: &str, state: &mut PingState) -> bool {
    let Some(captures) = stats_re().captures(line) else {
        return false;
    };
    state.packets_transmitted = captures.get(1).map(|m| m.as_str().to_string());
    state.packets_received = captures.get(2).map(|m| m.as_str().to_string());
    state.packet_loss_percent = captures.get(3).map(|m| m.as_str().to_string());
    if let Some(time) = Regex::new(r"time (\d+)ms")
        .ok()
        .and_then(|regex| regex.captures(line))
        .and_then(|captures| captures.get(1))
    {
        state.time_ms = Some(time.as_str().to_string());
    }
    let duplicate_re = Regex::new(r"\+(\d+) duplicates").expect("valid duplicate regex");
    let error_re = Regex::new(r"\+(\d+) errors").expect("valid error regex");
    let corrupted_re = Regex::new(r"\+(\d+) corrupted").expect("valid corrupted regex");
    state.duplicates = duplicate_re
        .captures(line)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().into())
        .or(Some("0".into()));
    state.errors = error_re
        .captures(line)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().into());
    state.corrupted = corrupted_re
        .captures(line)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().into());
    true
}

fn parse_rtt_line(line: &str, state: &mut PingState) -> bool {
    let Some(captures) = rtt_re().captures(line) else {
        return false;
    };
    state.round_trip_ms_min = captures.get(1).map(|m| m.as_str().into());
    state.round_trip_ms_avg = captures.get(2).map(|m| m.as_str().into());
    state.round_trip_ms_max = captures.get(3).map(|m| m.as_str().into());
    state.round_trip_ms_stddev = captures.get(4).map(|m| m.as_str().into());
    true
}

fn insert_optional_number(
    object: &mut Map<String, Value>,
    key: &str,
    value: &Option<String>,
    raw: bool,
    float: bool,
) {
    if let Some(value) = value {
        object.insert(
            key.into(),
            if float {
                float_or_string(value, raw)
            } else {
                int_or_string(value, raw)
            },
        );
    }
}

fn summary_record(state: &PingState, raw: bool) -> Value {
    let mut row = Map::new();
    row.insert("type".into(), Value::String("summary".into()));
    if let Some(value) = &state.destination_ip {
        row.insert("destination_ip".into(), Value::String(value.clone()));
    }
    if let Some(value) = &state.sent_bytes {
        row.insert("sent_bytes".into(), int_or_string(value, raw));
    }
    row.insert(
        "pattern".into(),
        state
            .pattern
            .as_ref()
            .map_or(Value::Null, |value| Value::String(value.clone())),
    );
    if let Some(value) = &state.destination {
        row.insert("destination".into(), Value::String(value.clone()));
    }
    insert_optional_number(
        &mut row,
        "packets_transmitted",
        &state.packets_transmitted,
        raw,
        false,
    );
    insert_optional_number(
        &mut row,
        "packets_received",
        &state.packets_received,
        raw,
        false,
    );
    insert_optional_number(
        &mut row,
        "packet_loss_percent",
        &state.packet_loss_percent,
        raw,
        true,
    );
    insert_optional_number(&mut row, "duplicates", &state.duplicates, raw, false);
    insert_optional_number(&mut row, "time_ms", &state.time_ms, raw, false);
    insert_optional_number(&mut row, "errors", &state.errors, raw, false);
    insert_optional_number(&mut row, "corrupted", &state.corrupted, raw, false);
    insert_optional_number(
        &mut row,
        "round_trip_ms_min",
        &state.round_trip_ms_min,
        raw,
        true,
    );
    insert_optional_number(
        &mut row,
        "round_trip_ms_avg",
        &state.round_trip_ms_avg,
        raw,
        true,
    );
    insert_optional_number(
        &mut row,
        "round_trip_ms_max",
        &state.round_trip_ms_max,
        raw,
        true,
    );
    insert_optional_number(
        &mut row,
        "round_trip_ms_stddev",
        &state.round_trip_ms_stddev,
        raw,
        true,
    );
    Value::Object(row)
}

impl ScocParser for PingParser {
    fn descriptor(&self) -> &'static ParserDescriptor {
        &DESCRIPTOR
    }

    fn parse(&self, input: &[u8], options: &ParseOptions) -> Result<Value, ScocError> {
        let text = input_to_str("ping", input)?;
        if text.trim().is_empty() {
            return Ok(Value::Object(Map::new()));
        }
        let raw = options.bool("raw").unwrap_or(false);
        let mut state = PingState::default();
        let mut responses = Vec::new();

        for (index, line) in text.lines().enumerate() {
            if line.starts_with("PATTERN: ") {
                state.pattern = line.split_once(": ").map(|(_, value)| value.to_string());
                continue;
            }
            if let Some((destination, bytes)) = parse_header(line) {
                state.destination_ip = Some(destination);
                state.sent_bytes = Some(bytes);
                continue;
            }
            if line.starts_with("---") {
                state.footer = true;
                state.destination = line
                    .split_whitespace()
                    .nth(1)
                    .map(|value| value.to_string());
                continue;
            }
            if line.trim().is_empty() {
                continue;
            }
            if state.footer {
                if parse_stats_line(line, &mut state) || parse_rtt_line(line, &mut state) {
                    continue;
                }
                continue;
            }
            if let Some(reply) = parse_reply(
                line,
                raw,
                state.destination_ip.as_deref(),
                state.sent_bytes.as_deref(),
                state.pattern.as_deref(),
            ) {
                responses.push(batch_response(reply));
                continue;
            }
            if let Some(timeout) = parse_timeout(
                line,
                raw,
                state.destination_ip.as_deref(),
                state.sent_bytes.as_deref(),
                state.pattern.as_deref(),
            ) {
                responses.push(batch_response(timeout));
                continue;
            }
            if let Some(kind) = error_type(line) {
                let mut row = Map::new();
                row.insert("type".into(), Value::String(kind.into()));
                if let Some(destination) = &state.destination_ip {
                    row.insert("destination_ip".into(), Value::String(destination.clone()));
                }
                responses.push(Value::Object(row));
                continue;
            }
            if options.bool("ignoreErrors").unwrap_or(false) {
                continue;
            }
            return Err(ScocError::parse(
                "ping",
                Some(index + 1),
                format!("unrecognized ping line: {line}"),
            ));
        }

        let mut object = Map::new();
        if let Some(value) = &state.destination_ip {
            object.insert("destination_ip".into(), Value::String(value.clone()));
        }
        if let Some(value) = &state.sent_bytes {
            object.insert("data_bytes".into(), int_or_string(value, raw));
        }
        object.insert(
            "pattern".into(),
            state
                .pattern
                .as_ref()
                .map_or(Value::Null, |value| Value::String(value.clone())),
        );
        if let Some(value) = &state.destination {
            object.insert("destination".into(), Value::String(value.clone()));
        }
        // JC initializes `duplicates` before parsing the packet counters. Keep
        // that insertion order even though JSON object equality is order-insensitive.
        insert_optional_number(&mut object, "duplicates", &state.duplicates, raw, false);
        insert_optional_number(
            &mut object,
            "packets_transmitted",
            &state.packets_transmitted,
            raw,
            false,
        );
        insert_optional_number(
            &mut object,
            "packets_received",
            &state.packets_received,
            raw,
            false,
        );
        insert_optional_number(&mut object, "corrupted", &state.corrupted, raw, false);
        insert_optional_number(&mut object, "errors", &state.errors, raw, false);
        insert_optional_number(
            &mut object,
            "packet_loss_percent",
            &state.packet_loss_percent,
            raw,
            true,
        );
        insert_optional_number(&mut object, "time_ms", &state.time_ms, raw, false);
        insert_optional_number(
            &mut object,
            "round_trip_ms_min",
            &state.round_trip_ms_min,
            raw,
            true,
        );
        insert_optional_number(
            &mut object,
            "round_trip_ms_avg",
            &state.round_trip_ms_avg,
            raw,
            true,
        );
        insert_optional_number(
            &mut object,
            "round_trip_ms_max",
            &state.round_trip_ms_max,
            raw,
            true,
        );
        insert_optional_number(
            &mut object,
            "round_trip_ms_stddev",
            &state.round_trip_ms_stddev,
            raw,
            true,
        );
        object.insert("responses".into(), Value::Array(responses));
        Ok(Value::Object(object))
    }

    fn stream_parser(
        &self,
        options: &ParseOptions,
    ) -> Result<Box<dyn ScocStreamParser>, ScocError> {
        Ok(Box::new(PingStreamParser {
            lines: LineBuffer::new(),
            state: PingState::default(),
            raw: options.bool("raw").unwrap_or(false),
            ignore_errors: options.bool("ignoreErrors").unwrap_or(false),
            summary_emitted: false,
            line_number: 0,
        }))
    }
}

struct PingStreamParser {
    lines: LineBuffer,
    state: PingState,
    raw: bool,
    ignore_errors: bool,
    summary_emitted: bool,
    line_number: usize,
}

impl PingStreamParser {
    fn error_value(&self, line: &str, message: impl Into<String>) -> Value {
        let mut meta = Map::new();
        meta.insert("success".into(), Value::Bool(false));
        meta.insert("error".into(), Value::String(message.into()));
        meta.insert("line".into(), Value::String(line.into()));
        let mut row = Map::new();
        row.insert("_jc_meta".into(), Value::Object(meta));
        Value::Object(row)
    }

    fn process_line(&mut self, line: &str) -> Result<Vec<Value>, ScocError> {
        self.line_number += 1;
        if line.starts_with("PATTERN: ") {
            self.state.pattern = line.split_once(": ").map(|(_, value)| value.to_string());
            return Ok(Vec::new());
        }
        if let Some((destination, bytes)) = parse_header(line) {
            self.state.destination_ip = Some(destination);
            self.state.sent_bytes = Some(bytes);
            return Ok(Vec::new());
        }
        if line.starts_with("---") {
            self.state.footer = true;
            self.state.destination = line
                .split_whitespace()
                .nth(1)
                .map(|value| value.to_string());
            return Ok(Vec::new());
        }
        if line.trim().is_empty() {
            return Ok(Vec::new());
        }
        if self.state.footer {
            if parse_stats_line(line, &mut self.state) {
                return Ok(Vec::new());
            }
            if parse_rtt_line(line, &mut self.state) {
                self.summary_emitted = true;
                return Ok(vec![add_success_meta(
                    summary_record(&self.state, self.raw),
                    self.ignore_errors,
                )]);
            }
            return Ok(Vec::new());
        }
        if let Some(reply) = parse_reply(
            line,
            self.raw,
            self.state.destination_ip.as_deref(),
            self.state.sent_bytes.as_deref(),
            self.state.pattern.as_deref(),
        ) {
            return Ok(vec![add_success_meta(reply, self.ignore_errors)]);
        }
        if let Some(timeout) = parse_timeout(
            line,
            self.raw,
            self.state.destination_ip.as_deref(),
            self.state.sent_bytes.as_deref(),
            self.state.pattern.as_deref(),
        ) {
            return Ok(vec![add_success_meta(timeout, self.ignore_errors)]);
        }
        if let Some(kind) = error_type(line) {
            let mut row = Map::new();
            row.insert("type".into(), Value::String(kind.into()));
            if let Some(destination) = &self.state.destination_ip {
                row.insert("destination_ip".into(), Value::String(destination.clone()));
            }
            return Ok(vec![add_success_meta(
                Value::Object(row),
                self.ignore_errors,
            )]);
        }
        if self.ignore_errors {
            return Ok(vec![self.error_value(line, "could not parse ping line")]);
        }
        Err(ScocError::parse(
            "ping",
            Some(self.line_number),
            format!("unrecognized ping line: {line}"),
        ))
    }
}

impl ScocStreamParser for PingStreamParser {
    fn push(&mut self, chunk: &[u8]) -> Result<Vec<Value>, ScocError> {
        let lines = self.lines.push(chunk).map_err(|_| ScocError::Utf8 {
            parser: "ping".into(),
        })?;
        let mut output = Vec::new();
        for line in lines {
            output.extend(self.process_line(&line)?);
        }
        Ok(output)
    }

    fn finish(mut self: Box<Self>) -> Result<Vec<Value>, ScocError> {
        let mut output = Vec::new();
        if let Some(line) =
            std::mem::take(&mut self.lines)
                .finish()
                .map_err(|_| ScocError::Utf8 {
                    parser: "ping".into(),
                })?
        {
            output.extend(self.process_line(&line)?);
        }
        if self.state.footer && !self.summary_emitted && self.state.packets_transmitted.is_some() {
            output.push(add_success_meta(
                summary_record(&self.state, self.raw),
                self.ignore_errors,
            ));
        }
        Ok(output)
    }
}
