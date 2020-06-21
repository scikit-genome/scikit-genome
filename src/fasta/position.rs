#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    pub(crate) line: u64,
    pub(crate) byte: u64,
}

impl Position {
    pub fn new(line: u64, byte: u64) -> Position {
        Position { line, byte }
    }

    /// Line number (starting with 1)
    pub fn line(&self) -> u64 {
        self.line
    }

    /// Byte offset within the file
    pub fn byte(&self) -> u64 {
        self.byte
    }
}
