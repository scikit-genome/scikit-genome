/// FASTA parsing error
#[derive(Debug)]
pub enum Error {
    /// io::Error
    Io(std::io::Error),
    /// First non-empty line does not start with `>`
    InvalidStart {
        /// line number (1-based)
        line: usize,
        /// byte that was found instead
        found: u8,
    },
    /// Size limit of buffer was reached, which happens if `policy::BufPolicy::grow_to()` returned
    /// `None`. This does not happen with the default `struct.DoubleUntil.html` policy.
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
