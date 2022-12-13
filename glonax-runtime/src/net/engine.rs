use glonax_j1939::{
    decode::{EngineStarterMode, EngineTorqueMode},
    *,
};

use super::Routable;

pub struct EngineService {
    node: u8,
    engine_torque_mode: Option<EngineTorqueMode>,
    driver_demand: Option<u8>,
    actual_engine: Option<u8>,
    rpm: Option<u16>,
    source_addr: Option<u8>,
    starter_mode: Option<EngineStarterMode>,
}

#[async_trait::async_trait]
impl Routable for EngineService {
    fn node(&self) -> u8 {
        self.node
    }

    fn ingress(&mut self, pgn: PGN, frame: &Frame) -> bool {
        if pgn == PGN::ElectronicEngineController2 {
            self.engine_torque_mode = decode::spn899(frame.pdu()[0]);
            self.driver_demand = decode::spn512(frame.pdu()[1]);
            self.actual_engine = decode::spn513(frame.pdu()[2]);
            self.rpm = decode::spn190(&frame.pdu()[3..5].try_into().unwrap());
            self.source_addr = decode::spn1483(frame.pdu()[5]);
            self.starter_mode = decode::spn1675(frame.pdu()[6]);

            true
        } else {
            false
        }
    }
}

impl std::fmt::Display for EngineService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Torque mode: {:?}; Drivers Demand {}%; Actual Engine: {}%; RPM {}; Starter mode: {:?}",
            self.engine_torque_mode.as_ref().unwrap(),
            self.driver_demand.unwrap(),
            self.actual_engine.unwrap(),
            self.rpm.unwrap(),
            self.starter_mode.as_ref().unwrap(),
        )
    }
}

impl EngineService {
    pub fn new(node: u8) -> Self {
        Self {
            node,
            engine_torque_mode: None,
            driver_demand: None,
            actual_engine: None,
            rpm: None,
            source_addr: None,
            starter_mode: None,
        }
    }

    pub fn rpm(&self) -> Option<u16> {
        self.rpm
    }
}

#[test]
fn engine_service_engine_on() {
    let mut engine_service = EngineService::new(0x0);

    let frame = FrameBuilder::new(IdBuilder::from_pgn(PGN::ElectronicEngineController2).build())
        .copy_from_slice(&[0xF0, 0xEA, 0x7D, 0x00, 0x00, 0x00, 0xF0, 0xFF])
        .build();
    assert_eq!(
        engine_service.ingress(PGN::ElectronicEngineController2, &frame),
        true
    );
    assert_eq!(
        engine_service.engine_torque_mode.unwrap(),
        EngineTorqueMode::NoRequest
    );
    assert_eq!(engine_service.driver_demand.unwrap(), 109);
    assert_eq!(engine_service.actual_engine.unwrap(), 0);
    assert_eq!(engine_service.rpm.unwrap(), 0);
    assert_eq!(engine_service.source_addr.unwrap(), 0);
    assert_eq!(
        engine_service.starter_mode.unwrap(),
        EngineStarterMode::StartNotRequested
    );
}

#[test]
fn engine_service_engine_off() {
    let mut engine_service = EngineService::new(0x0);

    let frame = FrameBuilder::new(IdBuilder::from_pgn(PGN::ElectronicEngineController2).build())
        .copy_from_slice(&[0xF3, 0x91, 0x91, 0xAA, 0x18, 0x00, 0xF3, 0xFF])
        .build();

    assert_eq!(
        engine_service.ingress(PGN::ElectronicEngineController2, &frame),
        true
    );
    assert_eq!(
        engine_service.engine_torque_mode.unwrap(),
        EngineTorqueMode::PTOGovernor
    );
    assert_eq!(engine_service.driver_demand.unwrap(), 20);
    assert_eq!(engine_service.actual_engine.unwrap(), 20);
    assert_eq!(engine_service.rpm.unwrap(), 789);
    assert_eq!(engine_service.source_addr.unwrap(), 0);
    assert_eq!(
        engine_service.starter_mode.unwrap(),
        EngineStarterMode::StartFinished
    );
}