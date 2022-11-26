use crate::{
    device::{Hcu, Mecu, Vecu},
    net::motion::SchematicMotion,
    runtime::{self, MotionChain},
    EcuConfig, RuntimeContext,
};

use runtime::operand::Operand;

pub(crate) async fn exec_service<K: Operand>(
    config: &EcuConfig,
    mut runtime: RuntimeContext<K>,
) -> runtime::Result {
    use crate::device::CoreDevice;

    runtime.new_gateway_device::<Vecu>();

    let signal_device = Mecu::new(runtime.signal_manager.pusher());
    runtime.subscribe_gateway_device(signal_device);

    let mut motion_device = runtime.new_gateway_device::<Hcu>();

    let mut motion_chain =
        MotionChain::new(&mut motion_device, &runtime.tracer).enable(config.global.enable_motion);

    tokio::task::spawn(async move {
        while runtime.core_device.as_mut().unwrap().next().await.is_ok() {}
    });

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
