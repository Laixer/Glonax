use crate::core::metric::{Signal, SignalSource, SignalTuple};

mod encoder;
pub(crate) use encoder::Encoder;
use tokio::sync::broadcast;

pub struct SignalPusher {
    queue: broadcast::Sender<SignalTuple>,
}

impl SignalPusher {
    pub async fn push(&mut self, source: SignalSource, signal: Signal) {
        let subaddress = source & 0b00001111;
        let address = source >> 4;

        trace!(
            "Push new signal: 0x{:X?}:{} ⇨ {}",
            address,
            subaddress,
            signal.value
        );

        self.queue.send((source, signal)).unwrap();
    }
}

pub struct SignalReader(broadcast::Receiver<SignalTuple>);

impl SignalReader {
    #[inline]
    pub async fn recv(&mut self) -> Result<(u32, Signal), broadcast::error::RecvError> {
        self.0.recv().await
    }
}

pub struct SignalManager {
    queue: (
        broadcast::Sender<SignalTuple>,
        broadcast::Receiver<SignalTuple>,
    ),
}

impl SignalManager {
    /// Construct new signal manager.
    pub fn new() -> Self {
        Self {
            queue: broadcast::channel(128),
        }
    }

    pub fn pusher(&self) -> SignalPusher {
        SignalPusher {
            queue: self.queue.0.clone(),
        }
    }

    #[inline]
    pub fn reader(&self) -> SignalReader {
        SignalReader(self.queue.0.subscribe())
    }
}
