use j1939::{spn, Frame, FrameBuilder, IdBuilder, PDU_MAX_LENGTH, PGN};

use crate::net::Parsable;

use super::vecraft::VecraftConfigMessage;

pub enum EngineMessage {
    TorqueSpeedControl(spn::TorqueSpeedControl1Message),
    BrakeController1(spn::ElectronicBrakeController1Message),
    EngineController1(spn::ElectronicEngineController1Message),
    EngineController2(spn::ElectronicEngineController2Message),
    EngineController3(spn::ElectronicEngineController3Message),
    FanDrive(spn::FanDriveMessage),
    VehicleDistance(spn::VehicleDistanceMessage),
    Shutdown(spn::ShutdownMessage),
    EngineTemperature1(spn::EngineTemperature1Message),
    EngineFluidLevelPressure1(spn::EngineFluidLevelPressure1Message),
    FuelEconomy(spn::FuelEconomyMessage),
    AmbientConditions(spn::AmbientConditionsMessage),
    PowerTakeoffInformation(spn::PowerTakeoffInformationMessage),
}

#[derive(Default)]
pub struct EngineManagementSystem {
    /// Destination address.
    destination_address: u8,
    /// Source address.
    source_address: u8,
}

impl EngineManagementSystem {
    /// Construct a new engine management system.
    pub fn new(da: u8, sa: u8) -> Self {
        Self {
            destination_address: da,
            source_address: sa,
        }
    }

    /// Set or unset identification mode.
    pub fn set_ident(&self, on: bool) -> Vec<Frame> {
        VecraftConfigMessage {
            destination_address: self.destination_address,
            source_address: self.source_address,
            ident_on: Some(on),
            reboot: false,
        }
        .to_frame()
    }

    /// System reboot / reset
    pub fn reboot(&self) -> Vec<Frame> {
        VecraftConfigMessage {
            destination_address: self.destination_address,
            source_address: self.source_address,
            ident_on: None,
            reboot: true,
        }
        .to_frame()
    }

    /// Request torque control
    pub fn torque_control(&self, rpm: u16) -> Frame {
        let frame_builder = FrameBuilder::new(
            IdBuilder::from_pgn(PGN::TorqueSpeedControl1)
                .priority(3)
                .da(self.destination_address)
                .sa(self.source_address)
                .build(),
        );

        frame_builder
            .copy_from_slice(
                &spn::TorqueSpeedControl1Message {
                    override_control_mode: Some(spn::OverrideControlMode::SpeedControl),
                    speed_control_condition: None,
                    control_mode_priority: None,
                    speed: Some(rpm),
                    torque: None,
                }
                .to_pdu(),
            )
            .build()
    }

    pub fn start(&self, rpm: u16) -> Frame {
        let frame_builder = FrameBuilder::new(
            IdBuilder::from_pgn(PGN::TorqueSpeedControl1)
                .priority(3)
                .da(self.destination_address)
                .sa(self.source_address)
                .build(),
        );

        // TODO: This is not correct. 'SpeedTorqueLimitControl' is not used for starting the engine.
        frame_builder
            .copy_from_slice(
                &spn::TorqueSpeedControl1Message {
                    override_control_mode: Some(spn::OverrideControlMode::SpeedTorqueLimitControl),
                    speed_control_condition: None,
                    control_mode_priority: None,
                    speed: Some(rpm),
                    torque: None,
                }
                .to_pdu(),
            )
            .build()
    }

    pub fn shutdown(&self) -> Frame {
        // TODO: Make this a J1939 message
        let mut frame_builder = FrameBuilder::new(
            IdBuilder::from_pgn(PGN::ElectronicBrakeController1)
                .priority(3)
                .da(self.destination_address)
                .sa(self.source_address)
                .build(),
        );

        frame_builder.as_mut()[3] = 0b0001_0000;

        frame_builder.set_len(PDU_MAX_LENGTH).build()
    }
}

