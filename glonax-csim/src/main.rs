// Copyright (C) 2023 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use clap::Parser;

mod config;

const DEVICE_NET_LOCAL_ADDR: u8 = 0x9e;

#[derive(Parser)]
#[command(author = "Copyright (C) 2023 Laixer Equipment B.V.")]
#[command(version, propagate_version = true)]
#[command(about = "Glonax Machine Simulator", long_about = None)]
struct Args {
    /// CAN network interfaces.
    interface: Vec<String>,
    /// Randomize the start position.
    #[arg(long)]
    randomize_start: bool,
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

type PosLock = std::sync::Arc<std::sync::atomic::AtomicU32>;

async fn listener(config: config::SimConfig, lock: PosLock, lock2: PosLock, lock3: PosLock) {
    use glonax::net::J1939Network;

    let network = J1939Network::new(config.interface.first().unwrap(), 0x4A).unwrap();
    let mut router = glonax::net::Router::new(network);

    let mut service = glonax::net::ActuatorService::new2(0x4A);

    let mut locked = false;

    loop {
        if let Err(e) = router.listen().await {
            log::error!("{}", e);
        };

        if let Some(message) = router.try_accept2(&mut service) {
            if let Some(motion_message) = message.1 {
                log::trace!(
                    "0x{:X?} » {}",
                    router.frame_source().unwrap(),
                    motion_message
                );

                locked = motion_message.locked;
            }
            if let Some(actuator_message) = message.0 {
                if locked {
                    continue;
                }
                log::trace!(
                    "0x{:X?} » {}",
                    router.frame_source().unwrap(),
                    actuator_message
                );

                if let Some(boom_va) = actuator_message.actuators[0] {
                    let position = lock.load(std::sync::atomic::Ordering::SeqCst);

                    if boom_va.is_negative() {
                        let mut adap = boom_va.abs() as u32 / 5_000;
                        if adap > position {
                            adap = position;
                        }
                        lock.store(position - adap, std::sync::atomic::Ordering::Relaxed);
                    } else {
                        let adap = boom_va.abs() as u32 / 5_000;
                        lock.store(
                            (position + adap).min(1832 - 1),
                            std::sync::atomic::Ordering::Relaxed,
                        );
                    }
                }

                if let Some(frame_va) = actuator_message.actuators[1] {
                    let position = lock3.load(std::sync::atomic::Ordering::SeqCst);

                    if frame_va.is_negative() {
                        let mut adap = frame_va.abs() as u32 / 5_000;
                        if adap > position {
                            adap = position;
                        }
                        lock3.store(position - adap, std::sync::atomic::Ordering::Relaxed);
                    } else {
                        let adap = frame_va.abs() as u32 / 5_000;

                        if position + adap > 6280 {
                            let position = (position + adap) - 6280;
                            lock3.store(position, std::sync::atomic::Ordering::Relaxed)
                        } else {
                            lock3.store(position + adap, std::sync::atomic::Ordering::Relaxed);
                        }
                    }
                }

                if let Some(arm_va) = actuator_message.actuators[4] {
                    let position = lock2.load(std::sync::atomic::Ordering::SeqCst);

                    if arm_va.is_positive() {
                        let mut adap = arm_va.abs() as u32 / 5_000;
                        if adap > position {
                            adap = position;
                        }
                        lock2.store(
                            (position - adap).max(685),
                            std::sync::atomic::Ordering::Relaxed,
                        );
                    } else {
                        let adap = arm_va.abs() as u32 / 5_000;
                        lock2.store(
                            (position + adap).min(2751),
                            std::sync::atomic::Ordering::Relaxed,
                        );
                    }
                }
            }
        }
    }
}

async fn signal_writer(config: config::SimConfig, lock: PosLock, lock2: PosLock, lock3: PosLock) {
    use glonax::net::{
        EngineManagementSystem, EngineMessage, J1939Network, KueblerEncoderService, Routable,
    };
    use rand::Rng;

    let neta = J1939Network::new(config.interface.first().unwrap(), 0x6A).unwrap();
    let netb = J1939Network::new(config.interface.first().unwrap(), 0x6B).unwrap();
    let netc = J1939Network::new(config.interface.first().unwrap(), 0x6C).unwrap();
    let netd = J1939Network::new(config.interface.first().unwrap(), 0x6D).unwrap();
    let net0 = J1939Network::new(config.interface.first().unwrap(), 0x0).unwrap();

    let mut rng = rand::rngs::OsRng::default();

    let mut encoder_a = KueblerEncoderService::new(0x6A);
    let mut encoder_b = KueblerEncoderService::new(0x6B);
    let mut encoder_c = KueblerEncoderService::new(0x6C);
    let mut encoder_d = KueblerEncoderService::new(0x6D);
    let engine_management_system = EngineManagementSystem::new(0x0);

    let position_d = rng.gen_range(0..=3140);

    loop {
        encoder_a.position = lock3.load(std::sync::atomic::Ordering::SeqCst);
        encoder_a.speed = 0;
        neta.send_vectored(&encoder_a.encode()).await.unwrap();

        tokio::time::sleep(std::time::Duration::from_millis(5)).await;

        encoder_b.position = lock.load(std::sync::atomic::Ordering::SeqCst);
        encoder_b.speed = 0;
        netb.send_vectored(&encoder_b.encode()).await.unwrap();

        tokio::time::sleep(std::time::Duration::from_millis(5)).await;

        encoder_c.position = lock2.load(std::sync::atomic::Ordering::SeqCst);
        encoder_c.speed = 0;
        netc.send_vectored(&encoder_c.encode()).await.unwrap();

        tokio::time::sleep(std::time::Duration::from_millis(5)).await;

        encoder_d.position = position_d;
        encoder_d.speed = 0;
        netd.send_vectored(&encoder_d.encode()).await.unwrap();

        tokio::time::sleep(std::time::Duration::from_millis(5)).await;

        let mut engine_message = EngineMessage::new(0x0);

        engine_message.driver_demand = Some(rng.gen_range(18..=20));
        engine_message.actual_engine = Some(rng.gen_range(19..=21));
        engine_message.rpm = Some(rng.gen_range(1180..=1200));

        net0.send_vectored(&engine_management_system.serialize(&mut engine_message))
            .await
            .unwrap();

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
}

async fn daemonize(config: &config::SimConfig) -> anyhow::Result<()> {
    use rand::Rng;

    let runtime = glonax::RuntimeBuilder::from_config(config)?
        .with_shutdown()
        .build();

    let mut rng = rand::rngs::OsRng::default();

    let yi = rng.gen_range(0..=1832 - 1);
    let yi2 = rng.gen_range(685 + 1..=2751 - 1);
    let yi2x = rng.gen_range(0..=6280);

    let l = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(yi));
    let l2 = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(yi2));
    let l3 = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(yi2x));

    runtime.spawn_background_task(listener(config.clone(), l.clone(), l2.clone(), l3.clone()));
    runtime.spawn_background_task(signal_writer(config.clone(), l, l2, l3));

    runtime.wait_for_shutdown().await;

    Ok(())
}
