use std::{io::Read, usize};

const DEFAULT_BUF_SIZE: usize = 1024;

// TODO: Testcases
// TODO: Replace head, tail with range.
// FUTURE: Avoid the need to copy bytes back to the buffer beginning.
/// The `Ring<R>` struct turns any reader into a ringbuffer.
pub struct Ring<R> {
    inner: R,
    buf: [u8; DEFAULT_BUF_SIZE],
    head: usize,
    tail: usize,
}

impl<R> Ring<R> {
    /// Creates a new `Ring<R>` with a default buffer capacity. The default is currently 1 KB,
    /// but may change in the future.
    pub fn new(inner: R) -> Self {
        Self {
            inner,
            buf: [0; DEFAULT_BUF_SIZE],
            head: 0,
            tail: 0,
        }
    }

    /// Gets a reference to the underlying reader.
    ///
    /// It is inadvisable to directly read from the underlying reader.
    pub fn get_ref(&self) -> &R {
        &self.inner
    }

    /// Gets a mutable reference to the underlying reader.
    ///
    /// It is inadvisable to directly read from the underlying reader.
    pub fn get_mut(&mut self) -> &mut R {
        &mut self.inner
    }

    /// Returns a reference to the internally buffered data.
    pub fn buffer(&self) -> &[u8] {
        &self.buf[self.head..self.tail]
    }

    /// Returns the number of bytes the internal buffer can read. {
    pub fn len(&self) -> usize {
        self.tail - self.head
    }

    /// Returns the number of bytes the internal buffer can hold at once.
    pub fn capacity(&self) -> usize {
        self.buf.len()
    }

    /// Check if the internal buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.tail == 0
    }

    /// Advance the buffer by how much is still left in the buffer.
    pub fn advance_return(&mut self, left: usize) -> Result<usize, ()> {
        if left <= self.len() {
            self.head = self.tail - left;
            Ok(self.len())
        } else {
            Err(()) // TODO
        }
    }

    /// Advance the buffer by how much was taken.
    pub fn advance(&mut self, taken: usize) -> Result<usize, ()> {
        if taken <= self.len() {
            self.head += taken;
            Ok(self.len())
        } else {
            Err(()) // TODO
        }
    }

    /// Unwraps this `Ring<R>`, returning the underlying reader.
    ///
    /// Note that any leftover data in the internal buffer is lost. Therefore,
    /// a following read from the underlying reader may lead to data loss.
    pub fn into_inner(self) -> R {
        self.inner
    }

    /// Reset the internal header and tail pointers.
    fn reset(&mut self) {
        self.head = 0;
        self.tail = 0;
    }
}

impl<T: Read> Read for Ring<T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.head == self.tail {
            self.reset();
        }
        if self.is_empty() {
            debug_assert_eq!(self.head, self.tail);

            if self.fill_buf()? == 0 {
                return Ok(0);
            }
        }

        if buf.len() > self.len() {
            buf[..self.len()].copy_from_slice(self.buffer());
            Ok(self.len())
        } else {
            buf.copy_from_slice(&self.buffer()[..buf.len()]);
            Ok(buf.len())
        }
    }
}

impl<T: Read> Ring<T> {
    pub fn fill_buf(&mut self) -> std::io::Result<usize> {
        if self.head > 0 && self.tail > 0 {
            debug_assert!(self.head < self.tail);

            let buf_len = self.len();
            self.buf.copy_within(self.head..self.tail, 0);
            self.reset();
            self.tail = buf_len;
        }

        let read = self.inner.read(&mut self.buf[self.tail..])?;
        self.tail += read;
        Ok(read)
    }
}
