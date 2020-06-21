use crate::fasta;

pub trait Record {
    fn description(&self) -> &[u8];
    fn sequence(&self) -> &[u8];
    fn id_bytes(&self) -> &[u8] {
        self.description().split(|b| *b == b' ').next().unwrap()
    }
    fn id(&self) -> Result<&str, std::str::Utf8Error> {
        str::from_utf8(self.id_bytes())
    }
    fn desc_bytes(&self) -> Option<&[u8]> {
        self.description().splitn(2, |b| *b == b' ').nth(1)
    }

    /// Return the description of the record as string slice, if present. Otherwise, `None` is returned.
    fn desc(&self) -> Option<Result<&str, std::str::Utf8Error>> {
        self.desc_bytes().map(str::from_utf8)
    }

    /// Return both the ID and the description of the record (if present)
    /// This should be faster than calling `id()` and `desc()` separately.
    fn id_desc_bytes(&self) -> (&[u8], Option<&[u8]>) {
        let mut description = self.description().splitn(2, |c| *c == b' ');

        (description.next().unwrap(), description.next())
    }

    fn id_desc(&self) -> Result<(&str, Option<&str>), std::str::Utf8Error> {
        let mut description = str::from_utf8(self.description())?.splitn(2, ' ');

        Ok((description.next().unwrap(), description.next()))
    }
}

pub struct RecordIterator<'a, Read, BufferPolicy = DefaultPolicy>
where
    BufferPolicy: 'a,
    Read: std::io::Read + 'a,
{
    parser: &'a mut fasta::parser::Parser<Read, BufferPolicy>,
}

impl<'a, Read, BufferPolicy> Iterator for RecordIterator<'a, Read, BufferPolicy>
where
    BufferPolicy: fasta::buffer_policy::BufferPolicy + 'a,
    Read: std::io::Read + 'a,
{
    type Item = Result<fasta::owned_record::OwnedRecord, fasta::error::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.parser.next().map(|rec| rec.map(|r| r.to_owned_record()))
    }
}
