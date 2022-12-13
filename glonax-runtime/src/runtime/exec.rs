use std::time::Instant;

use crate::{
    core::motion::Motion,
    runtime::{self, program},
    ProgramConfig, RuntimeContext,
};

use super::operand::{Operand, ProgramFactory};

pub async fn exec_service<K: Operand + ProgramFactory>(
    _config: &ProgramConfig,
    mut runtime: RuntimeContext<K>,
) -> runtime::Result {
    let mut program_manager = runtime.new_program_manager();
    let motion_manager = runtime.new_motion_manager();
    let mut signal_manager = runtime.new_signal_manager();

    runtime.eventhub.subscribe(program_manager.adapter());
    runtime.eventhub.subscribe(signal_manager.adapter());

    let motion_publisher = motion_manager.publisher();

    tokio::task::spawn(async move {
        loop {
            runtime.eventhub.next().await
        }
    });

    info!("Execute programs on queue");

    loop {
        let program = tokio::select! {
            program = program_manager.recv() => {
                Ok(program)
            }
            _ = runtime.shutdown.1.recv() => {
                Err(())
            }
        };

        if let Ok(Some(program_argument)) = program {
            let mut program = match runtime.operand.fetch_program(&program_argument) {
                Ok(program) => program,
                Err(_) => {
                    warn!(
                        "Program ({}) was not registered with the operand",
                        program_argument.id
                    );
                    continue;
                }
            };

            info!("Start program ({})", program_argument.id);

            motion_publisher.publish(Motion::ResumeAll).await; // TOOD: Handle result

            let mut ctx = program::Context::new(&mut signal_manager);
            if let Some(motion) = program.boot(&mut ctx) {
                motion_publisher.publish(motion).await; // TOOD: Handle result
            };

            // Loop until this program reaches its termination condition. If
            // the program does not terminate we'll run until the application is killed.
            while !program.can_terminate(&mut ctx) {
                let start_step_execute = Instant::now();

                tokio::select! {
                    // Query the operand program for the next motion step. The
                    // entire thread is dedicated to the program therefore steps
                    // can claim an unlimited time slice.
                    plan = program.step(&mut ctx) => {
                        if let Some(motion) = plan {
                            motion_publisher.publish(motion).await; // TOOD: Handle result
                        }
                    }
                    _ = runtime.shutdown.1.recv() => {
                        // Stop all motion for safety.
                        motion_publisher.publish(Motion::StopAll).await; // TOOD: Handle result

                        warn!("Program ({}) terminated by external signal", program_argument.id);

                        return Ok(());
                    }
                };

                ctx.step_count += 1;
                ctx.last_step = start_step_execute;
            }

            // Execute an optional last action before program termination.
            if let Some(motion) = program.term_action(&mut ctx) {
                motion_publisher.publish(motion).await; // TOOD: Handle result
            }

            info!("Program ({}) terminated with success", program_argument.id);
        } else {
            // Stop all motion for safety.
            motion_publisher.publish(Motion::StopAll).await; // TOOD: Handle result

            break;
        }
    }

    Ok(())
}
