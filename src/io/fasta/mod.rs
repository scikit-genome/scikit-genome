//! Efficient FASTA reading and writing
//!
//! # Example
//!
//! This example reads some content, writes it back and compares the output
//! (should be the same):
//!
//! ```
//! use seq_io::fasta::{Reader, Record};
//!
//! let input = b">id1
//! ACGT
//! ACGT
//! >id2
//! TGCA
//! TGCA
//! ";
//!
//! let mut reader = Reader::new(&input[..]);
//! let mut output = vec![];
//!
//! while let Some(record) = reader.next() {
//!     let record = record.expect("Error reading record");
//!     println!("id: {}", record.id().unwrap());
//!     record.write_wrap(&mut output, 4);
//! }
//!
//! assert_eq!(input, output.as_slice());
//! ```
//!
//! # Details on parsing behaviour
//!
//! * The parser handles UNIX (LF) and Windows (CRLF) line endings, but not old
//!   Mac-style (CR) endings. However, FASTA writing currently always uses UNIX
//!   line endings.
//! * Empty lines are allowed anywhere in the file, they will just be ignored.
//!   The first non-empty line must start with `>`, indicating the first header.
//! * Whitespace at the end of header and sequence lines is never removed.
//! * If two consecutive FASTA header lines (starting with `>`) are encountered
//!   without intermediate sequence line, the first record will have an empty
//!   sequence. The same is true if the input ends with a header line.
//! * Empty input will result in `None` being returned immediately by
//!   `fasta::Reader::next()` and in empty iterators for `RecordsIter` /
//!   `RecordsIntoIter`.
//! * Comment lines starting with `;` are not supported.
//!   If at the start of a file, there will be an error, since `>` is expected.
//!   Intermediate comments are appended to the sequence.

pub mod buffer_policy;
pub mod buffer_position;
pub mod error;
pub mod reader;
pub mod sequence;
