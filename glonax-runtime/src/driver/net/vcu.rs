use j1939::{protocol, Frame, PGN};

use crate::net::Parsable;

use super::vecraft::VecraftStatusMessage;

const STATUS_PGN: u32 = 65_288;

pub struct VehicleControlUnit {
    /// Destination address.
    destination_address: u8,
    /// Source address.
    source_address: u8,
}

impl VehicleControlUnit {
    /// Construct a new encoder service.
    pub fn new(da: u8, sa: u8) -> Self {
        Self {
            destination_address: da,
            source_address: sa,
        }
    }
}

impl Parsable<VecraftStatusMessage> for VehicleControlUnit {
    fn parse(&mut self, frame: &Frame) -> Option<VecraftStatusMessage> {
        if frame.id().pgn() == PGN::ProprietaryB(STATUS_PGN) {
            if frame.id().sa() != self.destination_address {
                return None;
            }

            Some(VecraftStatusMessage::from_frame(frame))
        } else {
            None
        }
    }
}

impl super::J1939Unit for VehicleControlUnit {
    fn name(&self) -> &str {
        "Vehicle control unit"
    }

    fn destination(&self) -> u8 {
        self.destination_address
    }

    fn source(&self) -> u8 {
        self.source_address
    }

    #[rustfmt::skip]
    async fn setup(
        &self,
        ctx: &mut super::NetDriverContext,
        router: &crate::net::Router,
        _runtime_state: crate::runtime::SharedOperandState,
    ) -> Result<(), super::J1939UnitError> {
        // TODO: FIX: It is possible that the request is send from 0x0.
        router.send(&protocol::request(self.destination_address, PGN::AddressClaimed)).await?;
        router.send(&protocol::request(self.destination_address, PGN::SoftwareIdentification)).await?;
        router.send(&protocol::request(self.destination_address, PGN::ComponentIdentification)).await?;
        ctx.tx_mark();

        Ok(())
    }

    async fn try_accept(
        &mut self,
        ctx: &mut super::NetDriverContext,
        router: &crate::net::Router,
        _runtime_state: crate::runtime::SharedOperandState,
    ) -> Result<(), super::J1939UnitError> {
        let mut result = Result::<(), super::J1939UnitError>::Ok(());

        if ctx.is_rx_timeout(std::time::Duration::from_millis(1_000)) {
            result = Err(super::J1939UnitError::MessageTimeout);
        }

        if let Some(status) = router.try_accept(self) {
            ctx.rx_mark();

            status.into_error()?;
        }

        result
    }
}
