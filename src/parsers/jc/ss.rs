use crate::parsers::common::{raw, StaticParser};
use crate::utils::input_to_str;
use crate::{
    OptionSpec, OutputShape, ParseOptions, ParserCapabilities, ParserDescriptor, ParserOutput,
    ParserTag, Platform, ScocError, UpstreamParser,
};
use regex::Regex;
use serde_json::{Map, Value};
const OPTIONS: [OptionSpec; 1] = [OptionSpec::bool("raw", false, "Return JC raw output")];
const PLATFORMS: [Platform; 1] = [Platform::Linux];
const TAGS: [ParserTag; 1] = [ParserTag::Command];
pub static DESCRIPTOR: ParserDescriptor = ParserDescriptor {
    name: "ss",
    aliases: &[],
    description: "`ss` command parser",
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
        standard_name: "ss",
        standard_version: "1.9",
        streaming_name: None,
        streaming_version: None,
    }),
};

const CONTAINS_COLON: [&str; 7] = ["nl", "p_raw", "raw", "udp", "tcp", "v_str", "icmp6"];
const SPACE_HOLD: char = '\u{0}';

struct Patterns {
    users_block: Regex,
    users: Regex,
    opts_field: Regex,
    timer_token: Regex,
}

impl Patterns {
    fn new(parser: &str) -> Result<Self, ScocError> {
        let build = |pattern: &str| {
            Regex::new(pattern).map_err(|error| ScocError::parse(parser, None, error.to_string()))
        };
        Ok(Self {
            users_block: build(r"users:\(\(.*?\)\)(?:\s|$)")?,
            users: build(r#"\("(?P<user>.*?)",pid=(?P<pid>\d+),fd=(?P<fd>\d+)\)"#)?,
            opts_field: build(r"ino:|uid:|sk:|users:|timer:|cgroup:|v6only:")?,
            timer_token: build(r"[a-z0-9.]+")?,
        })
    }

    /// The `users:((...))` block, without the whitespace that terminated it.
    fn users_block_span(&self, text: &str) -> Option<(usize, usize)> {
        let found = self.users_block.find(text)?;
        let mut end = found.end();
        if let Some(last) = text[..end].chars().next_back() {
            if last.is_whitespace() {
                end -= last.len_utf8();
            }
        }
        Some((found.start(), end))
    }
}

fn char_split(text: &str, at: usize) -> (String, String) {
    let chars: Vec<char> = text.chars().collect();
    let at = at.min(chars.len());
    (chars[..at].iter().collect(), chars[at..].iter().collect())
}

/// `jc.utils.convert_to_int`.
fn convert_to_int(text: &str) -> Option<i64> {
    let cleaned: String = text
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '-' || *c == '.')
        .collect();
    cleaned
        .parse::<i64>()
        .ok()
        .or_else(|| cleaned.parse::<f64>().ok().map(|f| f as i64))
}

fn parse_opts(patterns: &Patterns, text: &str, parser: &str) -> Result<Value, ScocError> {
    let mut text = text.replace(SPACE_HOLD, " ");
    let mut opts = Map::new();
    if let Some((start, end)) = patterns.users_block_span(&text) {
        let mut process_id = Map::new();
        for caps in patterns.users.captures_iter(&text[start..end]) {
            let mut record = Map::new();
            record.insert("user".into(), Value::String(caps["user"].into()));
            record.insert("file_descriptor".into(), Value::String(caps["fd"].into()));
            process_id.insert(caps["pid"].to_string(), Value::Object(record));
        }
        opts.insert("process_id".into(), Value::Object(process_id));
        text.replace_range(start..end, "");
    }
    for item in text.split(' ') {
        let item = item
            .replace("ino", "inode_number")
            .replace("sk", "cookie")
            .replace("uid", "uid_number");
        let Some((key, value)) = item.split_once(':') else {
            continue;
        };
        if key == "timer" {
            let tokens = patterns
                .timer_token
                .find_iter(value)
                .map(|token| token.as_str().to_string())
                .collect::<Vec<_>>();
            if tokens.len() < 3 {
                return Err(ScocError::parse(parser, None, "could not parse ss timer"));
            }
            let mut timer = Map::new();
            timer.insert("timer_name".into(), Value::String(tokens[0].clone()));
            timer.insert("expire_time".into(), Value::String(tokens[1].clone()));
            timer.insert("retrans".into(), Value::String(tokens[2].clone()));
            opts.insert(key.into(), Value::Object(timer));
        } else {
            opts.insert(key.into(), Value::String(value.into()));
        }
    }
    Ok(Value::Object(opts))
}

fn split_on_spaces(text: &str, min_run: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut spaces = 0usize;
    for ch in text.chars() {
        if ch == ' ' {
            spaces += 1;
        } else {
            if spaces >= min_run {
                out.push(std::mem::take(&mut current));
            } else {
                current.push_str(&" ".repeat(spaces));
            }
            spaces = 0;
            current.push(ch);
        }
    }
    out.push(current);
    out
}

