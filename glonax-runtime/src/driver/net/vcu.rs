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
    async fn try_accept(
        &mut self,
        ctx: &mut super::NetDriverContext,
        state: &super::J1939UnitOperationState,
        router: &crate::net::Router,
        _runtime_state: crate::runtime::SharedOperandState,
    ) -> Result<(), super::J1939UnitError> {
        match state {
            super::J1939UnitOperationState::Setup => {
                log::debug!(
                    "[0x{:X}] Vehicle control unit ingress setup",
                    self.destination_address
                );
            }
            super::J1939UnitOperationState::Running => {
                if let Some(status) = router.try_accept(self) {
                    ctx.rx_last = std::time::Instant::now();

                    if status.state == super::vecraft::State::FaultyGenericError
                        || status.state == super::vecraft::State::FaultyBusError
                    {
                        Err(super::J1939UnitError::new(
                            "Vehicle control unit".to_owned(),
                            self.destination_address,
                            super::J1939UnitErrorKind::BusError,
                        ))?;
                    }
                }
            }
            super::J1939UnitOperationState::Teardown => {
                log::debug!(
                    "[0x{:X}] Vehicle control unit ingress teardown",
                    self.destination_address
                );
            }
        }

        Ok(())
    }

    async fn tick(
        &self,
        ctx: &mut super::NetDriverContext,
        state: &super::J1939UnitOperationState,
        _router: &crate::net::Router,
        _runtime_state: crate::runtime::SharedOperandState,
    ) -> Result<(), super::J1939UnitError> {
        if state == &super::J1939UnitOperationState::Running
            && ctx.rx_last.elapsed().as_millis() > 1_000
        {
            Err(super::J1939UnitError::new(
                "Kubler inclinometer".to_owned(),
                self.destination_address,
                super::J1939UnitErrorKind::MessageTimeout,
            ))?;
        }

        Ok(())
    }
}
