use std::time::Duration;

use glonax::{core::Motion, SharedRuntimeState};
use tokio::time::sleep;

use crate::config::ProxyConfig;

pub type MotionReceiver = tokio::sync::mpsc::Receiver<Motion>;

pub(super) async fn service_host(config: ProxyConfig, runtime_state: SharedRuntimeState) {
    log::debug!("Starting host service");

    let mut service = glonax::net::HostService::default();

    loop {
        service.refresh();
        service.fill(runtime_state.clone()).await;

        sleep(Duration::from_millis(config.host_interval)).await;
    }
}

// TODO: Move to glonax-runtime
struct Encoder {
    rng: rand::rngs::OsRng,
    position: u32,
    factor: i16,
    bounds: (i16, i16),
    multiturn: bool,
    invert: bool,
}

impl Encoder {
    fn new(factor: i16, bounds: (i16, i16), multiturn: bool, invert: bool) -> Self {
        Self {
            rng: rand::rngs::OsRng,
            position: bounds.0 as u32,
            factor,
            bounds,
            multiturn,
            invert,
        }
    }

    fn position(&mut self, velocity: i16, jitter: bool) -> u32 {
        use rand::Rng;

        let velocity_norm = velocity / self.factor;

        let velocity_norm = if self.invert {
            -velocity_norm
        } else {
            velocity_norm
        };

        if self.multiturn {
            let mut position = (self.position as i16 + velocity_norm) % self.bounds.1;
            if position < 0 {
                position += self.bounds.1;
            }
            self.position = position as u32;
        } else {
            let mut position =
                (self.position as i16 + velocity_norm).clamp(self.bounds.0, self.bounds.1);
            if position < 0 {
                position += self.bounds.1;
            }
            self.position = position as u32;
        }

        if jitter && self.position < self.bounds.1 as u32 && self.position > 0 {
            self.position + self.rng.gen_range(0..=1)
        } else {
            self.position
        }
    }
}

pub(super) async fn service_net_encoder_sim(
    config: ProxyConfig,
    runtime_state: SharedRuntimeState,
) {
    use glonax::net::EncoderMessage;

    use std::sync::atomic::Ordering;

    log::debug!("Starting encoder service");

    let encoder_frame = Encoder::new(2_500, (0, 6_280), true, false);
    let encoder_boom = Encoder::new(5_000, (0, 1_832), false, false);
    let encoder_arm = Encoder::new(5_000, (685, 2_760), false, true);
    let encoder_attachment = Encoder::new(5_000, (0, 3_100), false, false);

    let mut control_devices = [
        (0x6A, glonax::core::Actuator::Slew, encoder_frame),
        (0x6B, glonax::core::Actuator::Boom, encoder_boom),
        (0x6C, glonax::core::Actuator::Arm, encoder_arm),
        (0x6D, glonax::core::Actuator::Attachment, encoder_attachment),
    ];

    loop {
        for device in control_devices.iter_mut() {
            sleep(Duration::from_millis(5)).await;

            let pos = runtime_state.read().await.ecu_state.power[device.1 as usize]
                .load(Ordering::SeqCst);

            EncoderMessage::new_with_position(
                device.0,
                device.2.position(pos, config.simulation_jitter),
            )
            .fill(runtime_state.clone())
            .await;
        }
    }
}

pub(super) async fn service_net_encoder(config: ProxyConfig, runtime_state: SharedRuntimeState) {
    use glonax::net::{EncoderService, J1939Network, Router};

    log::debug!("Starting encoder service");

    match J1939Network::new(&config.interface, glonax::consts::DEFAULT_J1939_ADDRESS) {
        Ok(network) => {
            let mut router = Router::new(network);

            let mut encoder_list = vec![
                EncoderService::new(0x6A),
                EncoderService::new(0x6B),
                EncoderService::new(0x6C),
                EncoderService::new(0x6D),
            ];

            loop {
                if let Err(e) = router.listen().await {
                    log::error!("Failed to receive from router: {}", e);
                }

                for encoder in &mut encoder_list {
                    if let Some(message) = router.try_accept(encoder) {
                        message.fill(runtime_state.clone()).await;
                    }
                }
            }
        }
        Err(e) => log::error!("Failed to create network: {}", e),
    }
}

pub(super) async fn service_net_ems_sim(_config: ProxyConfig, runtime_state: SharedRuntimeState) {
    use glonax::net::EngineMessage;

    log::debug!("Starting EMS service");

    use rand::Rng;
    let mut rng = rand::rngs::OsRng;

    loop {
        sleep(Duration::from_millis(10)).await;

        EngineMessage {
            driver_demand: Some(rng.gen_range(18..=20)),
            actual_engine: Some(rng.gen_range(19..=21)),
            rpm: Some(rng.gen_range(1180..=1200)),
            ..Default::default()
        }
        .fill(runtime_state.clone())
        .await;
    }
}

