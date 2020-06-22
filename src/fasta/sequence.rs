use std::{borrow, io, iter, slice};

use crate::fasta::buffer_policy::{StandardPolicy, BufferPolicy};
use crate::fasta::buffer_position::BufferPosition;
use crate::fasta::reader::Reader;
use crate::fasta::error::Error;

#[inline]
fn trim_carriage_return(line: &[u8]) -> &[u8] {
    if let Some((&b'\r', remaining)) = line.split_last() {
        remaining
    } else {
        line
    }
}

/// A FASTA record that borrows data from a buffer.
#[derive(Clone, Debug)]
pub struct BufferedSequence<'a> {
    pub buffer: &'a [u8],
    pub buffer_position: &'a BufferPosition,
}

impl<'a> Record for BufferedSequence<'a> {
    /// Return the FASTA sequence as byte slice.
    /// Note that this method of `RefRecord` returns
    /// the **raw** sequence, which may contain line breaks.
    /// Use `seq_lines()` to iterate over all lines without
    /// breaks, or use [`full_seq()`](struct.RefRecord.html#method.full_seq)
    /// to access the whole sequence at once.
    #[inline]
    fn data(&self) -> &[u8] {
        if self.buffer_position.sequence_position.len() > 1 {
            let start = *self.buffer_position.sequence_position.first().unwrap() + 1;

            let end = *self.buffer_position.sequence_position.last().unwrap();

            trim_carriage_return(&self.buffer[start..end])
        } else {
            b""
        }
    }

    #[inline]
    fn description(&self) -> &[u8] {
        trim_carriage_return(&self.buffer[self.buffer_position.position + 1..*self.buffer_position.sequence_position.first().unwrap()])
    }
}

impl<'a> BufferedSequence<'a> {
    /// Return an iterator over all sequence lines in the data
    #[inline]
    pub fn seq_lines(&self) -> LineIterator {
        LineIterator {
            data: &self.buffer,
            count: self.buffer_position.sequence_position.len() - 1,
            position_iterator: self
                .buffer_position
                .sequence_position
                .iter()
                .zip(self.buffer_position.sequence_position.iter().skip(1)),
        }
    }

    /// Returns the number of sequence lines.
    /// Equivalent to `self.seq_lines().len()`
    #[inline]
    pub fn num_seq_lines(&self) -> usize {
        self.seq_lines().len()
    }

    /// Returns the full sequence. If the sequence consists of a single line,
    /// then the sequence will be borrowed from the underlying buffer
    /// (equivalent to calling `RefRecord::seq()`). If there are multiple
    /// lines, an owned copy will be created (equivalent to `RefRecord::owned_seq()`).
    pub fn full_seq(&self) -> borrow::Cow<[u8]> {
        if self.num_seq_lines() == 1 {
            // only one line
            self.data().into()
        } else {
            self.owned_seq().into()
        }
    }

    /// Returns the sequence as owned `Vec`. **Note**: This function
    /// must be called in order to obtain a sequence that does not contain
    /// line endings (as returned by `seq()`)
    pub fn owned_seq(&self) -> Vec<u8> {
        let mut seq = Vec::new();
        for segment in self.seq_lines() {
            seq.extend(segment);
        }
        seq
    }

    /// Creates an owned copy of the record.
    pub fn to_owned_record(&self) -> Sequence {
        Sequence {
            description: self.description().to_vec(),
            data: self.owned_seq(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BufferedSequenceSet {
    pub buffer: Vec<u8>,
    pub count: usize,
    pub positions: Vec<BufferPosition>,
}

impl Default for BufferedSequenceSet {
    fn default() -> BufferedSequenceSet {
        BufferedSequenceSet {
            buffer: vec![],
            positions: vec![],
            count: 0,
        }
    }
}

impl<'a> iter::IntoIterator for &'a BufferedSequenceSet {
    type Item = BufferedSequence<'a>;
    type IntoIter = BufferSequenceSetIterator<'a>;
    fn into_iter(self) -> Self::IntoIter {
        BufferSequenceSetIterator {
            buffer: &self.buffer,
            position: self.positions.iter().take(self.count),
        }
    }
}

pub struct BufferSequenceSetIterator<'a> {
    buffer: &'a [u8],
    position: iter::Take<slice::Iter<'a, BufferPosition>>,
}

impl<'a> Iterator for BufferSequenceSetIterator<'a> {
    type Item = BufferedSequence<'a>;

    fn next(&mut self) -> Option<BufferedSequence<'a>> {
        self.position.next().map(|p| BufferedSequence {
            buffer: self.buffer,
            buffer_position: p,
        })
    }
}

pub struct LineIterator<'a> {
    pub count: usize,
    pub position_iterator: iter::Zip<slice::Iter<'a, usize>, iter::Skip<slice::Iter<'a, usize>>>,
    pub data: &'a [u8],
}

impl<'a> Iterator for LineIterator<'a> {
    type Item = &'a [u8];

    #[inline]
    fn next(&mut self) -> Option<&'a [u8]> {
        self.position_iterator
            .next()
            .map(|(start, next_start)| trim_carriage_return(&self.data[*start + 1..*next_start]))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let l = self.len();
        (l, Some(l))
    }
}

impl<'a> DoubleEndedIterator for LineIterator<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<&'a [u8]> {
        self.position_iterator
            .next_back()
            .map(|(start, next_start)| trim_carriage_return(&self.data[*start + 1..*next_start]))
    }
}

impl<'a> ExactSizeIterator for LineIterator<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.count
    }
}

pub trait Record {
    fn data(&self) -> &[u8];
    fn description(&self) -> &[u8];
}

pub struct RecordIterator<'a, R, P = StandardPolicy>
where
    P: 'a,
    R: std::io::Read + 'a,
{
    pub parser: &'a mut Reader<R, P>,
}

impl<'a, R, P> Iterator for RecordIterator<'a, R, P>
where
    P: BufferPolicy + 'a,
    R: io::Read + 'a,
{
    type Item = Result<Sequence, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.parser
            .next()
            .map(|record| {
                record.map(|r| {
                    r.to_owned_record()
                })
            })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Sequence {
    pub data: Vec<u8>,
    pub description: Vec<u8>,
}

impl Record for Sequence {
    #[inline]
    fn data(&self) -> &[u8] {
        &self.data
    }

    #[inline]
    fn description(&self) -> &[u8] {
        &self.description
    }
}

pub struct SequenceIterator<R: io::Read, P = StandardPolicy> {
    pub parser: Reader<R, P>,
}

impl<R, P> Iterator for SequenceIterator<R, P>
where
    P: BufferPolicy,
    R: io::Read,
{
    type Item = Result<Sequence, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        self.parser.next().map(|rec| rec.map(|r| r.to_owned_record()))
    }
}
