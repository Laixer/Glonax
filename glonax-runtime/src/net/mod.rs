use glonax_j1939::{j1939, J1939Listener};

// TODO: Rename to something with stream.
pub struct ControlNet {
    socket: J1939Listener,
}

impl ControlNet {
    pub fn open(ifname: &str, address: u8) -> Self {
        let socket = J1939Listener::bind(ifname, address).unwrap();
        socket.set_broadcast(true).unwrap();

        Self { socket }
    }

    // TODO: Maybe remove
    pub async fn send_to(&self, frame: &j1939::Frame) {
        debug!("Sending raw frame {}", frame);
        self.socket.send_to(frame).await.unwrap();
    }

    pub async fn accept(&self) -> j1939::Frame {
        self.socket.recv_from().await.unwrap()
    }

    pub async fn status(&self) {
        let frame = j1939::FrameBuilder::new(j1939::IdBuilder::from_pgn(65_282).build())
            .from_slice(&[0x71])
            .build();

        self.socket.send_to(&frame).await.unwrap();
    }

    pub async fn set_led(&self, node: u8, led_on: bool) {
        let frame = j1939::FrameBuilder::new(j1939::IdBuilder::from_pgn(45_312).da(node).build())
            .from_slice(&[b'Z', b'C', if led_on { 0x1 } else { 0x0 }])
            .build();

        self.socket.send_to(&frame).await.unwrap();
    }

    pub async fn set_address(&self, node: u8, address: u8) {
        let frame = j1939::FrameBuilder::new(j1939::IdBuilder::from_pgn(45_568).da(node).build())
            .from_slice(&[b'Z', b'C', address])
            .build();

        self.socket.send_to(&frame).await.unwrap();
    }

    pub async fn reset(&self, node: u8) {
        let frame = j1939::FrameBuilder::new(j1939::IdBuilder::from_pgn(45_312).da(node).build())
            .from_slice(&[b'Z', b'C', 0xff, 0x69])
            .build();

        self.socket.send_to(&frame).await.unwrap();
    }

    pub async fn enable_encoder(&self, node: u8, encoder: u8, encoder_on: bool) {
        let state = match (encoder, encoder_on) {
            (0, true) => 0b1101,
            (0, false) => 0b1100,
            (1, true) => 0b0111,
            (1, false) => 0b0011,
            _ => panic!(),
        };

        let frame = j1939::FrameBuilder::new(j1939::IdBuilder::from_pgn(45_824).da(node).build())
            .from_slice(&[b'Z', b'C', state])
            .build();

        self.socket.send_to(&frame).await.unwrap();
    }

    pub async fn set_motion_lock(&self, node: u8, locked: bool) {
        let frame = j1939::FrameBuilder::new(j1939::IdBuilder::from_pgn(45_824).da(node).build())
            .from_slice(&[b'Z', b'C', 0xff, if locked { 0x0 } else { 0x1 }])
            .build();

        self.socket.send_to(&frame).await.unwrap();
    }

    pub async fn request(&self, node: u8, _pgn: u32) {
        let frame = j1939::FrameBuilder::new(j1939::IdBuilder::from_pgn(59_904).da(node).build())
            .from_slice(&[0xfe, 0x18, 0xda])
            .build();

        self.socket.send_to(&frame).await.unwrap();
    }

    pub async fn gate_control(&self, node: u8, gate_bank: usize, value: [Option<i16>; 4]) {
        let bank = [40_960, 41_216];

        let frame = glonax_j1939::j1939::Frame::new(
            glonax_j1939::j1939::IdBuilder::from_pgn(bank[gate_bank])
                .da(node)
                .build(),
            [
                value[0].map_or(0xff, |v| v.to_le_bytes()[0]),
                value[0].map_or(0xff, |v| v.to_le_bytes()[1]),
                value[1].map_or(0xff, |v| v.to_le_bytes()[0]),
                value[1].map_or(0xff, |v| v.to_le_bytes()[1]),
                value[2].map_or(0xff, |v| v.to_le_bytes()[0]),
                value[2].map_or(0xff, |v| v.to_le_bytes()[1]),
                value[3].map_or(0xff, |v| v.to_le_bytes()[0]),
                value[3].map_or(0xff, |v| v.to_le_bytes()[1]),
            ],
        );

        self.socket.send_to(&frame).await.unwrap();
    }
}

/////////////////////

#[async_trait::async_trait]
pub trait J1939Stream {
    async fn recv_from(&self) -> std::io::Result<j1939::Frame>;

