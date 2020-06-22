use std::io::BufRead;
use std::io::Seek;

use memchr::Memchr;

use crate::io::fasta::buffer_policy::{BufferPolicy, StandardPolicy};
use crate::io::fasta::buffer_position::BufferPosition;
use crate::io::fasta::error::Error;
use crate::io::fasta::sequence::{BufferedSequence, BufferedSequenceSet, RecordIterator, SequenceIterator};

const BUFFER_SIZE: usize = 64 * 1024;

macro_rules! try_opt {
    ($expr: expr) => {
        match $expr {
            Ok(item) => item,
            Err(e) => return Some(Err(::std::convert::From::from(e))),
        }
    };
}

fn fill<R>(reader: &mut buf_redux::BufReader<R, buf_redux::policy::StdPolicy>) -> std::io::Result<usize> where R: std::io::Read {
    let size = reader.buffer().len();

    let mut read = 0;

    while size + read < reader.capacity() {
        match reader.read_into_buf() {
            Ok(0) => break,
            Ok(n) => read += n,
            Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }
    Ok(read)
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    pub index: u64,
    pub offset: u64,
}

impl Position {
    pub fn new(index: u64, offset: u64) -> Position {
        Position { index, offset }
    }

    pub fn index(&self) -> u64 {
        self.index
    }

    pub fn offset(&self) -> u64 {
        self.offset
    }
}

pub struct Reader<R: std::io::Read, P = StandardPolicy> {
    pub buffer_policy: P,
    pub buffer_position: BufferPosition,
    pub buffer_reader: buf_redux::BufReader<R>,
    pub finished: bool,
    pub position: Position,
    pub search_position: usize,
}

impl<R> Reader<R, StandardPolicy>
where
    R: std::io::Read,
{
    #[inline]
    pub fn new(reader: R) -> Reader<R, StandardPolicy> {
        Reader::with_capacity(reader, BUFFER_SIZE)
    }

    #[inline]
    pub fn with_capacity(reader: R, capacity: usize) -> Reader<R, StandardPolicy> {
        assert!(capacity >= 3);
        Reader {
            buffer_reader: buf_redux::BufReader::with_capacity(capacity, reader),
            buffer_position: BufferPosition {
                position: 0,
                sequence_position: Vec::with_capacity(1),
            },
            position: Position::new(0, 0),
            search_position: 0,
            finished: false,
            buffer_policy: StandardPolicy,
        }
    }
}

impl Reader<std::fs::File, StandardPolicy> {
    #[inline]
    pub fn from_path<P: AsRef<std::path::Path>>(path: P) -> std::io::Result<Reader<std::fs::File>> {
        std::fs::File::open(path).map(Reader::new)
    }
}

impl<R, P> Reader<R, P>
where
    R: std::io::Read,
    P: BufferPolicy,
{
    #[inline]
    pub fn set_policy<T: BufferPolicy>(self, policy: T) -> Reader<R, T> {
        Reader {
            buffer_reader: self.buffer_reader,
            buffer_position: self.buffer_position,
            position: self.position,
            search_position: self.search_position,
            finished: self.finished,
            buffer_policy: policy,
        }
    }

    #[inline]
    pub fn policy(&self) -> &P {
        &self.buffer_policy
    }

    pub fn next(&mut self) -> Option<Result<BufferedSequence, Error>> {
        if self.finished || !self.initialized() && !try_opt!(self.init()) {
            return None;
        }

        if !self.buffer_position.is_new() {
            self.next_position();
        }

        if !try_opt!(self.search()) && !try_opt!(self.next_complete()) {
            return None;
        }

        let sequence = BufferedSequence {
            buffer: self.buffer(),
            buffer_position: &self.buffer_position,
        };

        Some(Ok(sequence))
    }

    pub fn read_record_set(&mut self, set: &mut BufferedSequenceSet) -> Option<Result<(), Error>> {
        if self.finished {
            return None;
        }

        if !self.initialized() {
            if !try_opt!(self.init()) {
                return None;
            }
            if !try_opt!(self.search()) {
                return Some(Ok(()));
            }
        } else if !try_opt!(self.next_complete()) {
            return None;
        };

        set.buffer.clear();

        set.buffer.extend(self.buffer());

        let mut size = 0;

        for buffer_position in &mut set.positions {
            size += 1;

            buffer_position.update(&self.buffer_position);

            self.next_position();

            if self.finished || !try_opt!(self.search()) {
                set.count = size;

                return Some(Ok(()));
            }
        }

        loop {
            size += 1;
            set.positions.push(self.buffer_position.clone());

            self.next_position();

            if self.finished || !try_opt!(self.search()) {
                set.count = size;

                return Some(Ok(()));
            }
        }
    }

    #[inline]
    fn next_position(&mut self) {
        self.position.index += self.buffer_position.sequence_position.len() as u64;

        self.position.offset += (self.search_position - self.buffer_position.position) as u64;

        self.buffer_position.position = self.search_position;

        self.buffer_position.sequence_position.clear();
    }

    #[inline(always)]
    fn buffer(&self) -> &[u8] {
        self.buffer_reader.buffer()
    }

    #[inline(always)]
    fn initialized(&self) -> bool {
        self.position.index != 0
    }

    fn init(&mut self) -> Result<bool, Error> {
        if let Some((index, position, offset)) = self.first()? {
            return if offset == b'>' {
                self.buffer_position.position = position;
                self.position.offset = position as u64;
                self.position.index = index as u64;
                self.search_position = position + 1;
                Ok(true)
            } else {
                self.finished = true;
                Err(Error::InvalidStart {
                    line: index,
                    found: offset,
                })
            }
        }

        self.finished = true;

        Ok(false)
    }

    fn first(&mut self) -> Result<Option<(usize, usize, u8)>, Error> {
        let mut index = 0;

        while fill(&mut self.buffer_reader)? > 0 {
            let mut position = 0;
            let mut last_line_len = 0;

            for bytes in self.buffer().split(|b| *b == b'\n') {
                index += 1;

                if !bytes.is_empty() && bytes != b"\r" {
                    return Ok(Some((index, position, bytes[0])));
                }

                position += bytes.len() + 1;

                last_line_len = bytes.len();
            }

            self.buffer_reader.consume(position - 1 - last_line_len);

            self.buffer_reader.make_room();
        }

        Ok(None)
    }

    #[inline]
    fn search(&mut self) -> Result<bool, Error> {
        if self._search() {
            return Ok(true);
        }

        if self.buffer().len() < self.buffer_reader.capacity() {
            self.finished = true;

            self.buffer_position.sequence_position.push(self.search_position);

            return Ok(true);
        }

        Ok(false)
    }

    #[inline]
    fn _search(&mut self) -> bool {
        let buffer_size = self.buffer().len();

        for position in Memchr::new(b'\n', &self.buffer_reader.buffer()[self.search_position..]) {
            let position = self.search_position + position;

            let next_line_start = position + 1;

            if next_line_start == buffer_size {
                self.search_position = position;

                return false;
            }

            self.buffer_position.sequence_position.push(position);

            if self.buffer()[next_line_start] == b'>' {
                self.search_position = next_line_start;

                return true;
            }
        }

        self.search_position = buffer_size;

        false
    }

    fn next_complete(&mut self) -> Result<bool, Error> {
        loop {
            if self.buffer_position.position == 0 {
                self.grow()?;
            } else {
                self.make_room();
            }

            fill(&mut self.buffer_reader)?;

            if self.search()? {
                return Ok(true);
            }
        }
    }

    fn grow(&mut self) -> Result<(), Error> {
        let capacity = self.buffer_reader.capacity();

        let size = self.buffer_policy.grow_to(capacity).ok_or(Error::BufferLimit)?;

        self.buffer_reader.reserve(size - capacity);

        Ok(())
    }

    fn make_room(&mut self) {
        let consumed = self.buffer_position.position;

        self.buffer_reader.consume(consumed);

        self.buffer_reader.make_room();

        self.buffer_position.position = 0;

        self.search_position -= consumed;

        for s in &mut self.buffer_position.sequence_position {
            *s -= consumed;
        }
    }

    #[inline]
    pub fn position(&self) -> Option<&Position> {
        if self.buffer_position.is_new() {
            return None;
        }

        Some(&self.position)
    }

    pub fn records(&mut self) -> RecordIterator<R, P> {
        RecordIterator { parser: self }
    }

    pub fn into_records(self) -> SequenceIterator<R, P> {
        SequenceIterator { parser: self }
    }
}

impl<R, P> Reader<R, P> where R: std::io::Read + std::io::Seek, P: BufferPolicy, {
    pub fn seek(&mut self, position: &Position) -> Result<(), Error> {
        self.finished = false;

        let difference = position.offset as i64 - self.position.offset as i64;

        let relative_position = self.buffer_position.position as i64 + difference;

        if relative_position >= 0 && relative_position < (self.buffer().len() as i64) {
            self.search_position = relative_position as usize;

            self.buffer_position.reset(relative_position as usize);

            self.position = position.clone();

            return Ok(());
        }

        self.position = position.clone();

        self.search_position = 0;

        self.buffer_reader.seek(std::io::SeekFrom::Start(position.offset))?;

        fill(&mut self.buffer_reader)?;

        self.buffer_position.reset(0);

        Ok(())
    }
}
