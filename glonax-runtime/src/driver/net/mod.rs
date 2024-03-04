pub mod bosch_ems;
pub mod encoder;
pub mod engine;
pub mod fuzzer;
pub mod hydraulic;
pub mod inclino;
pub mod inspector;
pub mod reqres;
pub(super) mod vecraft;
pub mod volvo_ems;
mod volvo_vecu;

pub enum NetDriver {
    KueblerEncoder(super::KueblerEncoder),
    KueblerInclinometer(super::KueblerInclinometer),
    VolvoD7E(super::VolvoD7E),
    BoschEngineManagementSystem(super::BoschEngineManagementSystem),
    HydraulicControlUnit(super::HydraulicControlUnit),
    RequestResponder(super::RequestResponder),
}

impl NetDriver {
    pub fn kuebler_encoder(address: u8, vms_address: u8) -> crate::driver::net::NetDriver {
        crate::driver::net::NetDriver::KueblerEncoder(crate::driver::KueblerEncoder::new(
            address,
            vms_address,
        ))
    }

    pub fn kuebler_inclinometer(address: u8, vms_address: u8) -> crate::driver::net::NetDriver {
        crate::driver::net::NetDriver::KueblerInclinometer(crate::driver::KueblerInclinometer::new(
            address,
            vms_address,
        ))
    }

    pub fn volvo_d7e(address: u8, vms_address: u8) -> crate::driver::net::NetDriver {
        crate::driver::net::NetDriver::VolvoD7E(crate::driver::VolvoD7E::new(address, vms_address))
    }

    // TODO: Renamw ro laixer_hcu
    pub fn hydraulic_control_unit(address: u8, vms_address: u8) -> crate::driver::net::NetDriver {
        crate::driver::net::NetDriver::HydraulicControlUnit(
            crate::driver::HydraulicControlUnit::new(address, vms_address),
        )
    }

    pub fn request_responder(address: u8) -> crate::driver::net::NetDriver {
        crate::driver::net::NetDriver::RequestResponder(crate::driver::RequestResponder::new(
            address,
        ))
    }
}

// FUTURE: Maybe move to runtime?
pub trait J1939Unit {
    /// Try to accept a message from the router.
    ///
    /// This method will try to accept a message from the router. If the router has a message
    /// available, the message will be parsed and the unit will be updated accordingly.
    fn try_accept(
        &mut self,
        router: &crate::net::Router,
        runtime_state: crate::runtime::SharedOperandState,
    ) -> impl std::future::Future<Output = ()> + Send;

    /// Tick the unit on interval.
    ///
    /// This method will be called on interval to allow the unit to perform any necessary
    /// operations. This method should be non-blocking and should only perform asynchronous
    /// I/O operations.
    ///
    /// This method is optional and may be a no-op.
    fn tick(
        &self,
        _router: &crate::net::Router,
        _runtime_state: crate::runtime::SharedOperandState,
    ) -> impl std::future::Future<Output = ()> + Send {
        std::future::ready(())
    }
}
