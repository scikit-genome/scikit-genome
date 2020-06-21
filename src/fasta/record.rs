use crate::fasta;
use crate::fasta::buffer_policy::StandardPolicy;

pub trait Record {
    fn description(&self) -> &[u8];
    fn sequence(&self) -> &[u8];
}

pub struct RecordIterator<'a, R, P = StandardPolicy>
where
    P: 'a,
    R: std::io::Read + 'a,
{
    pub parser: &'a mut fasta::parser::Parser<R, P>,
}

impl<'a, R, P> Iterator for RecordIterator<'a, R, P>
where
    P: fasta::buffer_policy::BufferPolicy + 'a,
    R: std::io::Read + 'a,
{
    type Item = Result<fasta::owned_record::OwnedRecord, fasta::error::Error>;

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
