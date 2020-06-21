#[derive(Clone, Debug, Serialize, Deserialize)]
struct BufferPosition {
    position: usize,
    sequence_position: Vec<usize>,
}

impl BufferPosition {
    #[inline]
    fn is_new(&self) -> bool {
        self.sequence_position.is_empty()
    }

    #[inline]
    fn reset(&mut self, position: usize) {
        self.sequence_position.clear();

        self.position = position;
    }

    #[inline]
    fn update(&mut self, other: &Self) {
        self.position = other.position;

        self.sequence_position.clear();

        self.sequence_position.extend(&other.sequence_position);
    }
}
