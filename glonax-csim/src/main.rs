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

async fn daemonize(config: &config::SimConfig) -> anyhow::Result<()> {
    use glonax::net::{
        EngineManagementSystem, EngineMessage, J1939Network, KueblerEncoderService, Routable,
    };
    use rand::Rng;

    let neta = J1939Network::new(config.interface.first().unwrap(), 0x6A).unwrap();
    let netb = J1939Network::new(config.interface.first().unwrap(), 0x6B).unwrap();
    let netc = J1939Network::new(config.interface.first().unwrap(), 0x6C).unwrap();
    let netd = J1939Network::new(config.interface.first().unwrap(), 0x6D).unwrap();
    let net0 = J1939Network::new(config.interface.first().unwrap(), 0x0).unwrap();

    let mut rng = rand::thread_rng();

    let mut encoder_a = KueblerEncoderService::new(0x6A);
    let mut encoder_b = KueblerEncoderService::new(0x6B);
    let mut encoder_c = KueblerEncoderService::new(0x6C);
    let mut encoder_d = KueblerEncoderService::new(0x6D);
    let engine_management_system = EngineManagementSystem::new(0x0);

    loop {
        encoder_a.position = rng.gen_range(0..=314) * 1000;
        encoder_a.speed = rng.gen_range(0..=65_535);
        neta.send_vectored(&encoder_a.encode()).await.unwrap();

        tokio::time::sleep(std::time::Duration::from_millis(5)).await;

        encoder_b.position = rng.gen_range(0..=314) * 1000;
        encoder_b.speed = rng.gen_range(0..=65_535);
        netb.send_vectored(&encoder_b.encode()).await.unwrap();

        tokio::time::sleep(std::time::Duration::from_millis(5)).await;

        encoder_c.position = rng.gen_range(0..=314) * 1000;
        encoder_c.speed = rng.gen_range(0..=65_535);
        netc.send_vectored(&encoder_c.encode()).await.unwrap();

        tokio::time::sleep(std::time::Duration::from_millis(5)).await;

        encoder_d.position = rng.gen_range(0..=314) * 1000;
        encoder_d.speed = rng.gen_range(0..=65_535);
        netd.send_vectored(&encoder_d.encode()).await.unwrap();

        tokio::time::sleep(std::time::Duration::from_millis(5)).await;

        let mut engine_message = EngineMessage::new(0x0);

        engine_message.driver_demand = Some(rng.gen_range(0..=100));
        engine_message.actual_engine = Some(rng.gen_range(0..=100));
        engine_message.rpm = Some(rng.gen_range(0..=2_400));

        net0.send_vectored(&engine_management_system.serialize(&mut engine_message))
            .await
            .unwrap();

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
}
