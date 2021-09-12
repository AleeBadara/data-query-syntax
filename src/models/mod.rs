use std::collections::HashMap;

pub enum Command {
    LOAD,
    SELECT,
    UMBIGUOUS,
    UNKNOWN,
}

#[derive(Debug, Default)]
pub struct FileData {
    pub data: HashMap<String, Vec<String>>,
}

impl FileData {
    pub fn new() -> Self {
        FileData {
            data: HashMap::new(),
        }
    }
}

pub struct FileName {
    pub name: String,
    pub separator: String,
}
