mod compatibility;
mod descriptor;
mod error;
mod options;
mod parser;
pub mod parsers;
mod registry;
pub mod utils;

pub use compatibility::{CompatibilityBaseline, JC_BASELINE};
pub use descriptor::{
    OutputShape, ParserCapabilities, ParserDescriptor, ParserOutput, ParserTag, Platform,
    UpstreamParser,
};
pub use error::ScocError;
pub use options::{OptionDefault, OptionKind, OptionSpec, OptionValue, ParseOptions};
pub use parser::{ScocParser, ScocStreamParser};
pub use registry::{parse, parser, parsers, registry, stream_parser, ParserRegistry};

pub const fn compatibility_baseline() -> &'static CompatibilityBaseline {
    &JC_BASELINE
}
