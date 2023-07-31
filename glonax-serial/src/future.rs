use std::io::{Read, Write};

use tokio::io::{self, unix::AsyncFd, AsyncRead, AsyncWrite};

use crate::{BaudRate, FlowControl, Parity, StopBits};

/// A UART-based serial port implementation.
///
/// The port will be closed when the value is dropped.
pub struct Uart {
    inner: AsyncFd<crate::imp::Uart>,
}

impl Uart {
    /// Open an UART device.
    ///
    /// The UART device is openend with the most common settings found on UART
    /// configurations. For more fine grained control use the serial builder.
    ///
    /// ## Errors
    ///
    /// * `NoDevice` if the device could not be opened. This could indicate that the device is
    ///   already in use.
    /// * `InvalidInput` if `port` is not a valid device name.
    /// * `Io` for any other error while opening or initializing the device.
    pub fn open(path: &std::path::Path, baud_rate: BaudRate) -> super::Result<Self> {
        crate::builder::Builder::new(path)
            .unwrap()
            .set_baud_rate(baud_rate)
            .unwrap()
            .set_parity(Parity::ParityNone)
            .set_stop_bits(StopBits::Stop1)
            .set_flow_control(FlowControl::FlowNone)
            .build()
    }

    pub(crate) fn from_impl(value: crate::imp::Uart) -> super::Result<Self> {
        Ok(Self {
            inner: AsyncFd::new(value)?,
        })
    }

    pub async fn try_read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            let mut guard = self.inner.readable_mut().await?;

            match guard.try_io(|inner| inner.get_mut().read(buf)) {
                Ok(result) => return result,
                Err(_would_block) => continue,
            }
        }
    }

    pub async fn try_write(&mut self, buf: &[u8]) -> io::Result<usize> {
        loop {
            let mut guard = self.inner.writable_mut().await?;

            match guard.try_io(|inner| inner.get_mut().write(buf)) {
                Ok(result) => return result,
                Err(_would_block) => continue,
            }
        }
    }
}

impl AsyncRead for Uart {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        loop {
            let mut guard = match self.inner.poll_read_ready_mut(cx)? {
                std::task::Poll::Ready(guard) => guard,
                std::task::Poll::Pending => return std::task::Poll::Pending,
            };

            match guard.try_io(|inner| inner.get_mut().read(buf.initialize_unfilled())) {
                Ok(Ok(size)) => {
                    buf.advance(size);
                    return std::task::Poll::Ready(Ok(()));
                }
                Ok(Err(e)) => break std::task::Poll::Ready(Err(e)),
                Err(_would_block) => continue,
            }
        }
    }
}

impl AsyncWrite for Uart {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        loop {
            let mut guard = match self.inner.poll_write_ready_mut(cx)? {
                std::task::Poll::Ready(guard) => guard,
                std::task::Poll::Pending => return std::task::Poll::Pending,
            };

            match guard.try_io(|inner| inner.get_mut().write(buf)) {
                Ok(result) => return std::task::Poll::Ready(result),
                Err(_would_block) => continue,
            }
        }
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        loop {
            let mut guard = match self.inner.poll_write_ready_mut(cx)? {
                std::task::Poll::Ready(guard) => guard,
                std::task::Poll::Pending => return std::task::Poll::Pending,
            };

            match guard.try_io(|inner| inner.get_mut().flush()) {
                Ok(_) => return std::task::Poll::Ready(Ok(())),
                Err(_would_block) => continue,
            }
        }
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        Ok(()).into()
    }
}
