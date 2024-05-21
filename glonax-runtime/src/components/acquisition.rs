use crate::runtime::{CommandSender, Component, ComponentContext};

pub struct Acquisition {}

impl<Cnf: Clone> Component<Cnf> for Acquisition {
    fn new(_config: Cnf) -> Self
    where
        Self: Sized,
    {
        Self {}
    }

    fn tick(&mut self, _ctx: &mut ComponentContext, _command_tx: CommandSender) {}

    fn tick2(
        &mut self,
        ctx: &mut ComponentContext,
        ipc_rx: std::rc::Rc<crate::runtime::IPCReceiver>,
        _command_tx: CommandSender,
    ) {
        if let Ok(message) = ipc_rx.try_recv() {
            // log::trace!("Received IPC object: {:?}", object);

            use crate::core::{Object, ObjectType};

            match message.object {
                Object::Control(_control_signal) => {
                    // TODO: Handle control signal
                }
                Object::Engine(engine) => {
                    if message.object_type == ObjectType::Command {
                        ctx.machine.engine_command = Some(engine);
                        ctx.machine.engine_command_instant = Some(message.timestamp);
                    } else if message.object_type == ObjectType::Signal {
                        ctx.machine.engine_signal = engine;
                        ctx.machine.engine_signal_instant = Some(message.timestamp);
                    }
                }
                Object::GNSS(gnss_signal) => {
                    ctx.machine.gnss_signal = gnss_signal;
                    ctx.machine.gnss_signal_instant = Some(message.timestamp);
                }
                Object::Host(vms_signal) => {
                    ctx.machine.vms_signal = vms_signal;
                    ctx.machine.vms_signal_instant = Some(message.timestamp);
                }
                Object::Motion(motion) => {
                    if message.object_type == ObjectType::Command {
                        ctx.machine.motion_command = Some(motion);
                        ctx.machine.motion_command_instant = Some(message.timestamp);
                    } else if message.object_type == ObjectType::Signal {
                        ctx.machine.motion_signal = motion;
                        ctx.machine.motion_signal_instant = Some(message.timestamp);
                    }
                }
                Object::Target(target) => {
                    if ctx.machine.program_command.len() < 1_000 {
                        ctx.machine.program_command.push_back(target);
                    }
                }
            }
        }
    }
}
