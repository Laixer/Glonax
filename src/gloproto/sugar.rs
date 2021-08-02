use std::convert::{TryFrom, TryInto};

use serde::Deserialize;

use super::Body;

/// Predefined subcommands.
enum SubCommand {
    // Sensors.
    SensorTemperature = 0xe8,
    SensorAcceleration = 0xe9,
    SensorOrientation = 0xea,
    SensorDirection = 0xeb,
    SensorDistance = 0xec,
    SensorPressure = 0xed,
    SensorElevation = 0xee,

    // Switches.
    Switch2way = 0xc2,
    Switch3way = 0xc3,

    // Pwm.
    PwmPin = 0xf0,
}

impl TryFrom<u16> for SubCommand {
    type Error = ();

    fn try_from(v: u16) -> Result<Self, Self::Error> {
        match v {
            x if x == SubCommand::SensorTemperature as u16 => Ok(SubCommand::SensorTemperature),
            x if x == SubCommand::SensorAcceleration as u16 => Ok(SubCommand::SensorAcceleration),
            x if x == SubCommand::SensorOrientation as u16 => Ok(SubCommand::SensorOrientation),
            x if x == SubCommand::SensorDirection as u16 => Ok(SubCommand::SensorDirection),
            x if x == SubCommand::SensorDistance as u16 => Ok(SubCommand::SensorDistance),
            x if x == SubCommand::SensorPressure as u16 => Ok(SubCommand::SensorPressure),
            x if x == SubCommand::SensorElevation as u16 => Ok(SubCommand::SensorElevation),
            x if x == SubCommand::Switch2way as u16 => Ok(SubCommand::Switch2way),
            x if x == SubCommand::Switch3way as u16 => Ok(SubCommand::Switch3way),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum PacketError {
    DeviceNotRead = -100,
    InvalidPayloadSize = -101,
}

impl TryFrom<i8> for PacketError {
    type Error = ();

    fn try_from(v: i8) -> Result<Self, Self::Error> {
        match v {
            x if x == PacketError::DeviceNotRead as i8 => Ok(PacketError::DeviceNotRead),
            x if x == PacketError::InvalidPayloadSize as i8 => Ok(PacketError::InvalidPayloadSize),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub enum Sugar {
    Temperature(i16),
    Acceleration(i32, i32, i32),
    Orientation(i32, i32, i32),
    Direction(i32, i32, i32),
    PulsePin(u8, i16),
}

impl Sugar {
    pub fn to_body(self) -> Body {
        match self {
            Sugar::Temperature(_) => todo!(),
            Sugar::Acceleration(_, _, _) => todo!(),
            Sugar::Orientation(_, _, _) => todo!(),
            Sugar::Direction(_, _, _) => todo!(),
            Sugar::PulsePin(pin, value) => {
                let mut buf = Vec::with_capacity(3);
                buf.push(pin);
                buf.extend(value.to_le_bytes());
                Body::Custom {
                    subcommand: SubCommand::PwmPin as u16,
                    flags: 0,
                    payload: buf,
                }
            }
        }
    }

    pub fn parse(subcommand: u16, _: u16, payload: &Vec<u8>) -> std::result::Result<Self, ()> {
        // TODO: Parse without serde support.
        #[derive(Deserialize)]
        struct GloprotoVector {
            pub x: i32,
            pub y: i32,
            pub z: i32,
        }

        // TODO: Parse without serde support.
        #[derive(Deserialize)]
        struct GloprotoTemp {
            temp: i16,
        }

        match subcommand.try_into().unwrap() {
            SubCommand::SensorTemperature => match bincode::deserialize::<GloprotoTemp>(&payload) {
                Ok(temp) => Ok(Sugar::Temperature(temp.temp)),
                Err(_) => Err(()),
            },
            SubCommand::SensorAcceleration => {
                match bincode::deserialize::<GloprotoVector>(&payload) {
                    Ok(vector) => Ok(Sugar::Acceleration(vector.x, vector.y, vector.z)),
                    Err(_) => Err(()),
                }
            }
            SubCommand::SensorOrientation => {
                match bincode::deserialize::<GloprotoVector>(&payload) {
                    Ok(vector) => Ok(Sugar::Orientation(vector.x, vector.y, vector.z)),
                    Err(_) => Err(()),
                }
            }
            SubCommand::SensorDirection => match bincode::deserialize::<GloprotoVector>(&payload) {
                Ok(vector) => Ok(Sugar::Direction(vector.x, vector.y, vector.z)),
                Err(_) => Err(()),
            },
            _ => Err(()),
        }
    }
}
