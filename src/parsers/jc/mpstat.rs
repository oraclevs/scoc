use crate::parsers::shared::generic::{GenericKind, GenericParser};
use crate::{
    OptionSpec, OutputShape, ParserCapabilities, ParserDescriptor, ParserOutput, ParserTag,
    Platform, UpstreamParser,
};

const OPTIONS: [OptionSpec; 1] = [OptionSpec::bool(
    "raw",
    false,
    "Return raw-compatible output where supported",
)];
const PLATFORMS: [Platform; 2] = [Platform::Linux, Platform::MacOs];
const TAGS: [ParserTag; 1] = [ParserTag::Command];

pub static DESCRIPTOR: ParserDescriptor = ParserDescriptor {
    name: "mpstat",
    aliases: &["mpstat-s"],
    description: "JC 1.26.0 target `mpstat` parser",
    parser_version: "0.2.0",
    platforms: &PLATFORMS,
    tags: &TAGS,
    output: ParserOutput {
        normalized: OutputShape::Table,
        raw: Some(OutputShape::Table),
        stream_item: Some(OutputShape::Record),
    },
    capabilities: ParserCapabilities {
        raw: true,
        streaming: true,
        ignore_errors: false,
    },
    options: &OPTIONS,
    upstream: Some(UpstreamParser {
        standard_name: "mpstat",
        standard_version: "unknown-pinned",
        streaming_name: Some("mpstat-s"),
        streaming_version: Some("unknown-pinned"),
    }),
};

pub(crate) static PARSER: GenericParser = GenericParser::new(&DESCRIPTOR, GenericKind::Table);
