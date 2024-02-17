use j1939::{Frame, FrameBuilder, Id};
use rand::Rng;

pub struct Fuzzer {
    /// Destination id.
    destination_id: Id,
}

impl Fuzzer {
    /// Construct a new fuzzer.
    pub fn new(id: Id) -> Self {
        Self { destination_id: id }
    }

    pub fn gen_frame(&self) -> Frame {
        let random_number = rand::thread_rng().gen_range(0..=8);
        let random_bytes = (0..random_number)
            .map(|_| rand::random::<u8>())
            .collect::<Vec<u8>>();

        FrameBuilder::new(self.destination_id)
            .copy_from_slice(&random_bytes)
            .build()
    }
}
