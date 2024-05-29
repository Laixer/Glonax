use crate::{
    core::{Object, ObjectType},
    runtime::{ComponentContext, IPCReceiver, InitComponent},
};

pub struct Acquisition {}

impl<Cnf: Clone> InitComponent<Cnf> for Acquisition {
    fn new(_: Cnf) -> Self
    where
        Self: Sized,
    {
        Self {}
    }

    fn init(&self, ctx: &mut ComponentContext, ipc_rx: std::rc::Rc<IPCReceiver>) {
        ctx.machine.emergency = false;

        ctx.machine.vms_signal_set = false;
        ctx.machine.gnss_signal_set = false;
        ctx.machine.engine_signal_set = false;
        ctx.machine.motion_signal_set = false;
        ctx.machine.encoders_set = false;

        ctx.machine.vms_signal_changed = false;
        ctx.machine.gnss_signal_changed = false;
        ctx.machine.engine_signal_changed = false;
        ctx.machine.motion_signal_changed = false;
        ctx.machine.encoders_changed = false;

        for message in ipc_rx.try_iter() {
            log::trace!("Received IPC object: {:?}", message.object);

            match message.object {
                Object::Control(_control) => {}
                Object::Engine(engine) => {
                    if message.object_type == ObjectType::Command {
                        ctx.machine.engine_command = Some(engine);
                        ctx.machine.engine_command_instant = Some(message.timestamp);
                    } else if message.object_type == ObjectType::Signal {
                        if ctx.machine.engine_signal != engine {
                            ctx.machine.engine_signal_changed = true;
                        }
                        ctx.machine.engine_signal = engine;
                        ctx.machine.engine_signal_instant = Some(message.timestamp);
                        ctx.machine.engine_signal_set = true;
                    }
                }
                Object::GNSS(gnss_signal) => {
                    if message.object_type == ObjectType::Signal {
                        if ctx.machine.gnss_signal != gnss_signal {
                            ctx.machine.gnss_signal_changed = true;
                        }
                        ctx.machine.gnss_signal = gnss_signal;
                        ctx.machine.gnss_signal_instant = Some(message.timestamp);
                        ctx.machine.gnss_signal_set = true;
                    }
                }
                Object::Host(vms_signal) => {
                    if message.object_type == ObjectType::Signal {
                        if ctx.machine.vms_signal != vms_signal {
                            ctx.machine.vms_signal_changed = true;
                        }
                        ctx.machine.vms_signal = vms_signal;
                        ctx.machine.vms_signal_instant = Some(message.timestamp);
                        ctx.machine.vms_signal_set = true;
                    }
                }
                Object::Motion(motion) => {
                    if message.object_type == ObjectType::Signal {
                        if ctx.machine.motion_signal != motion {
                            ctx.machine.motion_signal_changed = true;
                        }
                        ctx.machine.motion_signal = motion;
                        ctx.machine.motion_signal_instant = Some(message.timestamp);
                        ctx.machine.motion_signal_set = true;
                    }
                }
                Object::Target(target) => {
                    if message.object_type == ObjectType::Command
                        && ctx.machine.program_command.len() < 1_000
                    {
                        ctx.machine.program_command.push_back(target);
                    }
                }
                Object::Encoder((id, value)) => {
                    if ctx.machine.encoders.get(&id) != Some(&value) {
                        ctx.machine.encoders_changed = true;
                    }
                    ctx.machine.encoders.insert(id, value);
                    ctx.machine.encoders_instant = Some(message.timestamp);
                    ctx.machine.encoders_set = true;
                }
            }
        }
    }
}
