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
    /// Kuebler Inclinometer 0 J1939 address.
    pub const J1939_ADDRESS_IMU0: u8 = 0x7A;
}

#[derive(Parser)]
#[command(author = "Copyright (C) 2024 Laixer Equipment B.V.")]
#[command(version, propagate_version = true)]
#[command(about = "Glonax proxy daemon", long_about = None)]
struct Args {
    /// Configuration file.
    #[arg(
        short = 'c',
        long = "config",
        alias = "conf",
        default_value = "/etc/glonax.conf",
        value_name = "FILE"
    )]
    config: std::path::PathBuf,
    /// Enable simulation mode.
    #[arg(long, default_value_t = false)]
    simulation: bool,
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

    let mut config: config::Config = glonax::from_file(args.config)?;

    if args.simulation {
        config.simulation.enabled = true;
    }
    if args.pilot_only {
        config.mode = config::OperationMode::PilotRestrict;
    }

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

    let instance = config.instance.clone();
    let instance2 = glonax::core::Instance::new(
        instance.id.clone(),
        instance.model.clone(),
        instance.machine_type,
        (1, 0, 0),
    );

    log::debug!("Starting proxy services");
    log::info!("Running in operation mode: {}", config.mode);
    log::info!("{}", instance2);

    if instance2.id().is_nil() {
        log::warn!("Instance ID is not set or invalid");
    }

    if config.simulation.enabled {
        log::info!("Running in simulation mode");
    }

    let mut runtime = glonax::runtime::builder(&config, instance2)?
        .with_shutdown()
        .enqueue_startup_motion(glonax::core::Motion::ResetAll)
        .build();

    runtime
        .schedule_interval::<glonax::components::Host>(Duration::from_millis(config.host.interval));

    if config.simulation.enabled {
        runtime.schedule_interval::<glonax::components::EncoderSimulator>(Duration::from_millis(5));
        runtime.schedule_interval::<glonax::components::EngineSimulator>(Duration::from_millis(10));

        runtime.schedule_motion_sink(device::sink_net_actuator_sim);
    } else {
        if config.nmea.is_some() {
            runtime.schedule_io_service(device::service_gnss);
        }

        runtime.schedule_j1939_service(j1939::rx_network_0, &config.j1939[0].interface);
        runtime.schedule_j1939_service(j1939::tx_network_0, &config.j1939[0].interface);
        runtime.schedule_j1939_service(j1939::rx_network_1, &config.j1939[1].interface);
        runtime.schedule_j1939_service(j1939::tx_network_1, &config.j1939[1].interface);

        runtime.schedule_j1939_motion_service(j1939::atx_network_1, &config.j1939[1].interface);
    }

    runtime.schedule_io_service(server::tcp_listen);
    runtime.schedule_io_service(server::unix_listen);
    runtime.schedule_io_service(server::net_announce);

    let mut components = vec![
        runtime.make_dynamic::<components::WorldBuilder>(0),
        runtime.make_dynamic::<components::SensorFusion>(2),
        runtime.make_dynamic::<components::LocalActor>(3),
    ];

    if config.mode == config::OperationMode::Autonomous {
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

    log::debug!("{} was shutdown gracefully", env!("CARGO_BIN_NAME"));

    Ok(())
}
