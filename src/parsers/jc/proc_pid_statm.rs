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
const PLATFORMS: [Platform; 1] = [Platform::Linux];
const TAGS: [ParserTag; 1] = [ParserTag::Command];

pub static DESCRIPTOR: ParserDescriptor = ParserDescriptor {
    name: "proc-pid-statm",
    aliases: &[],
    description: "JC 1.26.0 target `proc-pid-statm` parser",
    parser_version: "0.2.0",
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
        standard_name: "proc-pid-statm",
        standard_version: "unknown-pinned",
        streaming_name: None,
        streaming_version: None,
    }),
};

pub(crate) static PARSER: GenericParser = GenericParser::new(&DESCRIPTOR, GenericKind::Auto);
