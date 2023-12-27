// Copyright (C) 2023 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use clap::Parser;

mod config;
mod device;
mod encoder;
mod kinematic;
mod pipeline;
mod server;
mod state;

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

    let mut config = config::ProxyConfig {
        address: args.address,
        interface: args.interface,
        interface2: args.interface2,
        host_interval: args.host_interval,
        gnss_device: args.gnss_device,
        gnss_baud_rate: args.gnss_baud_rate,
        probe_interval: 60,
        probe: false,
        simulation: args.simulation,
        simulation_jitter: false,
        ..Default::default()
    };

    let instance: glonax::core::Instance = glonax::from_file(args.config)?;

    config.global.bin_name = env!("CARGO_BIN_NAME").to_string();
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

    log::debug!("Starting proxy services");
    log::info!("Instance ID: {}", instance.id);
    log::info!("Instance Model: {}", instance.model);
    log::info!("Instance Name: {}", instance.name);

    if instance.id.starts_with("00000000") {
        log::warn!("Instance ID is not set or invalid");
    }

    use std::time::Duration;

    // TODO: Enable service termination
    let mut runtime = glonax::runtime::builder(&config, instance)?
        // .with_shutdown()
        .enqueue_startup_motion(glonax::core::Motion::ResetAll)
        .build();

    if config.gnss_device.is_some() {
        runtime.schedule_io_service(device::service_gnss);
    }

    if config.simulation {
        log::warn!("Running in simulation mode");

        runtime.schedule_interval_component::<encoder::EncoderSimService>(Duration::from_millis(5));
        runtime.schedule_interval_component::<glonax::net::EngineManagementSystemSimulator>(
            Duration::from_millis(10),
        );

        runtime.spawn_motion_sink(device::sink_net_actuator_sim);
    } else {
        runtime.schedule_io_service(device::service_net_encoder);

        if config.interface2.is_some() {
            runtime.schedule_io_service(device::service_net_ems);
        }

        runtime.spawn_motion_sink(device::sink_net_actuator);
    }

    // TODO: Replace with the component pipeline
    // TODO: The network server is an io service
    // runtime.run_motion_service(server::service).await;

    let mut pipe = pipeline::PipelineComponent::default();

    pipe.add::<glonax::components::Host>();
    pipe.add::<kinematic::KinematicComponent>();

    runtime
        .run_interval_component(pipe, Duration::from_millis(15))
        .await;

    // runtime.wait_for_shutdown().await;

    log::debug!("{} was shutdown gracefully", config.global.bin_name);

    Ok(())
}
