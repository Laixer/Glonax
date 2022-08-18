use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

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
    queue: VecDeque<(i32, Parameter)>,
}

impl RuntimeProgram {
    pub fn new(config: &crate::config::ProgramConfig) -> Self {
        let mut queue = VecDeque::with_capacity(config.program_queue);

        queue.push_front((902, vec![]));

        if let Some(id) = config.program_id {
            queue.push_front((id, vec![]));
        } else {
            // queue.push_front((910, vec![]));

            queue.push_front((603, vec![-1.73, 1.01]));
            queue.push_front((603, vec![-1.31, 0.87]));
            queue.push_front((603, vec![-0.56, 0.74]));
            queue.push_front((603, vec![-0.19, 0.46]));
            queue.push_front((603, vec![-0.82, 0.40]));
            queue.push_front((603, vec![-1.77, 0.36]));
            queue.push_front((603, vec![-2.09, 0.63]));
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

        while let Some((id, params)) = self.queue.pop_back() {
            let mut program = match runtime.operand.fetch_program(id, params) {
                Ok(program) => program,
                Err(_) => {
                    warn!("Program {} was not registered with the operand", id);
                    continue;
                }
            };

            info!("Start program: {}", id);

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

                // FUTURE: Ensure the step is called *at least* once ever 50ms.
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

                    warn!("Program terminated by external signal");

                    return Ok(());
                }
            }

            // Execute an optional last action before program termination.
            if let Some(motion) = program.term_action(&mut ctx) {
                motion_chain.request(motion).await; // TOOD: Handle result
            }

            // Stop all motion for safety.
            motion_chain.request(Motion::StopAll).await; // TOOD: Handle result

            info!("Program terminated with success");
        }

        Ok(())
    }
}
