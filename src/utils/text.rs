use crate::ScocError;

pub fn input_to_str<'a>(parser: &str, input: &'a [u8]) -> Result<&'a str, ScocError> {
    std::str::from_utf8(input).map_err(|_| ScocError::Utf8 {
        parser: parser.to_string(),
    })
}

#[derive(Debug, Default)]
pub struct LineBuffer {
    pending: Vec<u8>,
}

impl LineBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, chunk: &[u8]) -> Result<Vec<String>, std::str::Utf8Error> {
        self.pending.extend_from_slice(chunk);
        let mut lines = Vec::new();
        let mut consumed = 0usize;
        while let Some(relative) = self.pending[consumed..]
            .iter()
            .position(|byte| *byte == b'\n')
        {
            let end = consumed + relative;
            let mut line = std::str::from_utf8(&self.pending[consumed..end])?.to_string();
            if line.ends_with('\r') {
                line.pop();
            }
            lines.push(line);
            consumed = end + 1;
        }
        if consumed > 0 {
            self.pending.drain(..consumed);
        }
        Ok(lines)
    }

    pub fn finish(mut self) -> Result<Option<String>, std::str::Utf8Error> {
        if self.pending.is_empty() {
            return Ok(None);
        }
        let mut line = std::str::from_utf8(&self.pending)?.to_string();
        if line.ends_with('\r') {
            line.pop();
        }
        self.pending.clear();
        Ok(Some(line))
    }
}
