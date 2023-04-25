use glonax_j1939::{
    decode::{EngineStarterMode, EngineTorqueMode},
    *,
};

use super::Routable;

// TODO: Rename to EMS
pub struct EngineService {
    pub node: u8,
    pub engine_torque_mode: Option<EngineTorqueMode>,
    pub driver_demand: Option<u8>,
    pub actual_engine: Option<u8>,
    pub rpm: Option<u16>,
    pub source_addr: Option<u8>,
    pub starter_mode: Option<EngineStarterMode>,
}

impl Routable for EngineService {
    fn ingress(&mut self, frame: &Frame) -> bool {
        if frame.len() != 8 {
            return false;
        }
        if frame.id().pgn() != PGN::ElectronicEngineController2 {
            return false;
        }
        if frame.id().sa() != self.node {
            return false;
        }

        self.engine_torque_mode = decode::spn899(frame.pdu()[0]);
        self.driver_demand = decode::spn512(frame.pdu()[1]);
        self.actual_engine = decode::spn513(frame.pdu()[2]);
        self.rpm = decode::spn190(&frame.pdu()[3..5].try_into().unwrap());
        self.source_addr = decode::spn1483(frame.pdu()[5]);
        self.starter_mode = decode::spn1675(frame.pdu()[6]);

        true
    }

    fn encode(&self) -> Vec<Frame> {
        let mut frames = vec![];

        let mut frame_builder = FrameBuilder::new(
            IdBuilder::from_pgn(PGN::ElectronicEngineController2)
                .sa(self.node)
                .build(),
        );

        if let Some(driver_demand) = self.driver_demand {
            frame_builder.as_mut()[1] = driver_demand + 125;
        }
        if let Some(actual_engine) = self.actual_engine {
            frame_builder.as_mut()[2] = actual_engine + 125;
        }

        if let Some(rpm) = self.rpm {
            let rpm_bytes = (rpm * 8).to_le_bytes();
            frame_builder.as_mut()[3..5].copy_from_slice(&rpm_bytes);
        }

        if let Some(source_addr) = self.source_addr {
            frame_builder.as_mut()[5] = source_addr;
        }

        frames.push(frame_builder.set_len(8).build());

        frames
    }
}

impl std::fmt::Display for EngineService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();

        if let Some(engine_torque_mode) = &self.engine_torque_mode {
            s.push_str(&format!("Torque mode: {:?}; ", engine_torque_mode));
        }

        if let Some(driver_demand) = self.driver_demand {
            s.push_str(&format!("Driver Demand: {}%; ", driver_demand));
        }

        if let Some(actual_engine) = self.actual_engine {
            s.push_str(&format!("Actual Engine: {}%; ", actual_engine));
        }

        if let Some(rpm) = self.rpm {
            s.push_str(&format!("RPM: {}; ", rpm));
        }

        if let Some(starter_mode) = &self.starter_mode {
            s.push_str(&format!("Starter mode: {:?}; ", starter_mode));
        }

        write!(f, "{}", s)
    }
}

impl crate::channel::BroadcastSource<crate::transport::Signal> for EngineService {
    fn fetch(&self, writer: &crate::channel::BroadcastChannelWriter<crate::transport::Signal>) {
        if let Some(driver_demand) = self.driver_demand {
            writer
                .send(crate::transport::Signal::new(
                    self.node as u32,
                    1,
                    crate::transport::signal::Metric::Percent(driver_demand as i32),
                ))
                .ok();
        }
        if let Some(actual_engine) = self.actual_engine {
            writer
                .send(crate::transport::Signal::new(
                    self.node as u32,
                    2,
                    crate::transport::signal::Metric::Percent(actual_engine as i32),
                ))
                .ok();
        }
        if let Some(rpm) = self.rpm {
            writer
                .send(crate::transport::Signal::new(
                    self.node as u32,
                    0,
                    crate::transport::signal::Metric::Rpm(rpm as i32),
                ))
                .ok();
        }
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turn_on() {
        let mut engine_service = EngineService::new(0x0);

        let frame =
            FrameBuilder::new(IdBuilder::from_pgn(PGN::ElectronicEngineController2).build())
                .copy_from_slice(&[0xF0, 0xEA, 0x7D, 0x00, 0x00, 0x00, 0xF0, 0xFF])
                .build();
        assert_eq!(engine_service.ingress(&frame), true);
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
    fn turn_off() {
        let mut engine_service = EngineService::new(0x0);

        let frame =
            FrameBuilder::new(IdBuilder::from_pgn(PGN::ElectronicEngineController2).build())
                .copy_from_slice(&[0xF3, 0x91, 0x91, 0xAA, 0x18, 0x00, 0xF3, 0xFF])
                .build();

        assert_eq!(engine_service.ingress(&frame), true);
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
}
