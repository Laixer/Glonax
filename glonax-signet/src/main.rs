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
    /// CAN network interface.
    interface: String,
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

    let bin_name = format!("{}@{}", env!("CARGO_BIN_NAME").to_string(), args.interface);

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
    use glonax::device::Mecu;
    use glonax::net::J1939Network;

    let queue = glonax::signal::SignalQueueWriter::new().unwrap();

    // let net = J1939Network::new(&config.interface, DEVICE_NET_LOCAL_ADDR)?;

    // let mut mecu = Mecu::new(queue);

    // let mut router = glonax::net::Router::new(net);

    loop {
        queue.send(glonax::transport::Signal::new(
            0x1,
            glonax::transport::signal::Metric::Angle(24500.0),
        ));

        tokio::time::sleep(std::time::Duration::from_millis(5)).await
    }

    // loop {
    //     if let Err(e) = router.listen().await {
    //         log::error!("{}", e);
    //     }

    //     router.try_accept(&mut mecu);
    // }
}
