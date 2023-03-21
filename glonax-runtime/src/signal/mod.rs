use crate::transport::Signal;
use prost::Message;

const QUEUE: &str = "/glonax_signal";

pub trait SignalSource {
    fn fetch(&self, writer: &SignalQueueWriter);
}

pub struct SignalQueueWriter(posixmq::PosixMq);

impl SignalQueueWriter {
    pub fn new() -> Result<Self, std::io::Error> {
        Ok(Self(
            posixmq::OpenOptions::writeonly().create().open(QUEUE)?,
        ))
    }

    pub fn send(&self, signal: Signal) {
        let buf = &signal.encode_to_vec();

        assert!(buf.len() > 0);
        match self.0.send(0, buf) {
            Ok(_) => trace!("Published signal: {}", signal),
            Err(_) => warn!("Failed to publish motion"),
        }
    }
}

pub struct SignalQueueReader(posixmq::PosixMq);

impl SignalQueueReader {
    pub fn new() -> Result<Self, std::io::Error> {
        Ok(Self(
            posixmq::OpenOptions::readonly()
                .nonblocking()
                .max_msg_len(128)
                .capacity(15)
                .create()
                .open(QUEUE)?,
        ))
    }

    // TODO: Clean this up
    pub fn recv(&self) -> Result<Signal, ()> {
        let mut buf = vec![0; self.0.attributes().unwrap_or_default().max_msg_len];

        let (_, len) = self.0.recv(&mut buf).unwrap();

        if len > 0 {
            if let Ok(signal) = Signal::decode(&buf[..len]) {
                Ok(signal)
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }
}

pub struct SignalQueueReaderAsync {
    inner: tokio::io::unix::AsyncFd<posixmq::PosixMq>,
}

impl SignalQueueReaderAsync {
    pub fn new() -> std::io::Result<Self> {
        let q = posixmq::OpenOptions::readonly()
            .nonblocking()
            .max_msg_len(128)
            .capacity(15)
            .create()
            .open(QUEUE)
            .unwrap();

        Ok(Self {
            inner: tokio::io::unix::AsyncFd::new(q)?,
        })
    }

    pub async fn recv(&self) -> Result<Signal, ()> {
        loop {
            let mut guard = self.inner.readable().await.unwrap();

            let mut buf = vec![
                0;
                self.inner
                    .get_ref()
                    .attributes()
                    .unwrap_or_default()
                    .max_msg_len
            ];

            // let (_, len) = self.0.recv(&mut buf).unwrap();

            match guard.try_io(|inner| inner.get_ref().recv(&mut buf)) {
                Ok(result) => {
                    let (_, len) = result.unwrap();

                    if len > 0 {
                        if let Ok(signal) = Signal::decode(&buf[..len]) {
                            return Ok(signal);
                        } else {
                            return Err(());
                        }
                    } else {
                        return Err(());
                    }
                }
                Err(_would_block) => continue,
            }
        }
    }

    // pub async fn write(&self, buf: &[u8]) -> std::io::Result<usize> {
    //     loop {
    //         let mut guard = self.inner.writable().await?;

    //         match guard.try_io(|inner| inner.get_ref().write(buf)) {
    //             Ok(result) => return result,
    //             Err(_would_block) => continue,
    //         }
    //     }
    // }
}

// impl tonic::codegen::futures_core::Stream for SignalQueueReader {
//     type Item = Signal;

//     fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
//         std::task::Poll::Ready(Some(self.recv().unwrap()))
//     }
// }
