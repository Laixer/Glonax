use std::{collections::VecDeque, sync::Arc};

use tokio::sync::RwLock;

use crate::core::metric::{Signal, SignalSource, SignalTuple};

const HISTORIC_ITEM_COUNT: usize = 32;

pub struct SignalPusher {
    queue: Arc<RwLock<VecDeque<SignalTuple>>>,
    // queue2: tokio::sync::mpsc::Sender<SignalTuple>,
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

        let mut queue = self.queue.write().await;
        queue.push_front((source, signal));

        if queue.len() > HISTORIC_ITEM_COUNT {
            queue.pop_back();
        }

        // self.queue2.send((source, signal)).await.ok();
    }
}

#[derive(Clone)]
pub struct SignalReader(Arc<RwLock<VecDeque<SignalTuple>>>);

impl SignalReader {
    pub fn try_lock(&self) -> Option<SignalReaderGuard> {
        match self.0.try_read() {
            Ok(guard) => Some(SignalReaderGuard(guard)),
            Err(_) => None,
        }
    }
}

pub struct SignalReaderGuard<'a>(tokio::sync::RwLockReadGuard<'a, VecDeque<SignalTuple>>);

impl SignalReaderGuard<'_> {
    /// Return the most recent item.
    #[inline]
    pub fn most_recent(&self) -> Option<&SignalTuple> {
        self.0.front()
    }

    /// Return the most recent item.
    pub fn most_recent_by_source(&self, source_input: SignalSource) -> Option<&Signal> {
        self.0
            .iter()
            .find(|(source, _)| source == &source_input)
            .map(|(_, signal)| signal)
    }

    // / Returns a front-to-back iterator.
    // #[inline]
    // pub fn iter(&self) -> vec_deque::Iter<SignalTuple> {
    //     self.0.iter()
    // }
}

pub struct SignalManager {
    queue: Arc<RwLock<VecDeque<SignalTuple>>>,
    // queue2: (
    //     tokio::sync::mpsc::Sender<SignalTuple>,
    //     tokio::sync::mpsc::Receiver<SignalTuple>,
    // ),
}

impl SignalManager {
    /// Construct new signal manager.
    pub fn new() -> Self {
        Self {
            queue: Arc::new(RwLock::new(VecDeque::new())),
            // queue2: tokio::sync::mpsc::channel(128),
        }
    }

    pub fn pusher(&self) -> SignalPusher {
        SignalPusher {
            queue: self.queue.clone(),
            // queue2: self.queue2.0.clone(),
        }
    }

    #[inline]
    pub fn reader(&self) -> SignalReader {
        SignalReader(self.queue.clone())
    }
}
