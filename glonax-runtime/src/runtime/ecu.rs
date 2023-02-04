use crate::{runtime, EcuConfig, RuntimeContext};

use runtime::operand::Operand;

pub(crate) async fn exec_service<K: Operand>(
    config: &EcuConfig,
    mut runtime: RuntimeContext<K>,
) -> runtime::Result {
    use crate::device::{CoreDevice, Gateway};

    let signal_manager = runtime.new_signal_manager();
    let motion_manager = runtime.new_motion_manager();

    let mut gateway = Gateway::new(&config.interface, &signal_manager)
        .map_err(|_| runtime::Error::CoreDeviceNotFound)?;

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
