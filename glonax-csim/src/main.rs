// Copyright (C) 2023 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use clap::Parser;

mod config;

#[derive(Parser)]
#[command(author = "Copyright (C) 2023 Laixer Equipment B.V.")]
#[command(version, propagate_version = true)]
#[command(about = "Glonax Machine Simulator", long_about = None)]
struct Args {
    /// CAN network interfaces.
    interface: Vec<String>,
    /// Randomize the start position.
    #[arg(long, default_value_t = true)]
    randomize_start: bool,
    /// Input commands will translate to the full motion range.
    #[arg(long, default_value_t = true)]
    jitter: bool,
    /// Daemonize the service.
    #[arg(long)]
    daemon: bool,
    /// Level of verbosity.
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let bin_name = env!("CARGO_BIN_NAME");

    let mut config = config::SimConfig {
        interface: args.interface,
        randomize_start: args.randomize_start,
        jitter: args.jitter,
        global: glonax::GlobalConfig::default(),
    };

    config.global.bin_name = bin_name.to_string();
    config.global.daemon = args.daemon;

    let mut log_config = simplelog::ConfigBuilder::new();
    if args.daemon {
        log_config.set_time_level(log::LevelFilter::Off);
        log_config.set_thread_level(log::LevelFilter::Off);
    } else {
        log_config.set_time_offset_to_local().ok();
        log_config.set_time_format_rfc2822();
    }

    log_config.set_target_level(log::LevelFilter::Off);
    log_config.set_location_level(log::LevelFilter::Off);
    log_config.add_filter_ignore_str("sled");
    log_config.add_filter_ignore_str("mio");

    let log_level = if args.daemon {
        log::LevelFilter::Info
    } else {
        match args.verbose {
            0 => log::LevelFilter::Error,
            1 => log::LevelFilter::Info,
            2 => log::LevelFilter::Debug,
            _ => log::LevelFilter::Trace,
        }
    };

    let color_choice = if args.daemon {
        simplelog::ColorChoice::Never
    } else {
        simplelog::ColorChoice::Auto
    };

    simplelog::TermLogger::init(
        log_level,
        log_config.build(),
        simplelog::TerminalMode::Mixed,
        color_choice,
    )?;

    if args.daemon {
        log::debug!("Running service as daemon");
    }

    log::trace!("{:#?}", config);

    daemonize(&config).await
}

struct EcuState {
    power: [std::sync::atomic::AtomicI16; 4],
    motion_lock: std::sync::atomic::AtomicBool,
}

impl EcuState {
    fn new() -> Self {
        Self {
            power: [0.into(), 0.into(), 0.into(), 0.into()],
            motion_lock: std::sync::atomic::AtomicBool::new(false),
        }
    }

