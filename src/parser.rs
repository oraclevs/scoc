use crate::{ParseOptions, ParserDescriptor, ScocError};

pub trait ScocParser: Send + Sync {
    fn descriptor(&self) -> &'static ParserDescriptor;

    fn parse(&self, input: &[u8], options: &ParseOptions) -> Result<serde_json::Value, ScocError>;

    fn stream_parser(
        &self,
        _options: &ParseOptions,
    ) -> Result<Box<dyn ScocStreamParser>, ScocError> {
        Err(ScocError::StreamingUnsupported {
            parser: self.descriptor().name.to_string(),
        })
    }
}

pub trait ScocStreamParser: Send {
    fn push(&mut self, chunk: &[u8]) -> Result<Vec<serde_json::Value>, ScocError>;
    fn finish(self: Box<Self>) -> Result<Vec<serde_json::Value>, ScocError>;
}
