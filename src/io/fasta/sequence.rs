use std::{borrow, io, iter, slice};

use crate::io::fasta::buffer_policy::{StandardPolicy, BufferPolicy};
use crate::io::fasta::buffer_position::BufferPosition;
use crate::io::fasta::reader::Reader;
use crate::io::fasta::error::Error;

#[inline]
fn trim_carriage_return(line: &[u8]) -> &[u8] {
    if let Some((&b'\r', remaining)) = line.split_last() {
        remaining
    } else {
        line
    }
}

#[derive(Clone, Debug)]
pub struct BufferedSequence<'a> {
    pub buffer: &'a [u8],
    pub buffer_position: &'a BufferPosition,
}

impl<'a> Record for BufferedSequence<'a> {
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
    #[inline]
    pub fn seq_lines(&self) -> LineIterator {
        LineIterator {
            bytes: &self.buffer,
            size: self.buffer_position.sequence_position.len() - 1,
            position_iterator: self
                .buffer_position
                .sequence_position
                .iter()
                .zip(self.buffer_position.sequence_position.iter().skip(1)),
        }
    }

    #[inline]
    pub fn num_seq_lines(&self) -> usize {
        self.seq_lines().len()
    }

    pub fn full_seq(&self) -> borrow::Cow<[u8]> {
        if self.num_seq_lines() == 1 {
            // only one line
            self.data().into()
        } else {
            self.owned_seq().into()
        }
    }

    pub fn owned_seq(&self) -> Vec<u8> {
        let mut seq = Vec::new();
        for segment in self.seq_lines() {
            seq.extend(segment);
        }
        seq
    }

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
    pub bytes: &'a [u8],
    pub position_iterator: iter::Zip<slice::Iter<'a, usize>, iter::Skip<slice::Iter<'a, usize>>>,
    pub size: usize,
}

impl<'a> Iterator for LineIterator<'a> {
    type Item = &'a [u8];

    #[inline]
    fn next(&mut self) -> Option<&'a [u8]> {
        self.position_iterator
            .next()
            .map(|(start, next_start)| trim_carriage_return(&self.bytes[*start + 1..*next_start]))
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
            .map(|(start, next_start)| {
                trim_carriage_return(&self.bytes[*start + 1..*next_start])
            })
    }
}

impl<'a> ExactSizeIterator for LineIterator<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.size
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
