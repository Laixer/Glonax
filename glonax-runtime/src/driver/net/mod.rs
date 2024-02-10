pub mod encoder;
pub mod engine;
pub mod hydraulic;
pub mod inspector;
pub mod reqres;
pub(super) mod vecraft;

// FUTURE: Maybe move?
pub trait J1939Unit {
    /// Try to accept a message from the router.
    ///
    /// This method will try to accept a message from the router. If the router has a message
    /// available, the message will be parsed and the unit will be updated accordingly.
    fn try_accept(
        &mut self,
        router: &mut crate::net::Router,
        runtime_state: crate::runtime::SharedOperandState,
    );
}
