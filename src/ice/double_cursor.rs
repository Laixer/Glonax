pub(crate) struct DoubleCursor<T> {
    inner: T,
    head: usize,
    tail: usize,
}

impl<T: AsRef<[u8]>> DoubleCursor<T> {
    pub(crate) fn buffer(&self) -> &[u8] {
        &self.inner.as_ref()[self.head..self.tail]
    }
}

impl<T: AsMut<[u8]>> DoubleCursor<T> {
    pub(crate) fn get_mut_avail(&mut self) -> &mut [u8] {
        &mut self.inner.as_mut()[self.tail..]
    }
}

impl<T> DoubleCursor<T> {
    pub(crate) fn new(inner: T) -> Self {
        Self {
            inner,
            head: 0,
            tail: 0,
        }
    }

    /// Reset the buffer state to empty.
    fn reset(&mut self) {
        self.head = 0;
        self.tail = 0;
    }

    #[inline]
    pub fn fill(&mut self, len: usize) {
        self.tail += len;
    }

    pub fn consume(&mut self, len: usize) {
        self.head += len;
        if self.head > 0 && self.is_empty() {
            self.reset();
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.tail - self.head
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
