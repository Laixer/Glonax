// Copyright (C) 2024 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use clap::{Parser, ValueHint};

mod components;
mod config;
mod device;
mod j1939;
mod server;

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
        value_name = "FILE",
        value_hint = ValueHint::FilePath
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

    let is_daemon = std::env::var("INVOCATION_ID").is_ok() || args.daemon;

    if is_daemon {
        log::set_max_level(LevelFilter::Debug);
        log::set_boxed_logger(Box::new(glonax::logger::SystemdLogger))?;
    } else {
        let log_level = if args.quiet {
            LevelFilter::Off
        } else {
            match args.verbose {
                0 => LevelFilter::Info,
                1 => LevelFilter::Debug,
                _ => LevelFilter::Trace,
            }
        };

        simplelog::TermLogger::init(
            log_level,
            simplelog::ConfigBuilder::new()
                .set_target_level(LevelFilter::Off)
                .set_location_level(LevelFilter::Off)
                .build(),
            simplelog::TerminalMode::Mixed,
            simplelog::ColorChoice::Auto,
        )?;
    }

    if is_daemon {
        log::info!("Running service as daemon");
    }

    log::trace!("{:#?}", config);

    run(config).await
}

async fn run(config: config::Config) -> anyhow::Result<()> {
    use glonax::driver::net::NetDriver;
    use glonax::service;
    use std::time::Duration;

    let bin_name = env!("CARGO_BIN_NAME").to_string();
    let version_major: u8 = glonax::consts::VERSION_MAJOR.parse().unwrap();
    let version_minor: u8 = glonax::consts::VERSION_MINOR.parse().unwrap();
    let version_patch: u8 = glonax::consts::VERSION_PATCH.parse().unwrap();

    let machine = config.machine.clone();
    let instance = glonax::core::Instance::new(
        machine.id.clone(),
        machine.model.clone(),
        machine.machine_type,
        (version_major, version_minor, version_patch),
    );

    log::info!("Starting {}", bin_name);
    log::debug!("Runtime version: {}", glonax::consts::VERSION);
    log::info!("Running in operation mode: {}", config.mode);
    log::info!("{}", instance);

    if instance.id().is_nil() {
        log::warn!("Instance ID is not set or invalid");
    }

    if config.simulation.enabled {
        log::info!("Running in simulation mode");
    }

    let mut runtime = glonax::runtime::builder(&config, instance.clone())?
        .with_shutdown()
        .enqueue_startup_motion(glonax::core::Motion::ResetAll)
        .build();

    runtime.schedule_service::<service::Host, service::HostConfig>(
        config.host.clone(),
        Duration::from_millis(config.host.interval.clamp(10, 1_000)),
    );

    if config.simulation.enabled {
        runtime.schedule_service::<service::EncoderSimulator, glonax::runtime::NullConfig>(
            glonax::runtime::NullConfig,
            Duration::from_millis(5),
        );
        runtime.schedule_service::<service::EngineSimulator, glonax::runtime::NullConfig>(
            glonax::runtime::NullConfig,
            Duration::from_millis(10),
        );

        runtime.schedule_motion_sink(device::sink_net_actuator_sim);
    } else {
        if let Some(gnss_config) = config.clone().gnss {
            runtime.schedule_io_service::<service::Gnss, service::GnssConfig>(gnss_config);
        }

        // TODO: Drivers may not match the configuration.
        let j1939_drivers_can0_rx = vec![
            NetDriver::kuebler_encoder(config.j1939[0].driver[0].id, config.j1939[0].address),
            NetDriver::kuebler_encoder(config.j1939[0].driver[1].id, config.j1939[0].address),
            NetDriver::kuebler_encoder(config.j1939[0].driver[2].id, config.j1939[0].address),
            NetDriver::kuebler_encoder(config.j1939[0].driver[3].id, config.j1939[0].address),
            NetDriver::kuebler_inclinometer(config.j1939[0].driver[4].id, config.j1939[0].address),
            NetDriver::request_responder(config.j1939[0].address),
        ];

        let j1939_drivers_can1_rx = vec![
            NetDriver::engine_management_system(
                config.j1939[1].driver[0].id,
                config.j1939[1].address,
            ),
            NetDriver::hydraulic_control_unit(
                config.j1939[1].driver[1].id,
                config.j1939[1].address,
            ),
            NetDriver::request_responder(config.j1939[1].address),
        ];

        let j1939_drivers_can1_tx = vec![
            NetDriver::engine_management_system(
                config.j1939[1].driver[0].id,
                config.j1939[1].address,
            ),
            NetDriver::hydraulic_control_unit(
                config.j1939[1].driver[1].id,
                config.j1939[1].address,
            ),
        ];

        runtime.schedule_j1939_service_rx(j1939_drivers_can0_rx, &config.j1939[0].interface);
        runtime.schedule_j1939_service_rx(j1939_drivers_can1_rx, &config.j1939[1].interface);
        runtime.schedule_j1939_service_tx(j1939_drivers_can1_tx, &config.j1939[1].interface);

        runtime.schedule_j1939_motion_service(j1939::atx_network_1, &config.j1939[1].interface);
    }

    runtime.schedule_io_func(server::tcp_listen);
    runtime.schedule_io_func(server::unix_listen);
    // runtime.schedule_io_service::<service::TcpServer, service::TcpServerConfig>(
    //     config.tcp_server.clone(),
    // );

    runtime.schedule_service::<service::Announcer, glonax::runtime::NullConfig>(
        glonax::runtime::NullConfig,
        Duration::from_millis(1_000),
    );

    let mut pipe = service::Pipeline::new(config.clone(), instance);

    pipe.insert_component::<components::WorldBuilder>(0);
    pipe.insert_component::<components::SensorFusion>(2);
    pipe.insert_component::<components::LocalActor>(3);

    if config.mode == config::OperationMode::Autonomous {
        pipe.insert_component::<components::Kinematic>(5);
        pipe.insert_component::<components::Controller>(10);
    }

    runtime.run_interval(pipe, Duration::from_millis(10)).await;

    log::debug!("Sending stop all motion to network");

    runtime.enqueue_motion(glonax::core::Motion::StopAll).await;

    // TODO: Shutdown all services and drivers.

    std::thread::sleep(Duration::from_millis(50));

    log::debug!("{} was shutdown gracefully", bin_name);

    Ok(())
}
