use crate::fasta::buffer_policy::{StandardPolicy, BufferPolicy};
use crate::fasta::position::Position;
use std::fs::File;
use std::path::Path;

pub struct Parser<Reader: std::io::Read, Policy = StandardPolicy> {
    buffer_policy: Policy,
    buffer_position: BufferPosition,
    buffer_reader: buf_redux::BufReader<Reader>,
    finished: bool,
    position: Position,
    search_position: usize,
}

impl<Reader> Parser<Reader, StandardPolicy>
where
    Reader: std::io::Read,
{
    /// Creates a new reader with the default buffer size of 64 KiB
    ///
    /// # Example:
    ///
    /// ```
    /// use seq_io::fasta::{Reader,Record};
    /// let fasta = b">id\nSEQUENCE";
    ///
    /// let mut reader = Reader::new(&fasta[..]);
    /// let record = reader.next().unwrap().unwrap();
    /// assert_eq!(record.id(), Ok("id"))
    /// ```
    #[inline]
    pub fn new(reader: Reader) -> Parser<Reader, StandardPolicy> {
        Parser::with_capacity(reader, BUFFER_SIZE)
    }

    /// Creates a new reader with a given buffer capacity. The minimum allowed
    /// capacity is 3.
    #[inline]
    pub fn with_capacity(reader: Reader, capacity: usize) -> Parser<Reader, DefaultPolicy> {
        assert!(capacity >= 3);
        Parser {
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

impl Parser<File, DefaultPolicy> {
    /// Creates a reader from a file path.
    ///
    /// # Example:
    ///
    /// ```no_run
    /// use seq_io::fasta::Reader;
    ///
    /// let mut reader = Reader::from_path("seqs.fasta").unwrap();
    ///
    /// // (... do something with the reader)
    /// ```
    #[inline]
    pub fn from_path<P: AsRef<Path>>(path: P) -> io::Result<Parser<File>> {
        File::open(path).map(Parser::new)
    }
}

impl<R, P> Parser<R, P>
where
    R: std::io::Read,
    P: BufferPolicy,
{
    /// Returns a reader with the given buffer policy applied
    #[inline]
    pub fn set_policy<T: BufferPolicy>(self, policy: T) -> Parser<R, T> {
        Parser {
            buffer_reader: self.buffer_reader,
            buffer_position: self.buffer_position,
            position: self.position,
            search_position: self.search_position,
            finished: self.finished,
            buffer_policy: policy,
        }
    }

    /// Returns the `BufPolicy` of the reader
    #[inline]
    pub fn policy(&self) -> &P {
        &self.buffer_policy
    }

    /// Searches the next FASTA record and returns a [RefRecord](struct.RefRecord.html) that
    /// borrows its data from the underlying buffer of this reader.
    ///
    /// # Example:
    ///
    /// ```no_run
    /// use seq_io::fasta::{Reader,Record};
    ///
    /// let mut reader = Reader::from_path("seqs.fasta").unwrap();
    ///
    /// while let Some(record) = reader.next() {
    ///     let record = record.unwrap();
    ///     println!("{}", record.id().unwrap());
    /// }
    /// ```
    pub fn next(&mut self) -> Option<Result<RefRecord, Error>> {
        if self.finished || !self.initialized() && !try_opt!(self.init()) {
            return None;
        }

        if !self.buffer_position.is_new() {
            self.next_position();
        }

        if !try_opt!(self.search()) && !try_opt!(self.next_complete()) {
            return None;
        }

        Some(Ok(RefRecord {
            buffer: self.get_buf(),
            buffer_position: &self.buffer_position,
        }))
    }

    /// Updates a [RecordSet](struct.RecordSet.html) with new data. The contents of the internal
    /// buffer are just copied over to the record set and the positions of all records are found.
    /// Old data will be erased. Returns `None` if the input reached its end.
    pub fn read_record_set(&mut self, rset: &mut RecordSet) -> Option<Result<(), Error>> {
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

        // copy buffer AFTER call to next_complete (initialization of buffer is done there)
        rset.buffer.clear();
        rset.buffer.extend(self.get_buf());

        // Update records that are already in the positions vector
        let mut n = 0;
        for pos in &mut rset.positions {
            n += 1;
            pos.update(&self.buffer_position);

            self.next_position();
            if self.finished || !try_opt!(self.search()) {
                rset.count = n;
                return Some(Ok(()));
            }
        }

        // Add more positions if necessary
        loop {
            n += 1;
            rset.positions.push(self.buffer_position.clone());

            self.next_position();
            if self.finished || !try_opt!(self.search()) {
                rset.count = n;
                return Some(Ok(()));
            }
        }
    }

    // Sets starting points for next position
    #[inline]
    fn next_position(&mut self) {
        self.position.line += self.buffer_position.sequence_position.len() as u64;
        self.position.byte += (self.search_position - self.buffer_position.position) as u64;
        self.buffer_position.position = self.search_position;
        self.buffer_position.sequence_position.clear();
    }

    #[inline(always)]
    fn get_buf(&self) -> &[u8] {
        self.buffer_reader.buffer()
    }

    #[inline(always)]
    fn initialized(&self) -> bool {
        self.position.line != 0
    }

    // moves to the first record positon, ignoring newline characters
    fn init(&mut self) -> Result<bool, Error> {
        if let Some((line_num, pos, byte)) = self.first_byte()? {
            if byte == b'>' {
                self.buffer_position.position = pos;
                self.position.byte = pos as u64;
                self.position.line = line_num as u64;
                self.search_position = pos + 1;
                return Ok(true);
            } else {
                self.finished = true;
                return Err(Error::InvalidStart {
                    line: line_num,
                    found: byte,
                });
            }
        }
        self.finished = true;
        Ok(false)
    }

    fn first_byte(&mut self) -> Result<Option<(usize, usize, u8)>, Error> {
        let mut line_num = 0;

        while fill_buf(&mut self.buffer_reader)? > 0 {
            let mut pos = 0;
            let mut last_line_len = 0;
            for line in self.get_buf().split(|b| *b == b'\n') {
                line_num += 1;
                if !line.is_empty() && line != b"\r" {
                    return Ok(Some((line_num, pos, line[0])));
                }
                pos += line.len() + 1;
                last_line_len = line.len();
            }
            // If an orphan '\r' is found at the end of the buffer,
            // we need to move it to the start and re-search the line
            self.buffer_reader.consume(pos - 1 - last_line_len);
            self.buffer_reader.make_room();
        }
        Ok(None)
    }

    /// Finds the position of the next record
    /// and returns true if found; false if end of buffer reached.
    #[inline]
    fn search(&mut self) -> Result<bool, Error> {
        if self._search() {
            return Ok(true);
        }

        // nothing found
        if self.get_buf().len() < self.buffer_reader.capacity() {
            // EOF reached, there will be no next record
            self.finished = true;
            self.buffer_position.sequence_position.push(self.search_position);
            return Ok(true);
        }

        Ok(false)
    }

    // returns true if complete position found, false if end of buffer reached.
    #[inline]
    fn _search(&mut self) -> bool {
        let bufsize = self.get_buf().len();

        for pos in Memchr::new(b'\n', &self.buffer_reader.buffer()[self.search_position..]) {
            let pos = self.search_position + pos;
            let next_line_start = pos + 1;

            if next_line_start == bufsize {
                // cannot check next byte -> treat as incomplete
                self.search_position = pos; // make sure last byte is re-searched next time
                return false;
            }

            self.buffer_position.sequence_position.push(pos);
            if self.get_buf()[next_line_start] == b'>' {
                // complete record was found
                self.search_position = next_line_start;
                return true;
            }
        }

        // record end not found
        self.search_position = bufsize;

        false
    }

    /// To be called when the end of the buffer is reached and `next_pos` does not find
    /// the next record. Incomplete bytes will be moved to the start of the buffer.
    /// If the record still doesn't fit in, the buffer will be enlarged.
    /// After calling this function, the position will therefore always be 'complete'.
    /// this function assumes that the buffer was fully searched
    fn next_complete(&mut self) -> Result<bool, Error> {
        loop {
            if self.buffer_position.position == 0 {
                // first record -> buffer too small
                self.grow()?;
            } else {
                // not the first record -> buffer may be big enough
                self.make_room();
            }

            // fill up remaining buffer
            fill_buf(&mut self.buffer_reader)?;

            if self.search()? {
                return Ok(true);
            }
        }
    }

    // grow buffer
    fn grow(&mut self) -> Result<(), Error> {
        let cap = self.buffer_reader.capacity();
        let new_size = self.buffer_policy.grow_to(cap).ok_or(Error::BufferLimit)?;
        let additional = new_size - cap;
        self.buffer_reader.reserve(additional);
        Ok(())
    }

    // move incomplete bytes to start of buffer
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

    /// Returns the current position (useful with `seek()`).
    /// If `next()` has not yet been called, `None` will be returned.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate seq_io;
    /// # fn main() {
    /// use seq_io::fasta::{Reader,Position};
    ///
    /// let fasta = b">id1
    /// ACGT
    /// >id2
    /// TGCA";
    ///
    /// let mut reader = Reader::new(&fasta[..]);
    ///
    /// // skip one record
    /// reader.next().unwrap();
    /// // second position
    /// reader.next().unwrap();
    ///
    /// assert_eq!(reader.position(), Some(&Position::new(3, 10)));
    /// # }
    /// ```
    #[inline]
    pub fn position(&self) -> Option<&Position> {
        if self.buffer_position.is_new() {
            return None;
        }
        Some(&self.position)
    }

    /// Returns a borrowed iterator over all FASTA records. The records
    /// are owned (`OwnedRecord`), this is therefore slower than using
    /// `Reader::next()`.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate seq_io;
    /// # fn main() {
    /// use seq_io::fasta::{Reader,OwnedRecord};
    ///
    /// let fasta = b">id1
    /// ACGT
    /// >id2
    /// TGCA";
    ///
    /// let mut reader = Reader::new(&fasta[..]);
    ///
    /// let records: Result<Vec<_>, _> = reader
    ///     .records()
    ///     .collect();
    ///
    /// assert_eq!(records.unwrap(),
    ///     vec![
    ///         OwnedRecord {head: b"id1".to_vec(), seq: b"ACGT".to_vec()},
    ///         OwnedRecord {head: b"id2".to_vec(), seq: b"TGCA".to_vec()}
    ///     ]
    /// );
    /// # }
    /// ```
    pub fn records(&mut self) -> RecordIterator<R, P> {
        RecordIterator { rdr: self }
    }

    /// Returns an iterator over all FASTA records like `Reader::records()`,
    /// but with the difference that it owns the underlying reader.
    pub fn into_records(self) -> RecordsIntoIter<R, P> {
        RecordsIntoIter { rdr: self }
    }
}

impl<R, P> Parser<R, P> where R: io::Read + Seek, P: BufferPolicy, {
    /// Seeks to a specified position.  Keeps the underyling buffer if the seek position is
    /// found within it, otherwise it has to be discarded.
    /// If an error was returned before, seeking to that position will return the same error.
    /// The same is not always true with `None`. If there is no newline character at the end of the
    /// file, the last record will be returned instead of `None`.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate seq_io;
    /// # fn main() {
    /// use seq_io::fasta::{Reader,Position,OwnedRecord};
    /// use std::io::Cursor;
    ///
    /// let fasta = b">id1
    /// ACGT
    /// >id2
    /// TGCA";
    ///
    /// let mut cursor = Cursor::new(&fasta[..]);
    /// let mut reader = Reader::new(cursor);
    ///
    /// // read the first record and get its position
    /// let record1 = reader.next().unwrap().unwrap().to_owned_record();
    /// let pos1 = reader.position().unwrap().to_owned();
    ///
    /// // read the second record
    /// reader.next().unwrap().unwrap();
    ///
    /// // now seek to position of first record
    /// reader.seek(&pos1);
    /// assert_eq!(reader.next().unwrap().unwrap().to_owned_record(), record1);
    /// # }
    /// ```
    pub fn seek(&mut self, pos: &Position) -> Result<(), Error> {
        self.finished = false;
        let diff = pos.byte as i64 - self.position.byte as i64;
        let rel_pos = self.buffer_position.position as i64 + diff;
        if rel_pos >= 0 && rel_pos < (self.get_buf().len() as i64) {
            // position reachable within buffer -> no actual seeking necessary
            self.search_position = rel_pos as usize;
            self.buffer_position.reset(rel_pos as usize);
            self.position = pos.clone();
            return Ok(());
        }
        self.position = pos.clone();
        self.search_position = 0;
        self.buffer_reader.seek(io::SeekFrom::Start(pos.byte))?;
        fill_buf(&mut self.buffer_reader)?;
        self.buffer_position.reset(0);
        Ok(())
    }
}
