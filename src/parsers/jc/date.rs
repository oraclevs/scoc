use chrono::{Datelike, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Timelike, Utc};
use serde_json::{Map, Value};

use crate::parsers::common::StaticParser;
use crate::utils::input_to_str;
use crate::{
    OptionSpec, OutputShape, ParseOptions, ParserCapabilities, ParserDescriptor, ParserOutput,
    ParserTag, Platform, ScocError, UpstreamParser,
};

const OPTIONS: [OptionSpec; 1] = [OptionSpec::bool("raw", false, "Return JC raw output")];
const PLATFORMS: [Platform; 2] = [Platform::Linux, Platform::MacOs];
const TAGS: [ParserTag; 1] = [ParserTag::Command];

pub static DESCRIPTOR: ParserDescriptor = ParserDescriptor {
    name: "date",
    aliases: &[],
    description: "`date` command parser",
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
        standard_name: "date",
        standard_version: "2.6",
        streaming_name: None,
        streaming_version: None,
    }),
};

fn month_number(mon: &str) -> Option<u32> {
    match mon {
        "Jan" => Some(1),
        "Feb" => Some(2),
        "Mar" => Some(3),
        "Apr" => Some(4),
        "May" => Some(5),
        "Jun" => Some(6),
        "Jul" => Some(7),
        "Aug" => Some(8),
        "Sep" => Some(9),
        "Oct" => Some(10),
        "Nov" => Some(11),
        "Dec" => Some(12),
        _ => None,
    }
}

fn parse(
    descriptor: &'static ParserDescriptor,
    input: &[u8],
    _options: &ParseOptions,
) -> Result<Value, ScocError> {
    let text = input_to_str(descriptor.name, input)?.trim();
    if text.is_empty() {
        return Ok(Value::Object(Map::new()));
    }
    let tokens: Vec<&str> = text
        .trim_matches(|c| c == '(' || c == ')')
        .split_whitespace()
        .collect();
    if tokens.len() < 5 {
        return Err(ScocError::parse(
            descriptor.name,
            None,
            "unrecognized date output",
        ));
    }

    // Default GNU/BSD date output is: weekday month day hh:mm:ss timezone year.
    let weekday = tokens[0];
    let month = tokens[1];
    let day = tokens[2]
        .parse::<u32>()
        .map_err(|_| ScocError::parse(descriptor.name, None, "invalid day"))?;
    let time_text = tokens[3];
    let explicit_period = tokens
        .get(4)
        .copied()
        .filter(|value| matches!(*value, "AM" | "PM" | "am" | "pm"));
    let timezone = tokens
        .get(if explicit_period.is_some() { 5 } else { 4 })
        .copied();
    let year_token = tokens
        .iter()
        .rev()
        .find(|value| value.len() == 4 && value.chars().all(|c| c.is_ascii_digit()))
        .ok_or_else(|| ScocError::parse(descriptor.name, None, "missing year"))?;
    let year = year_token
        .parse::<i32>()
        .map_err(|_| ScocError::parse(descriptor.name, None, "invalid year"))?;
    let month_num = month_number(month)
        .ok_or_else(|| ScocError::parse(descriptor.name, None, "invalid month"))?;
    let time = if let Some(period) = explicit_period {
        NaiveTime::parse_from_str(&format!("{time_text} {period}"), "%I:%M:%S %p")
            .or_else(|_| NaiveTime::parse_from_str(&format!("{time_text} {period}"), "%I:%M %p"))
    } else {
        NaiveTime::parse_from_str(time_text, "%H:%M:%S")
            .or_else(|_| NaiveTime::parse_from_str(time_text, "%H:%M"))
    }
    .map_err(|_| ScocError::parse(descriptor.name, None, "invalid time"))?;
    let date = NaiveDate::from_ymd_opt(year, month_num, day)
        .ok_or_else(|| ScocError::parse(descriptor.name, None, "invalid date"))?;
    let naive = NaiveDateTime::new(date, time);
    let utc_aware =
        timezone.is_some_and(|tz| matches!(tz, "UTC" | "GMT" | "Z" | "UTC+0000" | "UTC-0000"));
    let epoch_naive = Local
        .from_local_datetime(&naive)
        .single()
        .map(|dt| dt.timestamp());
    let epoch_utc = utc_aware.then(|| Utc.from_utc_datetime(&naive).timestamp());
    let hour12 = {
        let h = time.hour() % 12;
        if h == 0 {
            12
        } else {
            h
        }
    };
    let period = if time.hour() < 12 { "AM" } else { "PM" };
    let week_of_year = naive.format("%W").to_string().parse::<i64>().unwrap_or(0);
    let iso = if utc_aware {
        Utc.from_utc_datetime(&naive)
            .to_rfc3339_opts(chrono::SecondsFormat::Secs, false)
    } else {
        naive.format("%Y-%m-%dT%H:%M:%S").to_string()
    };

    let mut row = Map::new();
    row.insert("year".into(), Value::from(year));
    row.insert("month".into(), Value::String(month.into()));
    row.insert("month_num".into(), Value::from(month_num));
    row.insert("day".into(), Value::from(day));
    row.insert("weekday".into(), Value::String(weekday.into()));
    row.insert(
        "weekday_num".into(),
        Value::from(naive.weekday().number_from_monday()),
    );
    row.insert("hour".into(), Value::from(hour12));
    row.insert("hour_24".into(), Value::from(time.hour()));
    row.insert("minute".into(), Value::from(time.minute()));
    row.insert("second".into(), Value::from(time.second()));
    row.insert("period".into(), Value::String(period.into()));
    row.insert(
        "timezone".into(),
        timezone.map_or(Value::Null, |tz| Value::String(tz.into())),
    );
    row.insert(
        "utc_offset".into(),
        if utc_aware {
            Value::String("+0000".into())
        } else {
            Value::Null
        },
    );
    row.insert("day_of_year".into(), Value::from(naive.ordinal()));
    row.insert("week_of_year".into(), Value::from(week_of_year));
    row.insert("iso".into(), Value::String(iso));
    row.insert("epoch".into(), epoch_naive.map_or(Value::Null, Value::from));
    row.insert(
        "epoch_utc".into(),
        epoch_utc.map_or(Value::Null, Value::from),
    );
    row.insert("timezone_aware".into(), Value::Bool(utc_aware));
    Ok(Value::Object(row))
}

pub static DATE: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
