pub(crate) struct DoubleCursor<T> {
    inner: T,
    head: usize,
    tail: usize,
}

impl<T: std::fmt::Debug + AsRef<[u8]>> std::fmt::Debug for DoubleCursor<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DoubleCursor")
            .field("len", &self.len())
            .field("head", &self.head)
            .field("tail", &self.tail)
            .field("inner", &self.buffer())
            .finish()
    }
}

impl<T: AsRef<[u8]>> DoubleCursor<T> {
    pub(crate) fn buffer(&self) -> &[u8] {
        &self.inner.as_ref()[self.head..self.tail]
    }

    #[inline]
    pub fn is_full(&self) -> bool {
        self.tail == self.inner.as_ref().len()
    }
}

impl<T: AsMut<[u8]> + AsRef<[u8]>> DoubleCursor<T> {
    /// Return the largest available slice.
    ///
    /// The returned slice is available for writing. After the bytes are written
    /// to the allocated buffer call `fill` with the number of bytes written.
    pub(crate) fn allocate(&mut self) -> &mut [u8] {
        if self.is_full() {
            self.trim();
        }

        &mut self.inner.as_mut()[self.tail..]
    }

    fn trim(&mut self) {
        if self.head > 0 {
            if self.is_empty() {
                self.reset();
            } else {
                let len = self.len();
                self.inner.as_mut().copy_within(self.head..self.tail, 0);
                self.reset();
                self.tail = len;
            }
        }
    }
}

impl<T> DoubleCursor<T> {
    /// Construct new double cursor.
    pub(crate) fn new(inner: T) -> Self {
        Self {
            inner,
            head: 0,
            tail: 0,
        }
    }

    /// Reset buffer.
    #[inline]
    fn reset(&mut self) {
        self.head = 0;
        self.tail = 0;
    }

    /// Extend the buffer with len bytes.
    #[inline]
    pub fn fill(&mut self, len: usize) {
        self.tail += len;
    }

    /// Consume bytes from buffer.
    ///
    /// This method must be called when data is read from the buffer and
    /// the has been processed. When consume is called the internal head
    /// is moved forward. Any consequtive `buffer` calls will not return
    /// the consumed data again.
    ///
    /// If the buffer is empty after the just consumed bytes it is reset.
    /// This does not have any impact on read/write operations.
    pub fn consume(&mut self, len: usize) {
        self.head += len;

        if self.head > 0 && self.is_empty() {
            self.reset();
        }
    }

    /// Return the length of the current available buffer.
    #[inline]
    pub fn len(&self) -> usize {
        self.tail - self.head
    }

    /// Test if buffer is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