    fn lock(&self) {
        self.power[0].store(0, std::sync::atomic::Ordering::Relaxed);
        self.power[1].store(0, std::sync::atomic::Ordering::Relaxed);
        self.power[2].store(0, std::sync::atomic::Ordering::Relaxed);
        self.power[3].store(0, std::sync::atomic::Ordering::Relaxed);
        self.motion_lock
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    fn unlock(&self) {
        self.motion_lock
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }

    fn is_locked(&self) -> bool {
        self.motion_lock.load(std::sync::atomic::Ordering::Relaxed)
    }
}

async fn net_listener(config: config::SimConfig, state: std::sync::Arc<EcuState>) {
    use glonax::net::J1939Network;

    let network = J1939Network::new(config.interface.first().unwrap(), 0x4A).unwrap();
    let mut router = glonax::net::Router::new(network);

    let mut service = glonax::net::ActuatorService::new(0x4A);

    loop {
        if let Err(e) = router.listen().await {
            log::error!("{}", e);
        };

        if let Some(message) = router.try_accept(&mut service) {
            if let Some(motion_message) = message.1 {
                if motion_message.locked {
                    state.lock();
                } else {
                    state.unlock();
                }
            }
            if let Some(actuator_message) = message.0 {
                if state.is_locked() {
                    continue;
                }

                // FRAME
                if let Some(value) = actuator_message.actuators[1] {
                    state.power[0].store(value, std::sync::atomic::Ordering::Relaxed);
                }

                // BOOM
                if let Some(value) = actuator_message.actuators[0] {
                    state.power[1].store(value, std::sync::atomic::Ordering::Relaxed);
                }

                // ARM
                if let Some(value) = actuator_message.actuators[4] {
                    state.power[2].store(value, std::sync::atomic::Ordering::Relaxed);
                }

                // ATTACHMENT
                if let Some(value) = actuator_message.actuators[5] {
                    state.power[3].store(value, std::sync::atomic::Ordering::Relaxed);
                }
            }
        }
    }
}

async fn ecu_simulator(config: config::SimConfig, state: std::sync::Arc<EcuState>) {
    use glonax::net::{EncoderService, EngineManagementSystem, EngineMessage, J1939Network};
    use rand::Rng;

    let mut rng = rand::rngs::OsRng::default();

    let neta = J1939Network::new(config.interface.first().unwrap(), 0x6A).unwrap();
    let netb = J1939Network::new(config.interface.first().unwrap(), 0x6B).unwrap();
    let netc = J1939Network::new(config.interface.first().unwrap(), 0x6C).unwrap();
    let netd = J1939Network::new(config.interface.first().unwrap(), 0x6D).unwrap();
    let net0 = J1939Network::new(config.interface.first().unwrap(), 0x0).unwrap();

    let encoder_a = EncoderService::new(0x6A);
    let encoder_b = EncoderService::new(0x6B);
    let encoder_c = EncoderService::new(0x6C);
    let encoder_d = EncoderService::new(0x6D);
    let engine_management_system = EngineManagementSystem::new(0x0);

    let mut encoder_a_position = rng.gen_range(0..=6280);
    let mut encoder_b_position = rng.gen_range(0..=1832 - 1);
    let mut encoder_c_position = rng.gen_range(685..=2760);
    let mut encoder_d_position = rng.gen_range(0..=3100);

    loop {
        {
            let value = state.power[0].load(std::sync::atomic::Ordering::SeqCst);

            let fac = value / 2_500;
            let mut position_0 = (encoder_a_position as i16 + fac) % 6280;
            if position_0 < 0 {
                position_0 = 6280 + position_0;
            }

            encoder_a_position = position_0 as u32;
            let mut def_encoder_a_position = encoder_a_position;
            if config.jitter && encoder_a_position < 6280 && encoder_a_position > 0 {
                def_encoder_a_position += rng.gen_range(0..=1);
            }

            neta.send_vectored(&encoder_a.encode(def_encoder_a_position, 0))
                .await
                .unwrap();

            tokio::time::sleep(std::time::Duration::from_millis(rng.gen_range(3..=8))).await;
        }

        {
            let value = state.power[1].load(std::sync::atomic::Ordering::SeqCst);

            let fac = value / 5_000;
            let position_0 = (encoder_b_position as i16 + fac).clamp(0, 1832 - 1);

            encoder_b_position = position_0 as u32;
            let mut def_encoder_b_position = encoder_b_position;
            if config.jitter && def_encoder_b_position < 1832 - 1 && encoder_a_position > 0 {
                def_encoder_b_position += rng.gen_range(0..=1);
            }

            netb.send_vectored(&encoder_b.encode(def_encoder_b_position, 0))
                .await
                .unwrap();

            tokio::time::sleep(std::time::Duration::from_millis(rng.gen_range(3..=8))).await;
        }

        {
            let value = state.power[2].load(std::sync::atomic::Ordering::SeqCst);

            let fac = value / 5_000;
            let position_0 = (encoder_c_position as i16 - fac).clamp(685, 2760);

            encoder_c_position = position_0 as u32;
            let mut def_encoder_c_position = encoder_c_position;
            if config.jitter && encoder_c_position < 2760 && encoder_c_position > 685 {
                def_encoder_c_position += rng.gen_range(0..=1);
            }

            netc.send_vectored(&encoder_c.encode(def_encoder_c_position, 0))
                .await
                .unwrap();

            tokio::time::sleep(std::time::Duration::from_millis(rng.gen_range(3..=8))).await;
        }

        {
            let value = state.power[3].load(std::sync::atomic::Ordering::SeqCst);

            let fac = value / 5_000;
            let position_0 = (encoder_d_position as i16 + fac).clamp(0, 3100);

            encoder_d_position = position_0 as u32;
            let mut def_encoder_d_position = encoder_d_position;
            if config.jitter && encoder_d_position < 3100 && encoder_d_position > 0 {
                def_encoder_d_position += rng.gen_range(0..=1);
            }

            netd.send_vectored(&encoder_d.encode(def_encoder_d_position, 0))
                .await
                .unwrap();

            tokio::time::sleep(std::time::Duration::from_millis(rng.gen_range(3..=8))).await;
        }

        let mut engine_message = EngineMessage::new(0x0);

        engine_message.driver_demand = Some(rng.gen_range(18..=20));
        engine_message.actual_engine = Some(rng.gen_range(19..=21));
        engine_message.rpm = Some(rng.gen_range(1180..=1200));

        net0.send_vectored(&engine_management_system.serialize(&mut engine_message))
            .await
            .unwrap();

        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
}

async fn daemonize(config: &config::SimConfig) -> anyhow::Result<()> {
    let runtime = glonax::RuntimeBuilder::from_config(config)?
        .with_shutdown()
        .build();

    let state = std::sync::Arc::new(EcuState::new());

    runtime.spawn_background_task(net_listener(config.clone(), state.clone()));
    runtime.spawn_background_task(ecu_simulator(config.clone(), state));

    runtime.wait_for_shutdown().await;

    Ok(())
}
