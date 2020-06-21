pub struct SequenceIterator<'a> {
    count: usize,
    position_iterator: iter::Zip<slice::Iter<'a, usize>, iter::Skip<slice::Iter<'a, usize>>>,
    sequences: &'a [u8],
}

impl<'a> Iterator for SequenceIterator<'a> {
    type Item = &'a [u8];

    #[inline]
    fn next(&mut self) -> Option<&'a [u8]> {
        self.position_iterator
            .next()
            .map(|(start, next_start)| trim_cr(&self.sequences[*start + 1..*next_start]))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let l = self.len();
        (l, Some(l))
    }
}

impl<'a> DoubleEndedIterator for SequenceIterator<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<&'a [u8]> {
        self.position_iterator
            .next_back()
            .map(|(start, next_start)| trim_cr(&self.sequences[*start + 1..*next_start]))
    }
}

impl<'a> ExactSizeIterator for SequenceIterator<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.count
    }
}