impl Parsable<EngineMessage> for EngineManagementSystem {
    fn parse(&mut self, frame: &Frame) -> Option<EngineMessage> {
        match frame.id().pgn() {
            PGN::TorqueSpeedControl1 => Some(EngineMessage::TorqueSpeedControl(
                spn::TorqueSpeedControl1Message::from_pdu(frame.pdu()),
            )),
            PGN::ElectronicBrakeController1 => {
                if frame.id().sa() != self.destination_address {
                    return None;
                }

                Some(EngineMessage::BrakeController1(
                    spn::ElectronicBrakeController1Message::from_pdu(frame.pdu()),
                ))
            }
            PGN::ElectronicEngineController1 => {
                if frame.id().sa() != self.destination_address {
                    return None;
                }

                Some(EngineMessage::EngineController1(
                    spn::ElectronicEngineController1Message::from_pdu(frame.pdu()),
                ))
            }
            PGN::ElectronicEngineController2 => {
                if frame.id().sa() != self.destination_address {
                    return None;
                }

                Some(EngineMessage::EngineController2(
                    spn::ElectronicEngineController2Message::from_pdu(frame.pdu()),
                ))
            }
            PGN::ElectronicEngineController3 => {
                if frame.id().sa() != self.destination_address {
                    return None;
                }

                Some(EngineMessage::EngineController3(
                    spn::ElectronicEngineController3Message::from_pdu(frame.pdu()),
                ))
            }
            PGN::FanDrive => {
                if frame.id().sa() != self.destination_address {
                    return None;
                }

                Some(EngineMessage::FanDrive(spn::FanDriveMessage::from_pdu(
                    frame.pdu(),
                )))
            }
            PGN::VehicleDistance => {
                if frame.id().sa() != self.destination_address {
                    return None;
                }

                Some(EngineMessage::VehicleDistance(
                    spn::VehicleDistanceMessage::from_pdu(frame.pdu()),
                ))
            }
            PGN::Shutdown => {
                if frame.id().sa() != self.destination_address {
                    return None;
                }

                Some(EngineMessage::Shutdown(spn::ShutdownMessage::from_pdu(
                    frame.pdu(),
                )))
            }
            PGN::EngineTemperature1 => {
                if frame.id().sa() != self.destination_address {
                    return None;
                }

                Some(EngineMessage::EngineTemperature1(
                    spn::EngineTemperature1Message::from_pdu(frame.pdu()),
                ))
            }
            PGN::EngineFluidLevelPressure1 => {
                if frame.id().sa() != self.destination_address {
                    return None;
                }

                Some(EngineMessage::EngineFluidLevelPressure1(
                    spn::EngineFluidLevelPressure1Message::from_pdu(frame.pdu()),
                ))
            }
            PGN::FuelEconomy => {
                if frame.id().sa() != self.destination_address {
                    return None;
                }

                Some(EngineMessage::FuelEconomy(
                    spn::FuelEconomyMessage::from_pdu(frame.pdu()),
                ))
            }
            PGN::AmbientConditions => {
                if frame.id().sa() != self.destination_address {
                    return None;
                }

                Some(EngineMessage::AmbientConditions(
                    spn::AmbientConditionsMessage::from_pdu(frame.pdu()),
                ))
            }
            PGN::PowerTakeoffInformation => {
                if frame.id().sa() != self.destination_address {
                    return None;
                }

                Some(EngineMessage::PowerTakeoffInformation(
                    spn::PowerTakeoffInformationMessage::from_pdu(frame.pdu()),
                ))
            }
            _ => None,
        }
    }
}

impl super::J1939Unit for EngineManagementSystem {
    async fn try_accept(
        &mut self,
        router: &crate::net::Router,
        runtime_state: crate::runtime::SharedOperandState,
    ) {
        if let Some(message) = router.try_accept(self) {
            if let Ok(mut runtime_state) = runtime_state.try_write() {
                match message {
                    EngineMessage::EngineController1(controller) => {
                        if let Some(driver_demand) = controller.driver_demand {
                            runtime_state.state.engine.driver_demand = driver_demand;
                        }
                        if let Some(actual_engine) = controller.actual_engine {
                            runtime_state.state.engine.actual_engine = actual_engine;
                        }
                        if let Some(rpm) = controller.rpm {
                            runtime_state.state.engine.rpm = rpm;
                        }

                        if let Some(starter_mode) = controller.starter_mode {
                            match starter_mode {
                                spn::EngineStarterMode::StarterActiveGearNotEngaged
                                | spn::EngineStarterMode::StarterActiveGearEngaged => {
                                    let _ = crate::core::EngineMode::Starting;
                                }
                                spn::EngineStarterMode::StartFinished => {
                                    if let Some(rpm) = controller.rpm {
                                        if rpm > 0 {
                                            let _ = crate::core::EngineMode::Request(rpm);
                                        }
                                    }
                                }
                                spn::EngineStarterMode::StartNotRequested
                                | spn::EngineStarterMode::StarterInhibitedEngineRunning
                                | spn::EngineStarterMode::StarterInhibitedEngineNotReady
                                | spn::EngineStarterMode::StarterInhibitedTransmissionInhibited
                                | spn::EngineStarterMode::StarterInhibitedActiveImmobilizer
                                | spn::EngineStarterMode::StarterInhibitedOverHeat
                                | spn::EngineStarterMode::StarterInhibitedReasonUnknown => {
                                    let _ = crate::core::EngineMode::NoRequest;
                                }
                                _ => {}
                            }
                        }
                    }
                    EngineMessage::EngineController2(_controller) => {
                        //
                    }
                    EngineMessage::EngineController3(_controller) => {
                        //
                    }
                    _ => {}
                }
            }
        }
    }

    // FUTURE: Optimize
    async fn tick(
        &self,
        router: &crate::net::Router,
        runtime_state: crate::runtime::SharedOperandState,
    ) {
        let engine = runtime_state.read().await.state.engine;
        let engine_request = runtime_state.read().await.state.engine_request;

        match engine.mode() {
            crate::core::EngineMode::NoRequest => {
                if engine_request == 0 {
                    if let Err(e) = router.inner().send(&self.torque_control(0)).await {
                        log::error!("Failed to speed request: {}", e);
                    }
                } else if let Err(e) = router.inner().send(&self.start(engine_request)).await {
                    log::error!("Failed to speed request: {}", e);
                }
            }
            crate::core::EngineMode::Starting => {
                if engine_request == 0 {
                    if let Err(e) = router.inner().send(&self.torque_control(0)).await {
                        log::error!("Failed to speed request: {}", e);
                    }
                } else if let Err(e) = router.inner().send(&self.start(engine_request)).await {
                    log::error!("Failed to speed request: {}", e);
                }
            }
            crate::core::EngineMode::Request(_) => {
                if engine_request == 0 {
                    if let Err(e) = router.inner().send(&self.shutdown()).await {
                        log::error!("Failed to speed request: {}", e);
                    }
                } else if let Err(e) = router
                    .inner()
                    .send(&self.torque_control(engine_request))
                    .await
                {
                    log::error!("Failed to speed request: {}", e);
                }
            }
        }
    }
}
