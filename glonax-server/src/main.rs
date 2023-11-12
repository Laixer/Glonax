// Copyright (C) 2023 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use clap::Parser;

mod config;
mod device;
mod probe;
mod server;

#[derive(Parser)]
#[command(author = "Copyright (C) 2023 Laixer Equipment B.V.")]
#[command(version, propagate_version = true)]
#[command(about = "Glonax proxy daemon", long_about = None)]
struct Args {
    /// Bind address.
    #[arg(short = 'b', long = "bind", default_value = "0.0.0.0:30051")]
    address: String,
    /// CAN network interface.
    interface: String,
    /// CAN network interface.
    interface2: Option<String>,
    /// Refresh host service interval in milliseconds.
    #[arg(long, default_value_t = 500, value_name = "INTERVAL")]
    host_interval: u64,
    /// Configuration file.
    #[arg(
        short = 'c',
        long = "config",
        default_value = "/etc/glonax.conf",
        value_name = "FILE"
    )]
    config: std::path::PathBuf,
    /// Path to GNSS device.
    #[arg(long, value_name = "DEVICE")]
    gnss_device: Option<std::path::PathBuf>,
    /// Serial baud rate.
    #[arg(long, default_value_t = 9_600, value_name = "RATE")]
    gnss_baud_rate: usize,
    /// Probe interval in seconds.
    #[arg(long, default_value_t = 60, value_name = "INTERVAL")]
    probe_interval: u64,
    /// Disable probing.
    #[arg(long)]
    no_probe: bool,
    /// Enable simulation mode.
    #[arg(long, default_value_t = false)]
    simulation: bool,
    /// Quiet output (no logging).
    #[arg(long)]
    quiet: bool,
    /// Daemonize the service.
    #[arg(short = 'D', long)]
    daemon: bool,
    /// Level of verbosity.
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use log::LevelFilter;

    let args = Args::parse();

    let bin_name = env!("CARGO_BIN_NAME");

    let mut config = config::ProxyConfig {
        address: args.address,
        interface: args.interface,
        interface2: args.interface2,
        host_interval: args.host_interval,
        gnss_device: args.gnss_device,
        gnss_baud_rate: args.gnss_baud_rate,
        probe_interval: args.probe_interval,
        probe: !args.no_probe,
        simulation: args.simulation,
        simulation_jitter: false,
        instance: glonax::from_file(args.config)?,
        global: glonax::GlobalConfig::default(),
    };

    config.global.bin_name = bin_name.to_string();
    config.global.daemon = args.daemon;

    let mut log_config = simplelog::ConfigBuilder::new();
    if args.daemon {
        log_config.set_time_level(LevelFilter::Off);
        log_config.set_thread_level(LevelFilter::Off);
    }

    log_config.set_target_level(LevelFilter::Off);
    log_config.set_location_level(LevelFilter::Off);
    log_config.add_filter_ignore_str("sled");
    log_config.add_filter_ignore_str("mio");

    let log_level = if args.daemon {
        LevelFilter::Info
    } else if args.quiet {
        LevelFilter::Off
    } else {
        match args.verbose {
            0 => LevelFilter::Info,
            1 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
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

    use std::sync::Arc;
    use tokio::sync::RwLock;

    log::debug!("Starting proxy services");

    if config.simulation {
        log::warn!("Simulation mode is enabled");
    }

    log::info!("Instance ID: {}", config.instance.id);
    log::info!("Instance Model: {}", config.instance.model);
    log::info!("Instance Name: {}", config.instance.name);

    if config.instance.id.starts_with("00000000") {
        log::warn!("Instance ID is not set or invalid");
    }

    // TODO: Enable service termination
    let mut runtime = glonax::RuntimeBuilder::from_config(&config)?
        // .with_shutdown()
        .enqueue_motion(glonax::core::Motion::ResetAll)
        .build();

    let machine_state = Arc::new(RwLock::new(glonax::RuntimeState::default()));

    runtime.spawn_service(&machine_state, device::service_host);
    runtime.spawn_service(&machine_state, device::service_gnss);

    if config.simulation {
        runtime.spawn_service(&machine_state, device::service_net_encoder_sim);
        runtime.spawn_service(&machine_state, device::service_net_ems_sim);

        runtime.spawn_motion_sink(&machine_state, device::sink_net_actuator_sim);
    } else {
        runtime.spawn_service(&machine_state, device::service_net_encoder);
        runtime.spawn_service(&machine_state, device::service_net_ems);

        runtime.spawn_motion_sink(&machine_state, device::sink_net_actuator);
    }

    runtime.spawn_middleware_service(&machine_state, probe::service);

    runtime
        .run_motion_service(&machine_state, server::service)
        .await;

    // runtime.wait_for_shutdown().await;

    log::debug!("{} was shutdown gracefully", config.global.bin_name);

    Ok(())
}
