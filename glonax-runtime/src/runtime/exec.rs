use std::time::Instant;

use crate::{
    core::motion::Motion,
    runtime::{self, operand::FunctionTrait, program},
    ProgramConfig, RuntimeContext,
};

use super::operand::{FunctionFactory, Operand};

pub async fn exec_service<K: Operand + FunctionFactory + 'static>(
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
        let function = tokio::select! {
            function = program_manager.recv() => {
                Ok(function)
            }
            _ = runtime.shutdown.1.recv() => {
                Err(())
            }
        };

        if let Ok(Some(function_argument)) = function {
            let mut function = match runtime.operand.fetch_function(&function_argument) {
                Ok(function) => function,
                Err(_) => {
                    warn!(
                        "Function {} was not registered with the operand",
                        function_argument.name()
                    );
                    continue;
                }
            };

            info!("Execute function: {}", function_argument);

            motion_publisher.publish(Motion::ResumeAll).await; // TOOD: Handle result

            let mut ctx = program::Context::new(&mut signal_manager);
            if let Some(motion) = function.boot(&mut ctx) {
                motion_publisher.publish(motion).await; // TOOD: Handle result
            };

            // Loop until this function reaches its termination condition. If
            // the function does not terminate we'll run until the application is killed.
            while !function.can_terminate(&mut ctx) {
                let start_step_execute = Instant::now();

                tokio::select! {
                    // Query the operand function for the next motion step. The
                    // entire thread is dedicated to the function therefore steps
                    // can claim an unlimited time slice.
                    plan = function.step(&mut ctx) => {
                        if let Some(motion) = plan {
                            motion_publisher.publish(motion).await; // TOOD: Handle result
                        }
                    }
                    _ = runtime.shutdown.1.recv() => {
                        // Stop all motion for safety.
                        motion_publisher.publish(Motion::StopAll).await; // TOOD: Handle result

                        warn!("Function {} terminated by external signal", function_argument.name());

                        return Ok(());
                    }
                };

                ctx.step_count += 1;
                ctx.last_step = start_step_execute;
            }

            // Execute an optional last action before function termination.
            if let Some(motion) = function.term_action(&mut ctx) {
                motion_publisher.publish(motion).await; // TOOD: Handle result
            }

            info!(
                "Function {} terminated with success",
                function_argument.name()
            );
        } else {
            // Stop all motion for safety.
            motion_publisher.publish(Motion::StopAll).await; // TOOD: Handle result

            break;
        }
    }

    Ok(())
}
