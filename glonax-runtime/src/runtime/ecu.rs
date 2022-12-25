use crate::{
    runtime::{self},
    EcuConfig, RuntimeContext,
};

use runtime::operand::Operand;

pub(crate) async fn exec_service<K: Operand>(
    config: &EcuConfig,
    mut runtime: RuntimeContext<K>,
) -> runtime::Result {
    use crate::device::CoreDevice;

    let signal_manager = runtime.new_signal_manager();

    let mut gateway = runtime
        .new_network_gateway(&config.interface, &signal_manager)
        .await?;
    let motion_manager = runtime.new_motion_manager();

    runtime
        .eventhub
        .subscribe(motion_manager.adapter(gateway.hcu()));

    tokio::task::spawn(async move {
        loop {
            runtime.eventhub.next().await
        }
    });

    tokio::task::spawn(async move {
        loop {
            gateway.next().await.unwrap();
        }
    });

    runtime.shutdown.1.recv().await.unwrap();

    Ok(())
}
