use std::time::Instant;

use tokio::sync::mpsc;

use crate::{
    core::{motion::Motion, TraceWriter, Tracer},
    device::{Hcu, Mecu},
    runtime::{self, operand},
    ProgramConfig, RuntimeContext,
};

use super::{
    operand::{Operand, Parameter, ProgramFactory},
    MotionChain,
};

#[derive(serde::Serialize)]
struct ProgramTrace {
    /// Timestamp of the trace.
    timestamp: u128,
    /// Program identifier.
    id: i32,
}

impl ProgramTrace {
    fn now(id: i32) -> Self {
        Self {
            timestamp: crate::core::time::now().as_millis(),
            id,
        }
    }
}

pub struct RuntimeProgram {
    queue: (
        mpsc::Sender<(i32, Parameter)>,
        mpsc::Receiver<(i32, Parameter)>,
    ),
}

impl RuntimeProgram {
    pub async fn new(config: &ProgramConfig) -> Self {
        let queue = mpsc::channel(config.program_queue);

        use crate::kernel::excavator::ProgramSegment as Program;

        queue.0.send((Program::Sleep.into(), vec![0.5])).await.ok();

        if let Some(id) = config.program_id {
            queue.0.send((id, vec![])).await.ok();
        } else {
            // queue
            //     .0
            //     .send((Program::Kinematic.into(), [2.71, 2.34, 0.0].into()))
            //     .await
            //     .ok();
            queue
                .0
                .send((Program::Turn.into(), [1.57].into()))
                .await
                .ok();

            // queue.0.send((Program::Kinematic.into(), vec![-1.31, 0.87, 0.0])).await.ok();
            // queue.0.send((Program::Kinematic.into(), vec![-0.56, 0.74, 0.0])).await.ok();
            // queue.0.send((Program::Kinematic.into(), vec![-0.19, 0.46, 0.0])).await.ok();
            // queue.0.send((Program::Kinematic.into(), vec![-0.82, 0.40, 0.0])).await.ok();
            // queue.0.send((Program::Kinematic.into(), vec![-1.77, 0.36, 0.0])).await.ok();
            // queue.0.send((Program::Kinematic.into(), vec![-2.09, 0.63, 0.0])).await.ok();
        }

        Self { queue }
    }

    pub async fn exec_service<K: Operand + ProgramFactory>(
        mut self,
        config: &ProgramConfig,
        mut runtime: RuntimeContext<K>,
    ) -> runtime::Result {
        use crate::device::CoreDevice;

        let signal_device = Mecu::new(runtime.signal_manager.pusher());
        runtime.core_device.subscribe(signal_device);

        let mut motion_device = runtime.core_device.new_gateway_device::<Hcu>();

        let mut motion_chain = MotionChain::new(&mut motion_device, &runtime.tracer)
            .enable(config.global.enable_motion);

        let mut program_tracer = runtime.tracer.instance("program");

        tokio::task::spawn(async move { while runtime.core_device.next().await.is_ok() {} });

        info!("Execute programs on queue");

        loop {
            let program = tokio::select! {
                program = self.queue.1.recv() => {
                    Ok(program)
                }
                _ = runtime.shutdown.1.recv() => {
                    Err(())
                }
            };

            if let Ok(Some((id, params))) = program {
                let mut program = match runtime.operand.fetch_program(id, params) {
                    Ok(program) => program,
                    Err(_) => {
                        warn!("Program ({}) was not registered with the operand", id);
                        continue;
                    }
                };

                info!("Start program ({})", id);

                program_tracer.write_record(ProgramTrace::now(id));

                motion_chain.request(Motion::ResumeAll).await; // TOOD: Handle result

                let mut ctx = operand::Context::new(runtime.signal_manager.reader());
                if let Some(motion) = program.boot(&mut ctx) {
                    motion_chain.request(motion).await; // TOOD: Handle result
                };

                // Loop until this program reaches its termination condition. If
                // the program does not terminate we'll run until the application is killed.
                while !program.can_terminate(&mut ctx) {
                    let start_step_execute = Instant::now();

                    tokio::select! {
                        // Query the operand program for the next motion step. The
                        // entire thread is dedicated to the program therefore steps
                        // can claim an unlimited time slice.
                        p = program.step(&mut ctx) => {
                            if let Some(motion) = p {
                                motion_chain.request(motion).await; // TOOD: Handle result
                            }
                        }
                        _ = runtime.shutdown.1.recv() => {
                            // Stop all motion for safety.
                            motion_chain.request(Motion::StopAll).await; // TOOD: Handle result

                            warn!("Program ({}) terminated by external signal", id);

                            return Ok(());
                        }
                    };

                    ctx.step_count += 1;
                    ctx.last_step = start_step_execute;
                }

                // Execute an optional last action before program termination.
                if let Some(motion) = program.term_action(&mut ctx) {
                    motion_chain.request(motion).await; // TOOD: Handle result
                }

                info!("Program ({}) terminated with success", id);
            } else {
                // Stop all motion for safety.
                motion_chain.request(Motion::StopAll).await; // TOOD: Handle result

                break;
            }
        }

        Ok(())
    }
}
