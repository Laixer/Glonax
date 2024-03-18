use j1939::{spn, Frame, FrameBuilder, IdBuilder, PGN};

use crate::net::Parsable;

pub trait Engine {
    /// Request speed control
    fn request(&self, speed: u16) -> Frame;
    /// Start the engine
    fn start(&self, speed: u16) -> Frame;
    /// Stop the engine
    fn stop(&self, _speed: u16) -> Frame;
}

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
    EngineFluidLevelPressure2(spn::EngineFluidLevelPressure2Message),
    FuelEconomy(spn::FuelEconomyMessage),
    FuelConsumption(spn::FuelConsumptionMessage),
    AmbientConditions(spn::AmbientConditionsMessage),
    PowerTakeoffInformation(spn::PowerTakeoffInformationMessage),
    TankInformation1(spn::TankInformation1Message),
    VehicleElectricalPower(spn::VehicleElectricalPowerMessage),
    InletExhaustConditions1(spn::InletExhaustConditions1Message),
}

// TODO: Implement Engine trait
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
                override_control_mode: spn::OverrideControlMode::SpeedControl,
                speed_control_condition:
                    spn::RequestedSpeedControlCondition::TransientOptimizedDriveLineDisengaged,
                control_mode_priority: spn::OverrideControlModePriority::HighPriority,
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

impl Engine for EngineManagementSystem {
    fn request(&self, speed: u16) -> Frame {
        self.speed_control(speed)
    }

    fn start(&self, speed: u16) -> Frame {
        self.speed_control(speed)
    }

    fn stop(&self, _speed: u16) -> Frame {
        self.brake_control()
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
            PGN::EngineFluidLevelPressure2 => {
                if frame.id().sa() != self.destination_address {
                    return None;
                }

                Some(EngineMessage::EngineFluidLevelPressure2(
                    spn::EngineFluidLevelPressure2Message::from_pdu(frame.pdu()),
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
            PGN::FuelConsumption => {
                if frame.id().sa() != self.destination_address {
                    return None;
                }

                Some(EngineMessage::FuelConsumption(
                    spn::FuelConsumptionMessage::from_pdu(frame.pdu()),
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
            PGN::TANKInformation1 => {
                if frame.id().sa() != self.destination_address {
                    return None;
                }

                Some(EngineMessage::TankInformation1(
                    spn::TankInformation1Message::from_pdu(frame.pdu()),
                ))
            }
            PGN::VehicleElectricalPower1 => {
                if frame.id().sa() != self.destination_address {
                    return None;
                }

                Some(EngineMessage::VehicleElectricalPower(
                    spn::VehicleElectricalPowerMessage::from_pdu(frame.pdu()),
                ))
            }
            PGN::InletExhaustConditions1 => {
                if frame.id().sa() != self.destination_address {
                    return None;
                }

                Some(EngineMessage::InletExhaustConditions1(
                    spn::InletExhaustConditions1Message::from_pdu(frame.pdu()),
                ))
            }
            _ => None,
        }
    }
}

impl super::J1939Unit for EngineManagementSystem {
    fn name(&self) -> &str {
        "Engine management system"
    }

    fn destination(&self) -> u8 {
        self.destination_address
    }

    async fn try_accept(
        &mut self,
        ctx: &mut super::NetDriverContext,
        state: &super::J1939UnitOperationState,
        router: &crate::net::Router,
        runtime_state: crate::runtime::SharedOperandState,
    ) -> Result<(), super::J1939UnitError> {
        if state == &super::J1939UnitOperationState::Running {
            let mut result = Result::<(), super::J1939UnitError>::Ok(());

            if ctx.rx_last.elapsed().as_millis() > 500 {
                result = Err(super::J1939UnitError::MessageTimeout);
            }

            if let Some(message) = router.try_accept(self) {
                ctx.rx_last = std::time::Instant::now();

                if let Ok(mut runtime_state) = runtime_state.try_write() {
                    runtime_state.state.engine_state_actual_instant =
                        Some(std::time::Instant::now());

                    if let EngineMessage::EngineController1(controller) = message {
                        if let Some(driver_demand) = controller.driver_demand {
                            runtime_state.state.engine.driver_demand = driver_demand;
                        }
                        if let Some(actual_engine) = controller.actual_engine {
                            runtime_state.state.engine.actual_engine = actual_engine;
                        }
                        if let Some(rpm) = controller.rpm {
                            runtime_state.state.engine.rpm = rpm;
                            runtime_state.state.engine_state_actual.speed = rpm;
                        }

                        if let Some(starter_mode) = controller.starter_mode {
                            match starter_mode {
                                spn::EngineStarterMode::StarterActiveGearNotEngaged
                                | spn::EngineStarterMode::StarterActiveGearEngaged => {
                                    runtime_state.state.engine_state_actual.state =
                                        crate::core::EngineState::Starting
                                }
                                spn::EngineStarterMode::StartFinished => {
                                    if let Some(rpm) = controller.rpm {
                                        if rpm > 0 {
                                            runtime_state.state.engine_state_actual.state =
                                                crate::core::EngineState::Request;
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
                                    runtime_state.state.engine_state_actual.state =
                                        crate::core::EngineState::NoRequest;
                                }
                                _ => {}
                            }
                        } else if let Some(rpm) = controller.rpm {
                            if rpm == 0 {
                                runtime_state.state.engine_state_actual.state =
                                    crate::core::EngineState::NoRequest;
                            } else if rpm < 500 {
                                runtime_state.state.engine_state_actual.state =
                                    crate::core::EngineState::Starting;
                            } else {
                                runtime_state.state.engine_state_actual.state =
                                    crate::core::EngineState::Request;
                            }
                        } else if controller.rpm.is_none()
                            || controller.actual_engine.is_none()
                            || controller.engine_torque_mode.is_none()
                        {
                            runtime_state.state.engine_state_actual.state =
                                crate::core::EngineState::NoRequest;
                        }
                    }
                }
            }

            result?
        }

        Ok(())
    }

    async fn tick(
        &self,
        ctx: &mut super::NetDriverContext,
        state: &super::J1939UnitOperationState,
        router: &crate::net::Router,
        runtime_state: crate::runtime::SharedOperandState,
    ) -> Result<(), super::J1939UnitError> {
        if state == &super::J1939UnitOperationState::Running {
            let request = runtime_state.read().await.governor_mode();
            match request.state {
                crate::core::EngineState::NoRequest => {
                    router.send(&self.request(request.speed)).await?;
                    ctx.tx_last = std::time::Instant::now();
                }
                crate::core::EngineState::Stopping => {
                    router.send(&self.stop(request.speed)).await?;
                    ctx.tx_last = std::time::Instant::now();
                }
                crate::core::EngineState::Starting => {
                    router.send(&self.start(request.speed)).await?;
                    ctx.tx_last = std::time::Instant::now();
                }
                crate::core::EngineState::Request => {
                    router.send(&self.request(request.speed)).await?;
                    ctx.tx_last = std::time::Instant::now();
                }
            }
        }

        Ok(())
    }
}
