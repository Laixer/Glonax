pub trait BroadcastSource<T> {
    fn fetch(&self, writer: &BroadcastChannelWriter<T>);
}

pub struct BroadcastChannel<T>(
    (
        tokio::sync::broadcast::Sender<T>,
        tokio::sync::broadcast::Receiver<T>,
    ),
);

impl<T: std::clone::Clone> BroadcastChannel<T> {
    pub fn new(capacity: usize) -> Self {
        Self(tokio::sync::broadcast::channel(capacity))
    }

    pub fn writer(&self) -> BroadcastChannelWriter<T> {
        BroadcastChannelWriter(self.0 .0.clone())
    }

    pub fn reader(&self) -> BroadcastChannelReader<T> {
        BroadcastChannelReader(self.0 .0.subscribe())
    }
}

pub struct BroadcastChannelReader<T>(tokio::sync::broadcast::Receiver<T>);

impl<T: Clone> BroadcastChannelReader<T> {
    pub async fn recv(&mut self) -> Result<T, tokio::sync::broadcast::error::RecvError>  {
        self.0.recv().await
    }
}

pub struct BroadcastChannelWriter<T>(tokio::sync::broadcast::Sender<T>);

impl<T: std::fmt::Display + Clone> BroadcastChannelWriter<T> {
    pub fn send(&self, value: T) {
        match self.0.send(value.clone()) {
            Ok(_) => trace!("Published message: {}", value),
            Err(_) => warn!("Failed to publish motion"),
        }
    }
}
