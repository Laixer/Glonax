use std::cell::RefCell;

use crate::{
    core::{Object, Rotator},
    driver::{EncoderConverter, VirtualEncoder},
    net::Parsable,
    runtime::{J1939Unit, J1939UnitError, NetDriverContext},
};

#[derive(Clone)]
pub struct Simulator {
    /// Network interface.
    interface: String,
    /// Destination address.
    destination_address: u8,
    /// Source address.
    source_address: u8,
    /// List of encoders.
    encoder_list: [(u8, crate::core::Actuator, VirtualEncoder, EncoderConverter); 4],
    /// List of encoder velocities.
    velocity_list: [RefCell<i16>; 4],
    /// List of encoder positions.
    position_list: [RefCell<u32>; 4],
}

impl Simulator {
    /// Construct a new encoder service.
    pub fn new(interface: &str, da: u8, sa: u8) -> Self {
        let encoder_frame = VirtualEncoder::new(2_500, (0, 6_280), true, false);
        let encoder_boom = VirtualEncoder::new(5_000, (0, 1_832), false, false);
        let encoder_arm = VirtualEncoder::new(5_000, (685, 2_760), false, true);
        let encoder_attachment = VirtualEncoder::new(5_000, (0, 3_100), false, false);

        let decoder_frame = EncoderConverter::new(1000.0, 0.0, true, nalgebra::Vector3::z_axis());
        let decoder_boom = EncoderConverter::new(
            1000.0,
            60_f32.to_radians(),
            true,
            nalgebra::Vector3::y_axis(),
        );
        let decoder_arm = EncoderConverter::new(1000.0, 0.0, true, nalgebra::Vector3::y_axis());
        let decoder_attachment =
            EncoderConverter::new(1000.0, 0.0, true, nalgebra::Vector3::y_axis());

        let encoder_list = [
            (
                0x6A,
                crate::core::Actuator::Slew,
                encoder_frame,
                decoder_frame,
            ),
            (
                0x6B,
                crate::core::Actuator::Boom,
                encoder_boom,
                decoder_boom,
            ),
            (0x6C, crate::core::Actuator::Arm, encoder_arm, decoder_arm),
            (
                0x6D,
                crate::core::Actuator::Attachment,
                encoder_attachment,
                decoder_attachment,
            ),
        ];

        Self {
            interface: interface.to_string(),
            destination_address: da,
            source_address: sa,
            encoder_list,
            velocity_list: Default::default(),
            position_list: Default::default(),
        }
    }
}

impl J1939Unit for Simulator {
    fn vendor(&self) -> &'static str {
        "laixer"
    }

    fn product(&self) -> &'static str {
        "simulator"
    }

    fn destination(&self) -> u8 {
        self.destination_address
    }

    fn source(&self) -> u8 {
        self.source_address
    }

    fn try_recv(
        &self,
        _ctx: &mut NetDriverContext,
        frame: &j1939::Frame,
        rx_queue: &mut Vec<Object>,
    ) -> Result<(), J1939UnitError> {
        let hcu0 = crate::driver::HydraulicControlUnit::new(&self.interface, 0x4a, 0x27);

        let message = hcu0.parse(frame).unwrap();

        match message {
            crate::driver::net::hydraulic::HydraulicMessage::Actuator(actuator) => {
                if let Some(value) = actuator.actuators[0] {
                    *self.velocity_list[1].borrow_mut() = value;
                }

                if let Some(value) = actuator.actuators[1] {
                    *self.velocity_list[0].borrow_mut() = value;
                }

                if let Some(value) = actuator.actuators[4] {
                    *self.velocity_list[2].borrow_mut() = value;
                }

                if let Some(value) = actuator.actuators[5] {
                    *self.velocity_list[3].borrow_mut() = value;
                }
            }
            crate::driver::net::hydraulic::HydraulicMessage::MotionConfig(motion) => {
                if let Some(true) = motion.locked {
                    for v in self.velocity_list.iter() {
                        *v.borrow_mut() = 0;
                    }
                }

                if let Some(true) = motion.reset {
                    for v in self.velocity_list.iter() {
                        *v.borrow_mut() = 0;
                    }
                }
            }
            _ => {}
        }

        // TOOD: Run this on every tick
        for (idx, encoder) in self.encoder_list.iter().enumerate() {
            let current_velocity = self.velocity_list[idx].borrow();
            let mut current_position = self.position_list[idx].borrow_mut();

            let new_position = encoder.2.position(*current_position, *current_velocity);

            *current_position = new_position;

            // DECODE POSITION

            let rotation = encoder.3.to_rotation(new_position as f32);
            let rotator = Rotator::relative(encoder.0, rotation);

            rx_queue.push(Object::Rotator(rotator));

            trace!(
                "Actuator: 0x{:X} Roll={:.2} Pitch={:.2} Yaw={:.2}",
                encoder.0,
                rotation.euler_angles().0.to_degrees(),
                rotation.euler_angles().1.to_degrees(),
                rotation.euler_angles().2.to_degrees()
            );
        }

        Ok(())
    }
}
