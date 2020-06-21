use crate::fasta::record::Record;
use crate::trim_cr;
use crate::fasta::sequence_iterator::SequenceIterator;
use std::borrow::Cow;
use crate::fasta::owned_record::OwnedRecord;
use crate::fasta::buffer_position::BufferPosition;

/// A FASTA record that borrows data from a buffer.
#[derive(Debug, Clone)]
pub struct RefRecord<'a> {
    pub buffer: &'a [u8],
    pub buffer_position: &'a BufferPosition,
}

impl<'a> Record for RefRecord<'a> {
    #[inline]
    fn description(&self) -> &[u8] {
        trim_cr(&self.buffer[self.buffer_position.position + 1..*self.buffer_position.sequence_position.first().unwrap()])
    }

    /// Return the FASTA sequence as byte slice.
    /// Note that this method of `RefRecord` returns
    /// the **raw** sequence, which may contain line breaks.
    /// Use `seq_lines()` to iterate over all lines without
    /// breaks, or use [`full_seq()`](struct.RefRecord.html#method.full_seq)
    /// to access the whole sequence at once.
    #[inline]
    fn sequence(&self) -> &[u8] {
        if self.buffer_position.sequence_position.len() > 1 {
            let start = *self.buffer_position.sequence_position.first().unwrap() + 1;

            let end = *self.buffer_position.sequence_position.last().unwrap();

            trim_cr(&self.buffer[start..end])
        } else {
            b""
        }
    }
}

impl<'a> RefRecord<'a> {
    /// Return an iterator over all sequence lines in the data
    #[inline]
    pub fn seq_lines(&self) -> SequenceIterator {
        SequenceIterator {
            sequences: &self.buffer,
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
    pub fn full_seq(&self) -> Cow<[u8]> {
        if self.num_seq_lines() == 1 {
            // only one line
            self.sequence().into()
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
    pub fn to_owned_record(&self) -> OwnedRecord {
        OwnedRecord {
            description: self.description().to_vec(),
            sequence: self.owned_seq(),
        }
    }
}
