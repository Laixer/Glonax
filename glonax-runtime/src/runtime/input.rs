use crate::{
    device::{Gamepad, InputDevice},
    net::motion::SchematicMotion,
    runtime, InputConfig, RuntimeContext,
};

use super::operand::Operand;

pub(crate) async fn exec_service<K: Operand>(
    config: &InputConfig,
    mut runtime: RuntimeContext<K>,
) -> runtime::Result {
    let mut input_device = Gamepad::new(std::path::Path::new(&config.device));

    let sock = tokio::net::UdpSocket::bind("0.0.0.0:0").await.unwrap();

    info!("Listen for input events");

    while let Ok(input) = input_device.next() {
        if let Ok(motion) = runtime.operand.try_from_input_device(input) {
            let schematic_motion = SchematicMotion::from_motion(motion);

            sock.send_to(schematic_motion.as_ref(), "0.0.0.0:54910")
                .await
                .unwrap();
        }
    }

    Ok(())
}
