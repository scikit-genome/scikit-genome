#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    InvalidStart {
        line: usize,
        found: u8,
    },
    BufferLimit,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::Io(ref e) => e.fmt(f),
            Error::InvalidStart { line, found } => write!(
                f,
                "FASTA parse error: expected '>' but found '{}' at file start, line {}.",
                (found as char).escape_default(),
                line
            ),
            Error::BufferLimit => write!(f, "FASTA parse error: buffer limit reached."),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::Io(e)
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Io(ref e) => e.description(),
            Error::InvalidStart { .. } => "invalid record start",
            Error::BufferLimit => "buffer limit reached",
        }
    }
}
