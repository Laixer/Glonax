// Copyright (C) 2024 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use clap::{Parser, ValueHint};

mod components;
mod config;
mod device;
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
        config.is_simulation = true;
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
    log::info!("Runtime version: {}", glonax::consts::VERSION);
    log::info!("Running in operation mode: {}", config.mode);
    log::info!("{}", instance);

    if instance.id().is_nil() {
        log::warn!("Instance ID is not set or invalid");
    }

    if config.is_simulation {
        log::info!("Running in simulation mode");
    }

    let mut runtime = glonax::runtime::builder(&config, instance.clone())?
        .with_shutdown()
        .build();

    runtime.schedule_service::<service::Host, _>(
        config.host.clone(),
        Duration::from_millis(config.host.interval.clamp(10, 1_000)),
    );

    // TODO: Do we need a simulator?
    if config.is_simulation {
        runtime.schedule_service::<service::EncoderSimulator, _>(
            glonax::runtime::NullConfig,
            Duration::from_millis(5),
        );
        runtime.schedule_service::<service::EngineSimulator, _>(
            glonax::runtime::NullConfig,
            Duration::from_millis(10),
        );

        runtime.schedule_motion_sink(device::sink_net_actuator_sim);
    } else {
        if let Some(gnss_config) = config.clone().gnss {
            runtime.schedule_io_service::<service::Gnss, _>(gnss_config);
        }

        // TODO: Check if RX,TX,ATX services should be scheduled.
        for j1939_net_config in &config.j1939 {
            use glonax::runtime::ControlNetwork;

            let mut net_rx = ControlNetwork::with_request_responder(j1939_net_config.address);
            for driver in &j1939_net_config.driver {
                let net_driver_config = glonax::driver::net::NetDriverConfig {
                    driver_type: driver.driver_type.clone(),
                    destination: driver.da,
                    source: driver.sa.unwrap_or(j1939_net_config.address), // TODO: Maybe remove 'source' from config.
                };

                net_rx.register_driver(NetDriver::try_from(net_driver_config).unwrap());
            }

            runtime.schedule_j1939_service_rx(net_rx, &j1939_net_config.interface);

            let mut net_tx = ControlNetwork::new(j1939_net_config.address);
            for driver in &j1939_net_config.driver {
                let net_driver_config = glonax::driver::net::NetDriverConfig {
                    driver_type: driver.driver_type.clone(),
                    destination: driver.da,
                    source: driver.sa.unwrap_or(j1939_net_config.address),
                };

                net_tx.register_driver(NetDriver::try_from(net_driver_config).unwrap());
            }

            runtime.schedule_j1939_service_tx(net_tx, &j1939_net_config.interface);
        }

        let j1939_index = 1;
        if j1939_index < config.j1939.len() {
            runtime.schedule_j1939_motion_service(
                glonax::runtime::atx_network_1,
                &config.j1939[j1939_index].interface,
            );
        }
    }

    if config.tcp_server.is_some() {
        runtime.schedule_io_func(server::tcp_listen);
    }
    runtime.schedule_io_func(server::unix_listen);
    // runtime.schedule_io_service::<service::TcpServer, service::TcpServerConfig>(
    //     config.tcp_server.clone(),
    // );

    runtime.schedule_service::<service::Announcer, _>(
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

    if config.mode != config::OperationMode::PilotRestrict {
        runtime.run_interval(pipe, Duration::from_millis(10)).await;
    } else {
        runtime.wait_for_shutdown().await;
    }

    log::debug!("Waiting for shutdown");

    // TODO: Shutdown all services and drivers.

    std::thread::sleep(Duration::from_millis(50));

    log::info!("{} was shutdown gracefully", bin_name);

    Ok(())
}