fn parse(
    d: &'static ParserDescriptor,
    input: &[u8],
    opts: &ParseOptions,
) -> Result<Value, ScocError> {
    let text = input_to_str(d.name, input)?;
    let patterns = Patterns::new(d.name)?;
    let fail = |message: &str| ScocError::parse(d.name, None, message);
    let lines = text.lines().filter(|l| !l.is_empty()).collect::<Vec<_>>();
    let Some(first) = lines.first() else {
        return Ok(Value::Array(vec![]));
    };
    if text.trim().is_empty() {
        return Ok(Value::Array(vec![]));
    }
    let header_text = first.to_lowercase();
    let recv_q_position = header_text
        .find("recv-q")
        .map(|byte| header_text[..byte].chars().count());
    let header_text = header_text
        .replace("netidstate", "netid state")
        .replace("local address:port", "local_address local_port")
        .replace("peer address:port", "peer_address peer_port")
        .replace("portprocess", "port")
        .replace('-', "_");
    let mut header_list = header_text
        .split_whitespace()
        .map(str::to_string)
        .collect::<Vec<_>>();
    let mut extra_opts = false;
    let mut entry_list: Vec<Value> = Vec::new();
    let mut output = Vec::new();

    for entry in &lines[1..] {
        if !entry.starts_with(|c: char| c.is_ascii_whitespace()) {
            let recv_at =
                recv_q_position.unwrap_or_else(|| entry.chars().count().saturating_sub(1));
            let (a, b) = char_split(entry, recv_at);
            let mut entry = format!("{a}  {b}");
            let (a, b) = char_split(&entry, 5);
            entry = format!("{a}  {b}");
            if let Some((start, end)) = patterns.users_block_span(&entry) {
                let held = entry[start..end].replace(' ', &SPACE_HOLD.to_string());
                entry = format!("{}{}{}", &entry[..start], held, &entry[end..]);
            }
            let trimmed = entry.trim();
            let mut fields = split_on_spaces(trimmed, 1);
            if fields.len() > header_list.len() || extra_opts {
                fields = split_on_spaces(trimmed, 2);
                extra_opts = true;
            }
            let get = |fields: &Vec<String>, index: usize| -> Result<String, ScocError> {
                fields
                    .get(index)
                    .cloned()
                    .ok_or_else(|| fail("unexpected ss line layout"))
            };
            if CONTAINS_COLON.contains(&get(&fields, 0)?.as_str()) && get(&fields, 4)?.contains(':')
            {
                let field = get(&fields, 4)?;
                let (address, port) = field.rsplit_once(':').unwrap_or((&field, ""));
                let (address, port) = (address.to_string(), port.to_string());
                fields[4] = address;
                fields.insert(5, port);
            }
            if CONTAINS_COLON.contains(&get(&fields, 0)?.as_str()) && get(&fields, 6)?.contains(':')
            {
                let field = get(&fields, 6)?;
                let (address, port) = field.rsplit_once(':').unwrap_or((&field, ""));
                let (address, port) = (address.to_string(), port.to_string());
                fields[6] = address;
                fields.insert(7, port);
            }
            let opts_start = if header_list.last().map(String::as_str) == Some("opts") {
                header_list.len() - 1
            } else {
                header_list.len()
            };
            if fields.len() > opts_start && patterns.opts_field.is_match(&fields[opts_start]) {
                let joined = fields[opts_start..].join(" ");
                fields.truncate(opts_start);
                fields.push(joined);
            }
            entry_list = fields.into_iter().map(Value::String).collect();
            let last = entry_list
                .last()
                .and_then(Value::as_str)
                .map(str::to_string);
            if let Some(last) = last {
                if patterns.opts_field.is_match(&last) {
                    if header_list.last().map(String::as_str) != Some("opts") {
                        header_list.push("opts".into());
                    }
                    let parsed = parse_opts(&patterns, &last, d.name)?;
                    *entry_list.last_mut().expect("non-empty") = parsed;
                }
            }
        }
        let mut line: Map<String, Value> = Map::new();
        for (key, value) in header_list.iter().zip(entry_list.iter()) {
            line.insert(key.clone(), value.clone());
        }
        let string_of = |line: &Map<String, Value>, key: &str| -> Result<String, ScocError> {
            line.get(key)
                .and_then(Value::as_str)
                .map(str::to_string)
                .ok_or_else(|| fail("unexpected ss line layout"))
        };
        let local_address = string_of(&line, "local_address")?;
        if local_address.contains('%') {
            let (address, interface) = local_address
                .rsplit_once('%')
                .unwrap_or((&local_address, ""));
            let (address, interface) = (address.to_string(), interface.to_string());
            line.insert("local_address".into(), Value::String(address));
            line.insert("interface".into(), Value::String(interface));
        }
        let netid = string_of(&line, "netid")?;
        if netid == "nl" {
            let address = string_of(&line, "local_address")?;
            let port = string_of(&line, "local_port")?;
            line.remove("local_address");
            line.remove("local_port");
            let mut channel = format!("{address}:{port}");
            if let Some((head, pid)) = channel.clone().rsplit_once('/') {
                line.insert("pid".into(), Value::String(pid.into()));
                channel = head.to_string();
            }
            line.insert("channel".into(), Value::String(channel));
        }
        if netid == "p_raw" {
            if let Some(value) = line.remove("local_address") {
                line.insert("link_layer".into(), value);
            }
            if let Some(value) = line.remove("local_port") {
                line.insert("interface".into(), value);
            }
        }
        if !CONTAINS_COLON.contains(&netid.as_str()) {
            if let Some(value) = line.remove("local_address") {
                line.insert("path".into(), value);
            }
        }
        output.push(line);
    }

    if !raw(opts) {
        for line in &mut output {
            for key in ["recv_q", "send_q", "pid"] {
                if let Some(Value::String(text)) = line.get(key).cloned() {
                    line.insert(
                        key.into(),
                        convert_to_int(&text).map_or(Value::Null, Value::from),
                    );
                }
            }
            for side in ["local", "peer"] {
                let key = format!("{side}_port");
                if let Some(Value::String(text)) = line.get(&key).cloned() {
                    if let Some(number) = convert_to_int(&text).filter(|n| *n >= 0) {
                        line.insert(format!("{key}_num"), Value::from(number));
                    }
                }
            }
        }
    }
    Ok(Value::Array(
        output.into_iter().map(Value::Object).collect(),
    ))
}
pub static SS: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
