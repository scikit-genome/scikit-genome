pub trait BufferPolicy {
    fn grow_to(&mut self, current_size: usize) -> Option<usize>;
}

pub struct StandardPolicy;

impl BufferPolicy for StandardPolicy {
    fn grow_to(&mut self, current_size: usize) -> Option<usize> {
        Some(if current_size < 1 << 23 {
            current_size * 2
        } else {
            current_size + (1 << 23)
        })
    }
}
