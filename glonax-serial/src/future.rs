use std::io::{Read, Write};

use tokio::io::{self, unix::AsyncFd};

pub struct AsyncUart {
    inner: AsyncFd<crate::Uart>,
}

impl AsyncUart {
    pub fn new(uart: crate::Uart) -> io::Result<Self> {
        Ok(Self {
            inner: AsyncFd::new(uart)?,
        })
    }

    pub async fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            let mut guard = self.inner.readable_mut().await?;

            match guard.try_io(|inner| inner.get_mut().read(buf)) {
                Ok(result) => return result,
                Err(_would_block) => continue,
            }
        }
    }

    pub async fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        loop {
            let mut guard = self.inner.writable_mut().await?;

            match guard.try_io(|inner| inner.get_mut().write(buf)) {
                Ok(result) => return result,
                Err(_would_block) => continue,
            }
        }
    }
}
