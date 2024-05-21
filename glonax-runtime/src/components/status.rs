use crate::runtime::{CommandSender, Component, ComponentContext};

pub struct StatusComponent {}

impl<Cnf: Clone> Component<Cnf> for StatusComponent {
    fn new(_: Cnf) -> Self
    where
        Self: Sized,
    {
        Self {}
    }

    fn tick(
        &mut self,
        _ctx: &mut ComponentContext,
        _ipc_rx: std::rc::Rc<crate::runtime::IPCReceiver>,
        _command_tx: CommandSender,
    ) {
        // // TODO: Report all statuses, not just a single one
        // /// Get the status of the machine.
        // ///
        // /// This method returns the status of the machine based on the current machine state. It takes
        // /// into account the status of the vehicle management system, global navigation satellite system,
        // /// engine, and other factors.
        // pub fn status(&self) -> core::Status {
        // core::Status::Healthy
        // use crate::core::{HostStatus, Status};

        // let  status = Status::Healthy;

        // match self.state.vms_signal.status {
        //     HostStatus::MemoryLow => {
        //         status = Status::Degraded;
        //     }
        //     HostStatus::CPUHigh => {
        //         status = Status::Degraded;
        //     }
        //     _ => {}
        // }

        // if let GnssStatus::DeviceNotFound = self.state.gnss_signal.status {
        //     status = Status::Faulty;
        // }

        // match self.state.engine.status {
        //     EngineStatus::NetworkDown => {
        //         status = Status::Faulty;
        //     }
        //     EngineStatus::MessageTimeout => {
        //         status = Status::Degraded;
        //     }
        //     _ => {}
        // }

        // status
        // }
    }
}
