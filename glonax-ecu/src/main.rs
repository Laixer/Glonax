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

    /// MQTT broker address.
    #[arg(short = 'c', long = "connect", default_value = "127.0.0.1")]
    address: String,

    /// MQTT broker port.
    #[arg(short, long, default_value_t = 1883)]
    port: u16,

    /// MQTT broker username.
    #[arg(short = 'U', long)]
    username: Option<String>,

    /// MQTT broker password.
    #[arg(short = 'P', long)]
    password: Option<String>,

    /// Disable machine motion (frozen mode).
    #[arg(long)]
    disable_motion: bool,

    /// Run motion requests slow.
    #[arg(long)]
    slow_motion: bool,

    /// Daemonize the service.
    #[arg(long)]
    daemon: bool,

    /// Number of runtime workers.
    #[arg(long)]
    workers: Option<usize>,

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
    config.global.mqtt_host = args.address;
    config.global.mqtt_port = args.port;
    config.global.mqtt_username = args.username;
    config.global.mqtt_password = args.password;
    config.global.enable_motion = !args.disable_motion;
    config.global.slow_motion = args.slow_motion;
    config.global.daemon = args.daemon;

    if let Some(workers) = args.workers {
        config.global.runtime_workers = workers;
    }

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
    use glonax::device::{Hcu, Mecu, Vecu};
    use glonax::kernel::excavator::Excavator;
    use glonax::net::J1939Network;
    use glonax::net::Router;

    let mut runtime = glonax::RuntimeBuilder::<Excavator>::from_config(config)?
        .enable_term_shutdown()
        .build();

    let signal_manager = runtime.new_signal_manager();
    let motion_manager = runtime.new_motion_manager();

    let net = std::sync::Arc::new(J1939Network::new(&config.interface, DEVICE_NET_LOCAL_ADDR)?);

    let mut vecu = Vecu::new(signal_manager.publisher());
    let mut mecu = Mecu::new(net.clone(), signal_manager.publisher());
    let mut hcu = Hcu::new(net.clone());

    let motion_device = Hcu::new(net.clone());

    runtime
        .eventhub
        .subscribe(motion_manager.adapter(motion_device));

    tokio::task::spawn(async move {
        loop {
            runtime.eventhub.next().await
        }
    });

    let mut router = Router::new(net);

    tokio::task::spawn(async move {
        loop {
            if let Err(e) = router.listen().await {
                log::error!("{}", e);
            }

            router.try_accept(&mut vecu);
            router.try_accept(&mut mecu);
            router.try_accept(&mut hcu);
        }
    });

    runtime.shutdown.1.recv().await.unwrap();

    Ok(())
}
