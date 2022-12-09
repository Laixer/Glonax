use crate::{
    device::{Gamepad, InputDevice},
    runtime, InputConfig, RuntimeContext,
};

use super::operand::Operand;

pub(crate) async fn exec_service<K: Operand>(
    config: &InputConfig,
    mut runtime: RuntimeContext<K>,
) -> runtime::Result {
    let mut input_device = Gamepad::new(std::path::Path::new(&config.device)).await;

    let motion_publisher = runtime.new_motion_manager().publisher();

    tokio::task::spawn(async move {
        loop {
            runtime.eventhub.next().await
        }
    });

    while let Ok(input) = input_device.next().await {
        if let Ok(motion) = runtime.operand.try_from_input_device(input) {
            motion_publisher.publish(motion).await;
        }
    }

    Ok(())
}