    async fn send_to(&self, frame: &j1939::Frame) -> std::io::Result<()>;
}

#[async_trait::async_trait]
impl J1939Stream for J1939Listener {
    async fn recv_from(&self) -> std::io::Result<j1939::Frame> {
        self.recv_from().await
    }

    async fn send_to(&self, frame: &j1939::Frame) -> std::io::Result<()> {
        self.send_to(frame).await
    }
}

pub struct ControlNet2<T: J1939Stream> {
    stream: T,
}

impl<T: J1939Stream> ControlNet2<T> {
    pub fn new(stream: T) -> Self {
        Self { stream }
    }

    pub fn inner(&self) -> &T {
        &self.stream
    }

    pub async fn accept(&self) -> j1939::Frame {
        self.stream.recv_from().await.unwrap()
    }

    pub async fn announce_status(&self) {
        let frame = j1939::FrameBuilder::new(j1939::IdBuilder::from_pgn(65_282).build())
            .from_slice(&[0x71])
            .build();

        self.stream.send_to(&frame).await.unwrap();
    }

    pub async fn set_led(&self, node: u8, led_on: bool) {
        let frame = j1939::FrameBuilder::new(j1939::IdBuilder::from_pgn(45_312).da(node).build())
            .from_slice(&[b'Z', b'C', if led_on { 0x1 } else { 0x0 }])
            .build();

        self.stream.send_to(&frame).await.unwrap();
    }

    pub async fn set_address(&self, node: u8, address: u8) {
        let frame = j1939::FrameBuilder::new(j1939::IdBuilder::from_pgn(45_568).da(node).build())
            .from_slice(&[b'Z', b'C', address])
            .build();

        self.stream.send_to(&frame).await.unwrap();
    }

    pub async fn reset(&self, node: u8) {
        let frame = j1939::FrameBuilder::new(j1939::IdBuilder::from_pgn(45_312).da(node).build())
            .from_slice(&[b'Z', b'C', 0xff, 0x69])
            .build();

        self.stream.send_to(&frame).await.unwrap();
    }

    pub async fn enable_encoder(&self, node: u8, encoder: u8, encoder_on: bool) {
        let state = match (encoder, encoder_on) {
            (0, true) => 0b1101,
            (0, false) => 0b1100,
            (1, true) => 0b0111,
            (1, false) => 0b0011,
            _ => panic!(),
        };

        let frame = j1939::FrameBuilder::new(j1939::IdBuilder::from_pgn(45_824).da(node).build())
            .from_slice(&[b'Z', b'C', state])
            .build();

        self.stream.send_to(&frame).await.unwrap();
    }

    pub async fn set_motion_lock(&self, node: u8, locked: bool) {
        let frame = j1939::FrameBuilder::new(j1939::IdBuilder::from_pgn(45_824).da(node).build())
            .from_slice(&[b'Z', b'C', 0xff, if locked { 0x0 } else { 0x1 }])
            .build();

        self.stream.send_to(&frame).await.unwrap();
    }

    pub async fn request(&self, node: u8, _pgn: u32) {
        let frame = j1939::FrameBuilder::new(j1939::IdBuilder::from_pgn(59_904).da(node).build())
            .from_slice(&[0xfe, 0x18, 0xda])
            .build();

        self.stream.send_to(&frame).await.unwrap();
    }

    pub async fn gate_control(&self, node: u8, gate_bank: usize, value: [Option<i16>; 4]) {
        let bank = [40_960, 41_216];

        let frame = glonax_j1939::j1939::Frame::new(
            glonax_j1939::j1939::IdBuilder::from_pgn(bank[gate_bank])
                .da(node)
                .build(),
            [
                value[0].map_or(0xff, |v| v.to_le_bytes()[0]),
                value[0].map_or(0xff, |v| v.to_le_bytes()[1]),
                value[1].map_or(0xff, |v| v.to_le_bytes()[0]),
                value[1].map_or(0xff, |v| v.to_le_bytes()[1]),
                value[2].map_or(0xff, |v| v.to_le_bytes()[0]),
                value[2].map_or(0xff, |v| v.to_le_bytes()[1]),
                value[3].map_or(0xff, |v| v.to_le_bytes()[0]),
                value[3].map_or(0xff, |v| v.to_le_bytes()[1]),
            ],
        );

        self.stream.send_to(&frame).await.unwrap();
    }
}
