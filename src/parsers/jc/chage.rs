use crate::parsers::common::{raw, StaticParser};
use crate::utils::{input_to_str, to_i64_value};
use crate::{
    OptionSpec, OutputShape, ParseOptions, ParserCapabilities, ParserDescriptor, ParserOutput,
    ParserTag, Platform, ScocError, UpstreamParser,
};
use serde_json::{Map, Value};
const OPTIONS: [OptionSpec; 1] = [OptionSpec::bool("raw", false, "Return JC raw output")];
const PLATFORMS: [Platform; 1] = [Platform::Linux];
const TAGS: [ParserTag; 1] = [ParserTag::Command];
pub static DESCRIPTOR: ParserDescriptor = ParserDescriptor {
    name: "chage",
    aliases: &[],
    description: "`chage --list` command parser",
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
        standard_name: "chage",
        standard_version: "1.1",
        streaming_name: None,
        streaming_version: None,
    }),
};
fn parse(
    descriptor: &'static ParserDescriptor,
    input: &[u8],
    options: &ParseOptions,
) -> Result<Value, ScocError> {
    let text = input_to_str(descriptor.name, input)?;
    let mut obj = Map::new();
    for (idx, line) in text.lines().filter(|l| !l.trim().is_empty()).enumerate() {
        let Some((k, v)) = line.split_once(':') else {
            return Err(ScocError::parse(
                descriptor.name,
                Some(idx + 1),
                "malformed chage row",
            ));
        };
        let key = match k.trim() {
            "Last password change" => "password_last_changed",
            "Password expires" => "password_expires",
            "Password inactive" => "password_inactive",
            "Account expires" => "account_expires",
            "Minimum number of days between password change" => "min_days_between_password_change",
            "Maximum number of days between password change" => "max_days_between_password_change",
            "Number of days of warning before password expires" => {
                "warning_days_before_password_expires"
            }
            _ => continue,
        };
        obj.insert(key.into(), Value::String(v.trim().into()));
    }
    if !raw(options) {
        for key in [
            "min_days_between_password_change",
            "max_days_between_password_change",
            "warning_days_before_password_expires",
        ] {
            if let Some(Value::String(v)) = obj.get(key).cloned() {
                obj.insert(key.into(), to_i64_value(&v));
            }
        }
    }
    Ok(Value::Object(obj))
}
pub static CHAGE: StaticParser = StaticParser::new(&DESCRIPTOR, parse);
