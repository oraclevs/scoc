use crate::options::OptionSpec;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Platform {
    Linux,
    MacOs,
    Windows,
    FreeBsd,
    Cygwin,
    Aix,
}

impl Platform {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Linux => "linux",
            Self::MacOs => "macos",
            Self::Windows => "windows",
            Self::FreeBsd => "freebsd",
            Self::Cygwin => "cygwin",
            Self::Aix => "aix",
        }
    }

    pub fn current() -> Option<Self> {
        if cfg!(target_os = "linux") {
            Some(Self::Linux)
        } else if cfg!(target_os = "macos") {
            Some(Self::MacOs)
        } else if cfg!(target_os = "windows") {
            Some(Self::Windows)
        } else if cfg!(target_os = "freebsd") {
            Some(Self::FreeBsd)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputShape {
    Scalar,
    Record,
    List,
    Table,
}

impl OutputShape {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Scalar => "Scalar",
            Self::Record => "Record",
            Self::List => "List",
            Self::Table => "Table",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParserOutput {
    pub normalized: OutputShape,
    pub raw: Option<OutputShape>,
    pub stream_item: Option<OutputShape>,
}

impl ParserOutput {
    pub const fn shape(self, raw: bool) -> OutputShape {
        if raw {
            match self.raw {
                Some(shape) => shape,
                None => self.normalized,
            }
        } else {
            self.normalized
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ParserCapabilities {
    pub raw: bool,
    pub streaming: bool,
    pub ignore_errors: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParserTag {
    Command,
    File,
    String,
    Generic,
}

impl ParserTag {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Command => "command",
            Self::File => "file",
            Self::String => "string",
            Self::Generic => "generic",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UpstreamParser {
    pub standard_name: &'static str,
    pub standard_version: &'static str,
    pub streaming_name: Option<&'static str>,
    pub streaming_version: Option<&'static str>,
}

#[derive(Clone, Copy, Debug)]
pub struct ParserDescriptor {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub description: &'static str,
    pub parser_version: &'static str,
    pub platforms: &'static [Platform],
    pub tags: &'static [ParserTag],
    pub output: ParserOutput,
    pub capabilities: ParserCapabilities,
    pub options: &'static [OptionSpec],
    pub upstream: Option<UpstreamParser>,
}

impl ParserDescriptor {
    pub fn supports_platform(&self, platform: Platform) -> bool {
        self.platforms.contains(&platform)
    }
}
