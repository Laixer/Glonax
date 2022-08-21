use crate::{
    device::{Hcu, Mecu, Vecu},
    runtime, InputConfig, RuntimeContext,
};

use runtime::operand::Operand;

pub(crate) async fn exec_service<K: Operand>(
    _config: &InputConfig,
    mut runtime: RuntimeContext<K>,
) -> runtime::Result {
    use crate::device::CoreDevice;

    info!("Listen for network events");

    runtime.core_device.new_gateway_device::<Vecu>();
    let mut _motion_device = runtime.core_device.new_gateway_device::<Hcu>();

    let signal_device = Mecu::new(runtime.signal_manager.pusher());
    runtime.core_device.subscribe(signal_device);

    tokio::task::spawn(async move { while runtime.core_device.next().await.is_ok() {} });

    let sock = tokio::net::UdpSocket::bind("0.0.0.0:54910").await.unwrap();
    let mut buf = [0; 1024];

    while let Ok((size, addr)) = sock.recv_from(&mut buf).await {
        debug!("Frame from {} with size {}", addr, size);

        // motion_device.actuate(motion).await; // TOOD: Handle result
    }

    Ok(())
}
