struct Ringbuf<T> {
    data: Vec<T>,
    begin: usize,
    end: usize,
}

impl<T> Ringbuf<T> {
    pub fn size(&self) -> usize {
        self.data.len()
    }

    pub fn contiguous(&self) -> bool {
        self.end >= self.begin
    }

    pub fn len(&self) -> usize {
        0
    }
}