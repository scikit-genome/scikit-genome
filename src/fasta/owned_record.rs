use crate::fasta;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct OwnedRecord {
    pub description: Vec<u8>,
    pub sequence: Vec<u8>,
}

impl fasta::record::Record for OwnedRecord {
    #[inline]
    fn description(&self) -> &[u8] {
        &self.description
    }

    #[inline]
    fn sequence(&self) -> &[u8] {
        &self.sequence
    }
}

/// Iterator of `OwnedRecord` that owns the underlying reader
pub struct OwnedRecordIntoIterator<Read: std::io::Read, BufferPolicy = DefaultPolicy> {
    parser: fasta::parser::Parser<Read, BufferPolicy>,
}

impl<Read, BufferPolicy> Iterator for OwnedRecordIntoIterator<Read, BufferPolicy>
where
    BufferPolicy: fasta::buffer_policy::BufferPolicy,
    Read: std::io::Read,
{
    type Item = Result<OwnedRecord, fasta::error::Error>;
    fn next(&mut self) -> Option<Self::Item> {
        self.parser.next().map(|rec| rec.map(|r| r.to_owned_record()))
    }
}
