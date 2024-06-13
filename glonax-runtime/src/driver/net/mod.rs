use sim::Simulator;
use vcu::VehicleControlUnit;

use super::{
    EngineManagementSystem, HydraulicControlUnit, KueblerEncoder, KueblerInclinometer, VolvoD7E,
};

pub mod encoder;
pub mod engine;
pub mod fuzzer;
pub mod hydraulic;
pub mod inclino;
pub mod inspector;
pub mod probe;
pub mod sim;
pub mod vcu;
pub(super) mod vecraft;
pub mod volvo_ems;
mod volvo_vecu;

pub(crate) fn driver_factory(
    vendor: &str,
    product: &str,
    interface: &str,
    da: u8,
    sa: u8,
) -> Option<Box<dyn crate::runtime::J1939Unit>> {
    match (vendor, product) {
        ("laixer", "vcu") => Some(Box::new(VehicleControlUnit::new(interface, da, sa))),
        ("laixer", "hcu") => Some(Box::new(HydraulicControlUnit::new(interface, da, sa))),
        ("laixer", "simulator") => Some(Box::new(Simulator::new(interface, da, sa))),
        ("volvo", "d7e") => Some(Box::new(VolvoD7E::new(interface, da, sa))),
        ("kübler", "inclinometer") => Some(Box::new(KueblerInclinometer::new(interface, da, sa))),
        ("j1939", "ecm") => Some(Box::new(EngineManagementSystem::new(interface, da, sa))),
        ("kübler", "encoder") => Some(Box::new(KueblerEncoder::new(interface, da, sa))),
        _ => None,
    }
}
