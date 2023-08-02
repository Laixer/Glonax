use tokio::sync::broadcast::{self, Receiver, Sender};

pub trait SignalChannel {
    fn push(&mut self, signal: crate::core::Signal);
}

pub trait SignalSource {
    fn fetch2(&self, writer: &mut impl SignalChannel);
}

pub trait BroadcastSource<T> {
    fn fetch(&self, writer: &BroadcastChannelWriter<T>);
}

pub type BroadcastChannelReader<T> = Receiver<T>;
pub type BroadcastChannelWriter<T> = Sender<T>;

pub fn broadcast_channel<T: Clone>(capacity: usize) -> BroadcastChannelWriter<T> {
    broadcast::channel(capacity).0
}

pub fn broadcast_bichannel<T: Clone>(
    capacity: usize,
) -> (BroadcastChannelWriter<T>, BroadcastChannelReader<T>) {
    broadcast::channel(capacity)
}
