use crate::descriptor::Platform;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ScocError {
    #[error("unknown SCOC parser `{name}`")]
    UnknownParser { name: String },

    #[error("parser `{parser}` does not support streaming")]
    StreamingUnsupported { parser: String },

    #[error("parser `{parser}` is not supported on {platform:?}")]
    UnsupportedPlatform { parser: String, platform: Platform },

    #[error("invalid option `{option}` for parser `{parser}`: {reason}")]
    InvalidOption {
        parser: String,
        option: String,
        reason: String,
    },

    #[error("invalid input for parser `{parser}`: {message}")]
    InvalidInput { parser: String, message: String },

    #[error("failed to parse `{parser}`: {message}")]
    Parse {
        parser: String,
        line: Option<usize>,
        message: String,
    },

    #[error("unsupported output variant for parser `{parser}`: {variant}")]
    UnsupportedVariant { parser: String, variant: String },

    #[error("internal SCOC error: {message}")]
    Internal { message: String },

    #[error("input for parser `{parser}` is not valid UTF-8")]
    Utf8 { parser: String },
}

impl ScocError {
    pub fn parse(
        parser: impl Into<String>,
        line: Option<usize>,
        message: impl Into<String>,
    ) -> Self {
        Self::Parse {
            parser: parser.into(),
            line,
            message: message.into(),
        }
    }

    pub fn unsupported_variant(parser: impl Into<String>, variant: impl Into<String>) -> Self {
        Self::UnsupportedVariant {
            parser: parser.into(),
            variant: variant.into(),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }
}
