use crate::parsers::shared::generic::{GenericKind, GenericParser};
use crate::{
    OptionSpec, OutputShape, ParserCapabilities, ParserDescriptor, ParserOutput, ParserTag,
    Platform,
};

const OPTIONS: [OptionSpec; 1] = [OptionSpec::bool(
    "raw",
    false,
    "Keep human-readable scalar fields without additional normalization",
)];
const PLATFORMS: [Platform; 2] = [Platform::Linux, Platform::MacOs];
const TAGS: [ParserTag; 1] = [ParserTag::Command];

pub static DESCRIPTOR: ParserDescriptor = ParserDescriptor {
    name: "go-version",
    aliases: &[],
    description: "Structured human-readable output parser for `go-version`",
    parser_version: "0.2.0",
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
    upstream: None,
};

pub(crate) static PARSER: GenericParser = GenericParser::new(&DESCRIPTOR, GenericKind::Version);
