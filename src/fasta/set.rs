use crate::fasta::ref_record::RefRecord;
use std::{iter, slice};

/// Set of FASTA records that owns it'P buffer
/// and knows the positions of each record.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Set {
    buffer: Vec<u8>,
    count: usize,
    positions: Vec<BufferPosition>,
}

impl Default for Set {
    fn default() -> Set {
        Set {
            buffer: vec![],
            positions: vec![],
            count: 0,
        }
    }
}

impl<'a> iter::IntoIterator for &'a Set {
    type Item = RefRecord<'a>;
    type IntoIter = SetIterator<'a>;
    fn into_iter(self) -> Self::IntoIter {
        SetIterator {
            buffer: &self.buffer,
            position: self.positions.iter().take(self.count),
        }
    }
}

pub struct SetIterator<'a> {
    buffer: &'a [u8],
    position: iter::Take<slice::Iter<'a, BufferPosition>>,
}

impl<'a> Iterator for SetIterator<'a> {
    type Item = RefRecord<'a>;

    fn next(&mut self) -> Option<RefRecord<'a>> {
        self.position.next().map(|p| RefRecord {
            buffer: self.buffer,
            buffer_position: p,
        })
    }
}
