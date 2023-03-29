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
#[command(about = "Glonax ECU daemon", long_about = None)]
struct Args {
    /// CAN network interfaces.
    interface: Vec<String>,
    /// Disable machine motion (frozen mode).
    #[arg(long)]
    disable_motion: bool,
    /// Run motion requests slow.
    #[arg(long)]
    slow_motion: bool,
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

    let bin_name = format!(
        "{}@{}",
        env!("CARGO_BIN_NAME").to_string(),
        args.interface.first().unwrap().to_string()
    );

    let mut config = config::EcuConfig {
        interface: args.interface,
        global: glonax::GlobalConfig::default(),
    };

    config.global.bin_name = bin_name;
    config.global.enable_motion = !args.disable_motion;
    config.global.slow_motion = args.slow_motion;
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

async fn daemonize(config: &config::EcuConfig) -> anyhow::Result<()> {
    use glonax::net::J1939Network;
    use glonax::net::{EngineService, KueblerEncoderService};
    use glonax::queue::SignalSource;

    let queue = glonax::queue::SignalQueueWriter::new().unwrap();

    // TODO: Assign new network ID to each J1939 network.
    let mut router = glonax::net::Router::from_iter(
        config
            .interface
            .iter()
            .map(|iface| J1939Network::new(iface, DEVICE_NET_LOCAL_ADDR).unwrap()),
    );

    let mut engine_service_list = vec![EngineService::new(0x0)];
    let mut encoder_list = vec![
        KueblerEncoderService::new(0x6A),
        KueblerEncoderService::new(0x6B),
        KueblerEncoderService::new(0x6C),
        KueblerEncoderService::new(0x6D),
    ];

    loop {
        if let Err(e) = router.listen().await {
            log::error!("{}", e);
        }

        for service in &mut engine_service_list {
            if router.try_accept(service) {
                log::debug!("{} » {}", router.frame_source().unwrap(), service);

                // service.fetch(&queue);
            }
        }

        for encoder in &mut encoder_list {
            if router.try_accept(encoder) {
                log::debug!("{} » {}", router.frame_source().unwrap(), encoder);

                // encoder.fetch(&queue);
            }
        }
    }
}
