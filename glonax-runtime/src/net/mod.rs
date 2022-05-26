use glonax_j1939::{j1939, J1939Listener};

pub struct ControlNet {
    socket: J1939Listener,
}

impl ControlNet {
    pub fn open(ifname: &str, address: u8) -> Self {
        let socket = J1939Listener::bind(ifname, address).unwrap();
        socket.set_broadcast(true).unwrap();

        Self { socket }
    }

    pub async fn accept(&self) -> j1939::Frame {
        loop {
            match tokio::time::timeout(
                std::time::Duration::from_millis(100),
                self.socket.recv_from(),
            )
            .await
            {
                Ok(e) => {
                    break e.unwrap();
                }
                Err(_) => {
                    self.status().await;
                    continue;
                }
            }
        }
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
        let frame = j1939::FrameBuilder::new(j1939::IdBuilder::from_pgn(45_568).da(node).build())
            .from_slice(&[b'Z', b'C', 0xff, 0x69])
            .build();

        self.socket.send_to(&frame).await.unwrap();
    }

    pub async fn request(&self, node: u8, _pgn: u32) {
        let frame = j1939::FrameBuilder::new(j1939::IdBuilder::from_pgn(59_904).da(node).build())
            .from_slice(&[0xfe, 0x18, 0xda])
            .build();

        self.socket.send_to(&frame).await.unwrap();
    }

    pub async fn gate_control(&self, node: u8, gate_bank: usize, value: [i16; 4]) {
        let bank = [40_960, 41_216];

        let frame = glonax_j1939::j1939::Frame::new(
            glonax_j1939::j1939::IdBuilder::from_pgn(bank[gate_bank])
                .da(node)
                .build(),
            [
                value[0].to_le_bytes()[0],
                value[0].to_le_bytes()[1],
                value[1].to_le_bytes()[0],
                value[1].to_le_bytes()[1],
                value[2].to_le_bytes()[0],
                value[2].to_le_bytes()[1],
                value[3].to_le_bytes()[0],
                value[3].to_le_bytes()[1],
            ],
        );

        self.socket.send_to(&frame).await.unwrap();
    }
}
