use std::time::{Duration, Instant};

use tokio::sync::mpsc;

use crate::{
    core::{motion::Motion, TraceWriter, Tracer},
    runtime::{self, operand},
    Runtime,
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

pub struct RuntimeProgram {
    queue: (
        mpsc::Sender<(i32, Parameter)>,
        mpsc::Receiver<(i32, Parameter)>,
    ),
}

impl RuntimeProgram {
    pub async fn new(config: &crate::config::ProgramConfig) -> Self {
        let queue = mpsc::channel(config.program_queue);

        queue.0.send((902, vec![])).await.ok();

        if let Some(id) = config.program_id {
            queue.0.send((id, vec![])).await.ok();
        } else {
            queue.0.send((603, vec![-1.73, 1.01])).await.ok();
            queue.0.send((603, vec![-1.31, 0.87])).await.ok();
            queue.0.send((603, vec![-0.56, 0.74])).await.ok();
            queue.0.send((603, vec![-0.19, 0.46])).await.ok();
            queue.0.send((603, vec![-0.82, 0.40])).await.ok();
            queue.0.send((603, vec![-1.77, 0.36])).await.ok();
            queue.0.send((603, vec![-2.09, 0.63])).await.ok();
        }

        Self { queue }
    }

    pub async fn exec_service<K>(mut self, mut runtime: Runtime<K>) -> runtime::Result
    where
        K: Operand + ProgramFactory,
    {
        let mut motion_chain = MotionChain::new(&mut runtime.motion_device, &runtime.tracer);

        let mut program_tracer = runtime.tracer.instance("program");

        info!("Execute programs on queue");

        loop {
            let program = tokio::select! {
                p = self.queue.1.recv() => {
                    Ok(p)
                }
                _ = runtime.shutdown.1.recv() => {
                    Err(())
                }
            };

            if let Ok(program) = program {
                if let Some((id, params)) = program {
                    let mut program = match runtime.operand.fetch_program(id, params) {
                        Ok(program) => program,
                        Err(_) => {
                            warn!("Program ({}) was not registered with the operand", id);
                            continue;
                        }
                    };

                    info!("Start program ({})", id);

                    program_tracer.write_record(ProgramTrace {
                        timestamp: crate::core::time::now().as_millis(),
                        id,
                    });

                    motion_chain.request(Motion::ResumeAll).await; // TOOD: Handle result

                    let mut ctx = operand::Context::new(runtime.signal_manager.reader());
                    program.boot(&mut ctx);

                    // Loop until this program reaches its termination condition. If
                    // the program does not terminate we'll run forever.
                    while !program.can_terminate(&mut ctx) {
                        // Deliberately slow down the program loop to limit CPU cycles.
                        // If the delay is small then this won't effect the program
                        // procession.
                        tokio::time::sleep(Duration::from_millis(2)).await;

                        let start_step_execute = Instant::now();

                        // TODO: Make step awaitable.
                        // Query the operand program for the next motion step. The
                        // entire thread is dedicated to the program therefore steps
                        // can take as long as they require.
                        if let Some(motion) = program.step(&mut ctx) {
                            motion_chain.request(motion).await; // TOOD: Handle result
                        }

                        ctx.step_count += 1;
                        ctx.last_step = start_step_execute;

                        if !runtime.shutdown.1.is_empty() {
                            // Stop all motion for safety.
                            motion_chain.request(Motion::StopAll).await; // TOOD: Handle result

                            warn!("Program ({}) terminated by external signal", id);

                            return Ok(());
                        }
                    }

                    // Execute an optional last action before program termination.
                    if let Some(motion) = program.term_action(&mut ctx) {
                        motion_chain.request(motion).await; // TOOD: Handle result
                    }

                    // Stop all motion for safety.
                    motion_chain.request(Motion::StopAll).await; // TOOD: Handle result

                    info!("Program ({}) terminated with success", id);
                }
            } else {
                // Stop all motion for safety.
                motion_chain.request(Motion::StopAll).await; // TOOD: Handle result

                break;
            }
        }

        Ok(())
    }
}
