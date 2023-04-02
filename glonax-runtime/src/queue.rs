use crate::transport::Signal;
use prost::Message;

const QUEUE: &str = "/glonax_signal";

pub trait SignalSource {
    fn fetch(&self, writer: &SignalQueueWriter);
}

pub trait SignalSource2 {
    fn fetch(&self, writer: &SignalQueueWriter2);
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

        assert!(!buf.is_empty());
        match self.0.send(0, buf) {
            Ok(_) => trace!("Published signal: {}", signal),
            Err(_) => warn!("Failed to publish motion"),
        }
    }
}

pub struct SignalQueueWriter2(tokio::sync::broadcast::Sender<Signal>);

impl SignalQueueWriter2 {
    pub fn new(sender: tokio::sync::broadcast::Sender<Signal>) -> Result<Self, std::io::Error> {
        Ok(Self(sender))
    }

    pub fn send(&self, signal: Signal) {
        match self.0.send(signal.clone()) {
            Ok(_) => trace!("Published signal: {}", signal),
            Err(_) => warn!("Failed to publish motion"),
        }
    }
}

/// //////////////////////////////
/// /////////////////////////////////
/// /////////////////////////////////

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
}

///////////
/// /////////////////////////////////
///

pub struct SignalQueueReaderAsync2 {
    inner: tokio::sync::broadcast::Receiver<Signal>,
}

impl SignalQueueReaderAsync2 {
    pub fn new(reader: tokio::sync::broadcast::Receiver<Signal>) -> std::io::Result<Self> {
        Ok(Self { inner: reader })
    }

    pub async fn recv(&mut self) -> Result<Signal, ()> {
        Ok(self.inner.recv().await.unwrap())
    }
}
