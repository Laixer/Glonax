use j1939::{Frame, PGN};

use crate::net::Parsable;

use super::vecraft::VecraftStatusMessage;

const STATUS_PGN: u32 = 65_288;

pub struct VehicleControlUnit {
    /// Destination address.
    destination_address: u8,
    /// Source address.
    _source_address: u8,
}

impl VehicleControlUnit {
    /// Construct a new encoder service.
    pub fn new(da: u8, sa: u8) -> Self {
        Self {
            destination_address: da,
            _source_address: sa,
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

    async fn try_accept(
        &mut self,
        ctx: &mut super::NetDriverContext,
        state: &super::J1939UnitOperationState,
        router: &crate::net::Router,
        _runtime_state: crate::runtime::SharedOperandState,
    ) -> Result<(), super::J1939UnitError> {
        if state == &super::J1939UnitOperationState::Running {
            let mut result = Result::<(), super::J1939UnitError>::Ok(());

            if ctx.is_rx_timeout(std::time::Duration::from_millis(1_000)) {
                result = Err(super::J1939UnitError::MessageTimeout);
            }

            if let Some(status) = router.try_accept(self) {
                ctx.rx_mark();

                if status.state == super::vecraft::State::FaultyGenericError
                    || status.state == super::vecraft::State::FaultyBusError
                {
                    result = Err(super::J1939UnitError::BusError);
                }
            }

            result?;
        }

        Ok(())
    }
}
