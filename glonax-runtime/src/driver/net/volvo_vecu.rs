use j1939::{Frame, FrameBuilder, IdBuilder, PGN};

#[allow(dead_code)]
#[derive(Default)]
pub struct VolvoVECU {
    /// Destination address.
    destination_address: u8,
    /// Source address.
    source_address: u8,
}

#[allow(dead_code)]
impl VolvoVECU {
    /// Construct a new engine management system.
    pub fn new(da: u8, sa: u8) -> Self {
        Self {
            destination_address: da,
            source_address: sa,
        }
    }

    pub fn network_unlock(&self) -> Frame {
        FrameBuilder::new(
            IdBuilder::from_pgn(PGN::ProprietaryB(65_410))
                .sa(self.source_address)
                .build(),
        )
        .copy_from_slice(&[0x0C, 0x5C, 0x00, 0x00, 0x00, 0x00, 0x05, 0xFF])
        .build()
    }
}