pub(super) async fn service_net_ems(config: ProxyConfig, runtime_state: SharedRuntimeState) {
    use glonax::net::{EngineManagementSystem, J1939Network, Router};

    if config.interface2.is_none() {
        return;
    }

    log::debug!("Starting EMS service");

    match J1939Network::new(
        &config.interface2.unwrap(),
        glonax::consts::DEFAULT_J1939_ADDRESS,
    ) {
        Ok(network) => {
            let mut router = Router::new(network);

            let mut engine_management_service = EngineManagementSystem;

            loop {
                if let Err(e) = router.listen().await {
                    log::error!("Failed to receive from router: {}", e);
                }

                if let Some(message) = router.try_accept(&mut engine_management_service) {
                    message.fill(runtime_state.clone()).await;
                }
            }
        }
        Err(e) => log::error!("Failed to create network: {}", e),
    }
}

pub(super) async fn service_gnss(config: ProxyConfig, runtime_state: SharedRuntimeState) {
    use tokio::io::{AsyncBufReadExt, BufReader};

    if config.gnss_device.is_none() {
        return;
    }

    log::debug!("Starting GNSS service");

    match glonax_serial::Uart::open(
        std::path::Path::new(config.gnss_device.as_ref().unwrap()),
        glonax_serial::BaudRate::from_speed(config.gnss_baud_rate),
    ) {
        Ok(serial) => {
            let reader = BufReader::new(serial);
            let mut lines = reader.lines();

            let service = glonax::net::NMEAService;

            while let Ok(Some(line)) = lines.next_line().await {
                if let Some(message) = service.decode(line) {
                    message.fill(runtime_state.clone()).await;
                }
            }
        }
        Err(e) => {
            log::error!("Failed to open serial: {}", e);
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    }
}

pub(super) async fn sink_net_actuator_sim(
    _config: ProxyConfig,
    runtime_state: SharedRuntimeState,
    mut motion_rx: MotionReceiver,
) {
    log::debug!("Starting motion listener");

    while let Some(motion) = motion_rx.recv().await {
        match motion {
            Motion::StopAll => {
                runtime_state.write().await.ecu_state.lock();
            }
            Motion::ResumeAll => {
                runtime_state.write().await.ecu_state.unlock();
            }
            Motion::ResetAll => {
                runtime_state.write().await.ecu_state.lock();
                runtime_state.write().await.ecu_state.unlock();
            }
            Motion::StraightDrive(_value) => {
                // TODO: Implement
            }
            Motion::Change(changes) => {
                if runtime_state.read().await.ecu_state.is_locked() {
                    continue;
                }

                for changeset in &changes {
                    runtime_state.write().await.ecu_state.power[changeset.actuator as usize]
                        .store(changeset.value, std::sync::atomic::Ordering::Relaxed);
                }
            }
        }
    }
}

pub(super) async fn sink_net_actuator(
    config: ProxyConfig,
    _runtime_state: SharedRuntimeState,
    mut motion_rx: MotionReceiver,
) {
    use glonax::net::{ActuatorService, J1939Network};

    log::debug!("Starting motion listener");

    match J1939Network::new(&config.interface, glonax::consts::DEFAULT_J1939_ADDRESS) {
        Ok(network) => {
            let service = ActuatorService::new(0x4A);

            while let Some(motion) = motion_rx.recv().await {
                match motion {
                    Motion::StopAll => {
                        if let Err(e) = network.send_vectored(&service.lock()).await {
                            log::error!("Failed to send motion: {}", e);
                        }
                    }
                    Motion::ResumeAll => {
                        if let Err(e) = network.send_vectored(&service.unlock()).await {
                            log::error!("Failed to send motion: {}", e);
                        }
                    }
                    Motion::ResetAll => {
                        if let Err(e) = network.send_vectored(&service.lock()).await {
                            log::error!("Failed to send motion: {}", e);
                        }
                        if let Err(e) = network.send_vectored(&service.unlock()).await {
                            log::error!("Failed to send motion: {}", e);
                        }
                    }
                    Motion::StraightDrive(value) => {
                        let frames = &service.drive_straight(value);

                        if let Err(e) = network.send_vectored(frames).await {
                            log::error!("Failed to send motion: {}", e);
                        }
                    }
                    Motion::Change(changes) => {
                        let frames = &service.actuator_command(
                            changes
                                .iter()
                                .map(|changeset| (changeset.actuator as u8, changeset.value))
                                .collect(),
                        );

                        if let Err(e) = network.send_vectored(frames).await {
                            log::error!("Failed to send motion: {}", e);
                        }
                    }
                }
            }

            log::debug!("Motion listener shutdown");
        }
        Err(e) => log::error!("Failed to create network: {}", e),
    }
}
