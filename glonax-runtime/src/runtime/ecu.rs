use crate::{
    device::{Hcu, Mecu, Vecu},
    net::motion::SchematicMotion,
    runtime, EcuConfig, RuntimeContext,
};

use runtime::operand::Operand;

pub(crate) async fn exec_service<K: Operand>(
    config: &EcuConfig,
    mut runtime: RuntimeContext<K>,
) -> runtime::Result {
    use crate::device::CoreDevice;

    runtime.core_device.new_gateway_device::<Vecu>();

    let signal_device = Mecu::new(runtime.signal_manager.pusher());
    runtime.core_device.subscribe(signal_device);

    let mut motion_device = runtime.core_device.new_gateway_device::<Hcu>();

    let mut motion_chain = runtime::MotionChain::new(&mut motion_device, &runtime.tracer)
        .enable(config.global.enable_motion);

    tokio::task::spawn(async move { while runtime.core_device.next().await.is_ok() {} });

    let address = if config.address.is_empty() {
        "0.0.0.0:54910".to_owned()
    } else {
        config.address.clone()
    };

    let sock = tokio::net::UdpSocket::bind(&address).await.unwrap();
    let mut buf = [0; 1024];

    info!("Listen for network events on {}", address);

    while let Ok((_size, _addr)) = sock.recv_from(&mut buf).await {
        let schematic_motion = SchematicMotion::try_from(&buf[..]).unwrap();

        motion_chain.request(schematic_motion).await; // TOOD: Handle result
    }

    Ok(())
}
