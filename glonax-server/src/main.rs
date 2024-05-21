// Copyright (C) 2024 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use clap::{Parser, ValueHint};

mod components;
mod config;

/// Interval for the host service.
// const SERVICE_HOST_INTERVAL: std::time::Duration = std::time::Duration::from_millis(200);

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

    glonax::log_system();

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

    // TODO: Let the runtie builder set the instance
    glonax::global::set_instance(instance);

    let mut runtime = glonax::runtime::builder()?.with_shutdown().build();

    runtime.schedule_io_service::<service::Host, _>(glonax::runtime::NullConfig {});

    if config.is_simulation {
        runtime.schedule_command_service::<service::ActuatorSimulator, _>(
            glonax::runtime::NullConfig {},
        );
    } else {
        if let Some(gnss_config) = config.clone().gnss {
            runtime.schedule_io_service::<service::Gnss, _>(gnss_config);
        }

        for j1939_net_config in &config.j1939 {
            runtime.schedule_io_service::<service::NetworkAuthorityRx, _>(j1939_net_config.clone());

            // TODO: Why not on all J1939 units?
            if j1939_net_config.authority_atx {
                runtime.schedule_command_service::<service::NetworkAuthorityAtx, _>(
                    j1939_net_config.clone(),
                );
            }
        }
    }

    if let Some(tcp_server) = config.tcp_server.clone() {
        runtime.schedule_io_service::<service::TcpServer, _>(tcp_server);
    }

    // TODO: Let runtime instantiate the pipeline
    use glonax::runtime::Service;
    let mut pipe = service::Pipeline::new(glonax::runtime::NullConfig {});

    // pipe.add_component_default::<glonax::components::HostComponent>();
    pipe.add_init_component::<glonax::components::Acquisition>();

    if config.is_simulation {
        pipe.add_component_default::<glonax::components::EncoderSimulator>();
        pipe.add_component_default::<glonax::components::EngineSimulator>();
    }

    pipe.add_component_default::<glonax::components::StatusComponent>();
    pipe.add_component_default::<glonax::components::EngineComponent>();
    pipe.add_component_default::<glonax::components::ControlComponent>();
    pipe.add_component_default::<components::WorldBuilder>();

    if config.mode != config::OperationMode::PilotRestrict {
        pipe.add_component_default::<components::SensorFusion>(); // TODO: Replaced by AQ?
        pipe.add_component_default::<components::LocalActor>();
    }

    if config.mode == config::OperationMode::Autonomous {
        pipe.add_component_default::<components::Kinematic>();
        pipe.add_component_default::<components::Controller>();
    } else {
        // pipe.add_component_default::<glonax::components::HydraulicComponent>();
    }

    runtime.run_interval(pipe, Duration::from_millis(10)).await;

    log::info!("Waiting for shutdown");

    runtime.wait_for_tasks().await;

    log::info!("{} was shutdown gracefully", bin_name);

    Ok(())
}
