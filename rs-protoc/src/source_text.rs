use crate::error::Result;

pub struct SourceBuffer {
    storage: String,
}

impl<'a> SourceBuffer {
    pub fn new_from_file(filename: &str) -> Result<Self> {
        todo!()
    }
    pub fn new(external: String) -> Result<Self> {
        Ok(SourceBuffer { storage: external })
    }
    pub fn text(&'a self) -> &'a str {
        self.storage.as_str()
    }
}
