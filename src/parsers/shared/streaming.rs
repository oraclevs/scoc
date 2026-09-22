use crate::{ParseOptions, ScocError, ScocStreamParser};
use serde_json::Value;

pub(crate) struct BufferedBatchStream {
    parser: &'static str,
    options: ParseOptions,
    buffer: Vec<u8>,
}

impl BufferedBatchStream {
    pub(crate) fn new(parser: &'static str, options: &ParseOptions) -> Self {
        Self {
            parser,
            options: options.clone(),
            buffer: Vec::new(),
        }
    }
}

impl ScocStreamParser for BufferedBatchStream {
    fn push(&mut self, chunk: &[u8]) -> Result<Vec<Value>, ScocError> {
        self.buffer.extend_from_slice(chunk);
        Ok(Vec::new())
    }

    fn finish(self: Box<Self>) -> Result<Vec<Value>, ScocError> {
        let value = crate::parse(self.parser, &self.buffer, &self.options)?;
        Ok(match value {
            Value::Array(values) => values,
            other => vec![other],
        })
    }
}
