use serde_json::{Map, Number, Value};

use crate::parsers::common::{raw, StaticParser};
use crate::utils::{input_to_str, to_f64, to_i64};
use crate::{
    OptionSpec, OutputShape, ParseOptions, ParserCapabilities, ParserDescriptor, ParserOutput,
    ParserTag, Platform, ScocError, UpstreamParser,
};

const OPTIONS: [OptionSpec; 1] = [OptionSpec::bool("raw", false, "Return JC raw output")];
const PLATFORMS: [Platform; 2] = [Platform::Linux, Platform::MacOs];
const TAGS: [ParserTag; 1] = [ParserTag::Command];

pub static DESCRIPTOR: ParserDescriptor = ParserDescriptor {
    name: "uptime",
    aliases: &[],
    description: "`uptime` command parser",
    parser_version: "0.1.0",
    platforms: &PLATFORMS,
    tags: &TAGS,
    output: ParserOutput {
        normalized: OutputShape::Record,
        raw: Some(OutputShape::Record),
        stream_item: None,
    },
    capabilities: ParserCapabilities {
        raw: true,
        streaming: false,
        ignore_errors: false,
    },
    options: &OPTIONS,
    upstream: Some(UpstreamParser {
        standard_name: "uptime",
        standard_version: "1.10",
        streaming_name: None,
        streaming_version: None,
    }),
};

fn float_value(text: &str) -> Value {
    to_f64(text)
        .and_then(Number::from_f64)
        .map_or(Value::Null, Value::Number)
}

fn process_uptime(raw_text: &str, object: &mut Map<String, Value>) {
    let mut days = 0i64;
    let mut hours = 0i64;
    let mut minutes = 0i64;
    if raw_text.contains("min") {
        let words: Vec<&str> = raw_text.split_whitespace().collect();
        if words.len() >= 2 {
            minutes = to_i64(words[words.len() - 2]).unwrap_or(0);
        }
    }
    if let Some(last) = raw_text.split_whitespace().last() {
        if let Some((h, m)) = last.split_once(':') {
            hours = to_i64(h).unwrap_or(0);
            minutes = to_i64(m).unwrap_or(0);
        }
    }
    if raw_text.contains("day") {
        days = raw_text
            .split_whitespace()
            .next()
            .and_then(to_i64)
            .unwrap_or(0);
    }
    object.insert("uptime_days".into(), Value::from(days));
    object.insert("uptime_hours".into(), Value::from(hours));
    object.insert("uptime_minutes".into(), Value::from(minutes));
    object.insert(
        "uptime_total_seconds".into(),
        Value::from(days * 86_400 + hours * 3_600 + minutes * 60),
    );
}

fn parse(
    descriptor: &'static ParserDescriptor,
    input: &[u8],
    options: &ParseOptions,
) -> Result<Value, ScocError> {
    let text = input_to_str(descriptor.name, input)?.trim();
    if text.is_empty() {
        return Ok(Value::Object(Map::new()));
    }
    let Some(load_pos) = text
        .find("load average")
        .or_else(|| text.find("load averages"))
    else {
        return Err(ScocError::parse(
            descriptor.name,
            None,
            "missing load average section",
        ));
    };
    let before = text[..load_pos].trim_end_matches(|c: char| c == ':' || c.is_whitespace());
    let after = text[load_pos..]
        .split_once(':')
        .map(|(_, v)| v)
        .unwrap_or("")
        .trim();
    let loads: Vec<&str> = after.split(',').map(str::trim).collect();

    let mut object = Map::new();
    let before_tokens: Vec<&str> = before.split_whitespace().collect();
    let time = before_tokens
        .first()
        .copied()
        .unwrap_or("")
        .trim_end_matches(',');
    object.insert("time".into(), Value::String(time.to_string()));

    let user_label = before_tokens
        .iter()
        .position(|token| matches!(token.trim_end_matches(','), "user" | "users"));
    let (uptime_end, users) = if let Some(label_index) = user_label {
        let count_index = label_index.saturating_sub(1);
        (count_index, before_tokens.get(count_index).copied())
    } else {
        (before_tokens.len(), None)
    };
    let uptime_start = before_tokens
        .iter()
        .position(|token| *token == "up")
        .map(|index| index + 1)
        .unwrap_or(1);
    let uptime_text = if uptime_start < uptime_end {
        before_tokens[uptime_start..uptime_end]
            .join(" ")
            .trim_end_matches(',')
            .to_string()
    } else {
        String::new()
    };
    object.insert("uptime".into(), Value::String(uptime_text.clone()));
    if let Some(users) = users {
        object.insert(
            "users".into(),
            if raw(options) {
                Value::String(users.into())
            } else {
                Value::from(to_i64(users).unwrap_or(0))
            },
        );
    }
    for (key, value) in ["load_1m", "load_5m", "load_15m"].into_iter().zip(loads) {
        object.insert(
            key.into(),
            if raw(options) {
                Value::String(value.into())
            } else {
                float_value(value)
            },
        );
    }

    if !raw(options) {
        let parts: Vec<&str> = time.split(':').collect();
        object.insert(
            "time_hour".into(),
            parts
                .first()
                .and_then(|v| to_i64(v))
                .map_or(Value::Null, Value::from),
        );
        object.insert(
            "time_minute".into(),
            parts
                .get(1)
                .and_then(|v| to_i64(v))
                .map_or(Value::Null, Value::from),
        );
        object.insert(
            "time_second".into(),
            parts
                .get(2)
                .and_then(|v| to_i64(v))
                .map_or(Value::Null, Value::from),
        );
        process_uptime(&uptime_text, &mut object);
    }
    Ok(Value::Object(object))
}

pub static UPTIME: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
