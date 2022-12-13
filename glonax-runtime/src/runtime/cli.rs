use crate::{core::program::ProgramArgument, runtime, InputConfig, RuntimeContext};

use super::operand::Operand;

pub(crate) async fn exec_service<K: Operand>(
    _config: &InputConfig,
    mut runtime: RuntimeContext<K>,
) -> runtime::Result {
    let mut program_manager = runtime.new_program_manager();

    tokio::task::spawn(async move {
        loop {
            runtime.eventhub.next().await
        }
    });

    program_manager
        .publish(ProgramArgument {
            id: 900,
            parameters: vec![10.0],
        })
        .await;

    info!("Program 901 submitted");

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    Ok(())
}
