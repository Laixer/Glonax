// Copyright (C) 2024 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use clap::Parser;

mod components;
mod config;
mod device;
mod j1939;
mod server;

pub(crate) mod consts {
    /// Vehicle Management System J1939 address.
    pub const J1939_ADDRESS_VMS: u8 = 0x27;
    /// Engine J1939 address.
    pub const J1939_ADDRESS_ENGINE0: u8 = 0x0;
    /// Hydraulic Control Unit J1939 address.
    pub const J1939_ADDRESS_HCU0: u8 = 0x4A;
    /// Kuebler Encoder 0 J1939 address.
    pub const J1939_ADDRESS_ENCODER0: u8 = 0x6A;
    /// Kuebler Encoder 1 J1939 address.
    pub const J1939_ADDRESS_ENCODER1: u8 = 0x6B;
    /// Kuebler Encoder 2 J1939 address.
    pub const J1939_ADDRESS_ENCODER2: u8 = 0x6C;
    /// Kuebler Encoder 3 J1939 address.
    pub const J1939_ADDRESS_ENCODER3: u8 = 0x6D;
}

#[derive(Parser)]
#[command(author = "Copyright (C) 2024 Laixer Equipment B.V.")]
#[command(version, propagate_version = true)]
#[command(about = "Glonax proxy daemon", long_about = None)]
struct Args {
    /// Bind address.
    #[arg(short = 'b', long = "bind", default_value = "0.0.0.0:30051")]
    address: String,
    /// CAN network interface.
    #[arg(required_unless_present = "simulation", short = 'i', long)]
    interface: Vec<String>,
    /// Refresh host service interval in milliseconds.
    #[arg(long, default_value_t = 200, value_name = "INTERVAL")]
    host_interval: u64,
    /// Configuration file.
    #[arg(
        short = 'c',
        long = "config",
        alias = "conf",
        default_value = "/etc/glonax.conf",
        value_name = "FILE"
    )]
    config: std::path::PathBuf,
    /// Path to NMEA device.
    #[arg(long, value_name = "DEVICE")]
    nmea_device: Option<std::path::PathBuf>,
    /// Serial baud rate.
    #[arg(long, default_value_t = 9_600, value_name = "RATE")]
    nmea_baud_rate: usize,
    /// Enable simulation mode.
    #[arg(long, default_value_t = false)]
    simulation: bool,
    /// Enable simulation jitter.
    #[arg(long, default_value_t = false)]
    simulation_jitter: bool,
    /// Enable pilot mode only.
    #[arg(long, default_value_t = false)]
    pilot_only: bool,
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
        host_interval: args.host_interval,
        nmea_device: args.nmea_device,
        nmea_baud_rate: args.nmea_baud_rate,
        simulation: args.simulation,
        simulation_jitter: args.simulation_jitter,
        pilot_only: args.pilot_only,
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

    ////////////////////

    use std::time::Duration;

    log::debug!("Starting proxy services");
    log::info!("{}", instance);

    if instance.id().is_nil() {
        log::warn!("Instance ID is not set or invalid");
    }

    if config.simulation {
        log::info!("Running in simulation mode");
    }
    if config.pilot_only {
        log::info!("Running in pilot only mode");
    }

    let mut runtime = glonax::runtime::builder(&config, instance)?
        .with_shutdown()
        .enqueue_startup_motion(glonax::core::Motion::ResetAll)
        .build();

    runtime
        .schedule_interval::<glonax::components::Host>(Duration::from_millis(config.host_interval));

    if config.simulation {
        runtime.schedule_interval::<glonax::components::EncoderSimulator>(Duration::from_millis(5));
        runtime.schedule_interval::<glonax::components::EngineSimulator>(Duration::from_millis(10));

        runtime.schedule_motion_sink(device::sink_net_actuator_sim);
    } else {
        if config.nmea_device.is_some() {
            runtime.schedule_io_service(device::service_gnss);
        }

        runtime.schedule_j1939_service(j1939::rx_network_0, &config.interface[0]);
        runtime.schedule_j1939_service(j1939::tx_network_0, &config.interface[0]);
        runtime.schedule_j1939_service(j1939::rx_network_1, &config.interface[1]);
        runtime.schedule_j1939_service(j1939::tx_network_1, &config.interface[1]);

        runtime.schedule_j1939_motion_service(j1939::atx_network_1, &config.interface[1]);
    }

    runtime.schedule_io_service(server::tcp_listen);
    runtime.schedule_io_service(server::unix_listen);
    runtime.schedule_io_service(server::net_announce);

    let mut components = vec![
        runtime.make_dynamic::<components::WorldBuilder>(0),
        runtime.make_dynamic::<components::SensorFusion>(2),
        runtime.make_dynamic::<components::LocalActor>(3),
    ];

    if !config.pilot_only {
        components.push(runtime.make_dynamic::<components::Kinematic>(5));
        components.push(runtime.make_dynamic::<components::Controller>(10));
    }

    runtime
        .run_interval(
            glonax::service::Pipeline::new(components),
            Duration::from_millis(10),
        )
        .await;

    // runtime.enqueue_motion(glonax::core::Motion::StopAll).await;

    std::thread::sleep(Duration::from_millis(50));

    log::debug!("{} was shutdown gracefully", config.global.bin_name);

    Ok(())
}
