use j1939::{protocol, spn, Frame, FrameBuilder, IdBuilder, PGN};

use crate::{
    core::{EngineState, Object, ObjectMessage},
    net::Parsable,
    runtime::{J1939Unit, J1939UnitError, NetDriverContext},
};

/// The `Engine` trait defines the basic operations for controlling an engine.
///
/// # Methods
///
/// * `request(&self, speed: u16) -> Frame`
///   - Requests speed control for the engine.
///   - `speed`: The desired speed to request.
///
/// * `start(&self, speed: u16) -> Frame`
///   - Starts the engine with the specified speed.
///   - `speed`: The initial speed to start the engine with.
///
/// * `stop(&self, _speed: u16) -> Frame`
///   - Stops the engine.
///   - `speed`: The speed parameter is ignored when stopping the engine.
pub trait Engine {
    /// Request speed control
    fn request(&self, speed: u16) -> Frame;
    /// Start the engine
    fn start(&self, speed: u16) -> Frame;
    /// Stop the engine
    fn stop(&self, speed: u16) -> Frame;
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
#[derive(Clone, Default)]
pub struct EngineManagementSystem {
    /// Network interface.
    #[allow(dead_code)]
    interface: String,
    /// Destination address.
    destination_address: u8,
    /// Source address.
    source_address: u8,
}

impl EngineManagementSystem {
    /// Construct a new engine management system.
    pub fn new(interface: &str, da: u8, sa: u8) -> Self {
        Self {
            interface: interface.to_string(),
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
    fn parse(&self, frame: &Frame) -> Option<EngineMessage> {
        match frame.id().pgn() {
            PGN::TorqueSpeedControl1 => Some(EngineMessage::TorqueSpeedControl(
                spn::TorqueSpeedControl1Message::from_pdu(frame.pdu()),
            )),
            PGN::ElectronicBrakeController1 => {
                if frame.id().source_address() != self.destination_address {
                    return None;
                }

                Some(EngineMessage::BrakeController1(
                    spn::ElectronicBrakeController1Message::from_pdu(frame.pdu()),
                ))
            }
            PGN::ElectronicEngineController1 => {
                if frame.id().source_address() != self.destination_address {
                    return None;
                }

                Some(EngineMessage::EngineController1(
                    spn::ElectronicEngineController1Message::from_pdu(frame.pdu()),
                ))
            }
            PGN::ElectronicEngineController2 => {
                if frame.id().source_address() != self.destination_address {
                    return None;
                }

                Some(EngineMessage::EngineController2(
                    spn::ElectronicEngineController2Message::from_pdu(frame.pdu()),
                ))
            }
            PGN::ElectronicEngineController3 => {
                if frame.id().source_address() != self.destination_address {
                    return None;
                }

                Some(EngineMessage::EngineController3(
                    spn::ElectronicEngineController3Message::from_pdu(frame.pdu()),
                ))
            }
            PGN::FanDrive => {
                if frame.id().source_address() != self.destination_address {
                    return None;
                }

                Some(EngineMessage::FanDrive(spn::FanDriveMessage::from_pdu(
                    frame.pdu(),
                )))
            }
            PGN::VehicleDistance => {
                if frame.id().source_address() != self.destination_address {
                    return None;
                }

                Some(EngineMessage::VehicleDistance(
                    spn::VehicleDistanceMessage::from_pdu(frame.pdu()),
                ))
            }
            PGN::Shutdown => {
                if frame.id().source_address() != self.destination_address {
                    return None;
                }

                Some(EngineMessage::Shutdown(spn::ShutdownMessage::from_pdu(
                    frame.pdu(),
                )))
            }
            PGN::EngineTemperature1 => {
                if frame.id().source_address() != self.destination_address {
                    return None;
                }

                Some(EngineMessage::EngineTemperature1(
                    spn::EngineTemperature1Message::from_pdu(frame.pdu()),
                ))
            }
            PGN::EngineFluidLevelPressure1 => {
                if frame.id().source_address() != self.destination_address {
                    return None;
                }

                Some(EngineMessage::EngineFluidLevelPressure1(
                    spn::EngineFluidLevelPressure1Message::from_pdu(frame.pdu()),
                ))
            }
            PGN::EngineFluidLevelPressure2 => {
                if frame.id().source_address() != self.destination_address {
                    return None;
                }

                Some(EngineMessage::EngineFluidLevelPressure2(
                    spn::EngineFluidLevelPressure2Message::from_pdu(frame.pdu()),
                ))
            }
            PGN::FuelEconomy => {
                if frame.id().source_address() != self.destination_address {
                    return None;
                }

                Some(EngineMessage::FuelEconomy(
                    spn::FuelEconomyMessage::from_pdu(frame.pdu()),
                ))
            }
            PGN::FuelConsumption => {
                if frame.id().source_address() != self.destination_address {
                    return None;
                }

                Some(EngineMessage::FuelConsumption(
                    spn::FuelConsumptionMessage::from_pdu(frame.pdu()),
                ))
            }
            PGN::AmbientConditions => {
                if frame.id().source_address() != self.destination_address {
                    return None;
                }

                Some(EngineMessage::AmbientConditions(
                    spn::AmbientConditionsMessage::from_pdu(frame.pdu()),
                ))
            }
            PGN::PowerTakeoffInformation => {
                if frame.id().source_address() != self.destination_address {
                    return None;
                }

                Some(EngineMessage::PowerTakeoffInformation(
                    spn::PowerTakeoffInformationMessage::from_pdu(frame.pdu()),
                ))
            }
            PGN::TANKInformation1 => {
                if frame.id().source_address() != self.destination_address {
                    return None;
                }

                Some(EngineMessage::TankInformation1(
                    spn::TankInformation1Message::from_pdu(frame.pdu()),
                ))
            }
            PGN::VehicleElectricalPower1 => {
                if frame.id().source_address() != self.destination_address {
                    return None;
                }

                Some(EngineMessage::VehicleElectricalPower(
                    spn::VehicleElectricalPowerMessage::from_pdu(frame.pdu()),
                ))
            }
            PGN::InletExhaustConditions1 => {
                if frame.id().source_address() != self.destination_address {
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

impl J1939Unit for EngineManagementSystem {
    fn vendor(&self) -> &'static str {
        "kübler"
    }

    fn product(&self) -> &'static str {
        "ecm"
    }

    fn destination(&self) -> u8 {
        self.destination_address
    }

    fn source(&self) -> u8 {
        self.source_address
    }

    fn setup(
        &self,
        _ctx: &mut NetDriverContext,
        tx_queue: &mut Vec<j1939::Frame>,
    ) -> Result<(), J1939UnitError> {
        tx_queue.push(protocol::request(
            self.destination_address,
            self.source_address,
            PGN::AddressClaimed,
        ));
        tx_queue.push(protocol::request(
            self.destination_address,
            self.source_address,
            PGN::SoftwareIdentification,
        ));
        tx_queue.push(protocol::request(
            self.destination_address,
            self.source_address,
            PGN::ComponentIdentification,
        ));

        Ok(())
    }

    fn try_recv(
        &self,
        ctx: &mut NetDriverContext,
        frame: &j1939::Frame,
        rx_queue: &mut Vec<Object>,
    ) -> Result<(), J1939UnitError> {
        if let Some(message) = self.parse(frame) {
            match message {
                EngineMessage::EngineController1(controller) => {
                    let mut engine_signal = crate::core::Engine::default();

                    if let Some(driver_demand) = controller.driver_demand {
                        engine_signal.driver_demand = driver_demand;
                    }
                    if let Some(actual_engine) = controller.actual_engine {
                        engine_signal.actual_engine = actual_engine;
                    }
                    if let Some(rpm) = controller.rpm {
                        engine_signal.rpm = rpm;
                    }

                    if let Some(starter_mode) = controller.starter_mode {
                        match starter_mode {
                            spn::EngineStarterMode::StarterActiveGearNotEngaged
                            | spn::EngineStarterMode::StarterActiveGearEngaged => {
                                engine_signal.state = EngineState::Starting;
                            }
                            spn::EngineStarterMode::StartFinished => {
                                if let Some(rpm) = controller.rpm {
                                    if rpm > 0 {
                                        engine_signal.state = EngineState::Request;
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
                                engine_signal.state = EngineState::NoRequest;
                            }
                            _ => {}
                        }
                    } else if let Some(rpm) = controller.rpm {
                        if rpm == 0 {
                            engine_signal.state = EngineState::NoRequest;
                        } else if rpm < 500 {
                            engine_signal.state = EngineState::Starting;
                        } else {
                            engine_signal.state = EngineState::Request;
                        }
                    } else if controller.rpm.is_none()
                        || controller.actual_engine.is_none()
                        || controller.engine_torque_mode.is_none()
                    {
                        engine_signal.state = EngineState::NoRequest;
                    }

                    ctx.set_rx_last_message(ObjectMessage::signal(Object::Engine(engine_signal)));

                    rx_queue.push(Object::Engine(engine_signal));

                    return Ok(());
                }
                EngineMessage::Shutdown(_shutdown) => {
                    // TODO: Handle shutdown message, set state to stopping

                    ctx.rx_mark();

                    return Ok(());
                }
                _ => {
                    ctx.rx_mark();

                    return Ok(());
                }
            }
        }

        Ok(())
    }
}
