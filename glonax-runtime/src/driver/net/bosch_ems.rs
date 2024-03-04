use j1939::{spn, Frame, FrameBuilder, IdBuilder, PGN};

use crate::net::Parsable;

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
pub struct BoschEngineManagementSystem {
    /// Destination address.
    destination_address: u8,
    /// Source address.
    source_address: u8,
}

impl BoschEngineManagementSystem {
    /// Construct a new engine management system.
    pub fn new(da: u8, sa: u8) -> Self {
        Self {
            destination_address: da,
            source_address: sa,
        }
    }

    /// Request torque control
    pub fn speed_control(&self, rpm: u16) -> Frame {
        FrameBuilder::new(
            IdBuilder::from_pgn(PGN::TorqueSpeedControl1)
                .priority(3)
                .da(self.destination_address)
                .sa(self.source_address)
                .build(),
        )
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

    /// Request brake control
    pub fn brake_control(&self) -> Frame {
        FrameBuilder::new(
            IdBuilder::from_pgn(PGN::ElectronicBrakeController1)
                .priority(3)
                .da(self.destination_address)
                .sa(self.source_address)
                .build(),
        )
        .copy_from_slice(
            &spn::ElectronicBrakeController1Message {
                asr_engine_control_active: None,
                asr_brake_control_active: None,
                abs_active: None,
                ebs_brake_switch: None,
                brake_pedal_position: None,
                abs_off_road_switch: None,
                asr_off_road_switch: None,
                asr_hill_holder_switch: None,
                traction_control_override_switch: None,
                accelerator_interlock_switch: None,
                engine_derate_switch: None,
                auxiliary_engine_shutdown_switch: Some(true),
                remote_accelerator_enable_switch: None,
                engine_retarder_selection: None,
                abs_fully_operational: None,
                ebs_red_warning_signal: None,
                abs_ebs_amber_warning_signal: None,
                atc_asr_information_signal: None,
                source_address: None,
                trailer_abs_status: None,
                tractor_mounted_trailer_abs_warning_signal: None,
            }
            .to_pdu(),
        )
        .build()
    }
}

impl Parsable<EngineMessage> for BoschEngineManagementSystem {
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

impl super::J1939Unit for BoschEngineManagementSystem {
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
                                    runtime_state.state.engine_state_actual =
                                        crate::core::EngineState::Starting(0)
                                }
                                spn::EngineStarterMode::StartFinished => {
                                    if let Some(rpm) = controller.rpm {
                                        if rpm > 0 {
                                            runtime_state.state.engine_state_actual =
                                                crate::core::EngineState::Request(rpm);
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
                                    runtime_state.state.engine_state_actual =
                                        crate::core::EngineState::NoRequest;
                                }
                                _ => {}
                            }
                        } else if let Some(rpm) = controller.rpm {
                            if rpm == 0 {
                                runtime_state.state.engine_state_actual =
                                    crate::core::EngineState::NoRequest;
                            } else if rpm < 500 {
                                runtime_state.state.engine_state_actual =
                                    crate::core::EngineState::Starting(rpm);
                            } else {
                                runtime_state.state.engine_state_actual =
                                    crate::core::EngineState::Request(rpm);
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
        match runtime_state.read().await.governor_mode() {
            crate::core::EngineState::NoRequest => {
                if let Err(e) = router.inner().send(&self.speed_control(0)).await {
                    log::error!("Failed to speed request: {}", e);
                }
            }
            crate::core::EngineState::Stopping => {
                if let Err(e) = router.inner().send(&self.brake_control()).await {
                    log::error!("Failed to speed request: {}", e);
                }
            }
            crate::core::EngineState::Starting(rpm) | crate::core::EngineState::Request(rpm) => {
                if let Err(e) = router.inner().send(&self.speed_control(rpm)).await {
                    log::error!("Failed to speed request: {}", e);
                }
            }
        }
    }
}
