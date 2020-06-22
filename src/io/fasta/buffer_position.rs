#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BufferPosition {
    pub position: usize,
    pub sequence_position: Vec<usize>,
}

impl BufferPosition {
    #[inline]
    pub fn is_new(&self) -> bool {
        self.sequence_position.is_empty()
    }

    #[inline]
    pub fn reset(&mut self, position: usize) {
        self.sequence_position.clear();

        self.position = position;
    }

    #[inline]
    pub fn update(&mut self, other: &Self) {
        self.position = other.position;

        self.sequence_position.clear();

        self.sequence_position.extend(&other.sequence_position);
    }
}
