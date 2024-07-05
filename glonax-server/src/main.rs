// Copyright (C) 2024 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use clap::{Parser, ValueHint};

mod config;

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
    use glonax::consts::*;
    use glonax::service;

    let bin_name = env!("CARGO_BIN_NAME").to_string();
    let version_major: u8 = VERSION_MAJOR.parse().unwrap();
    let version_minor: u8 = VERSION_MINOR.parse().unwrap();
    let version_patch: u8 = VERSION_PATCH.parse().unwrap();

    let machine = config.machine.clone();
    let instance = glonax::core::Instance::new(
        machine.id.clone(),
        machine.model.clone(),
        machine.machine_type,
        (version_major, version_minor, version_patch),
        machine.serial.clone(),
    );

    glonax::log_system();

    log::info!("Starting {}", bin_name);
    log::info!("Runtime version: {}", VERSION);
    log::info!("Running in operation mode: {}", config.mode);
    log::info!("{}", instance);

    if instance.id().is_nil() {
        log::warn!("Instance ID is not set or invalid");
    }

    glonax::global::set_instance(instance);

    let mut runtime = glonax::Runtime::default();
    runtime.register_shutdown_signal();

    runtime.schedule_io_sub_service::<service::UnixServer, _>(config.clone().unix_listener);
    runtime.schedule_io_sub_service::<service::Director, _>(glonax::runtime::NullConfig {});

    for j1939_net_config in &config.j1939 {
        runtime.schedule_net_service::<service::NetworkAuthority, _>(
            j1939_net_config.clone(),
            std::time::Duration::from_millis(10),
        );
    }

    runtime.wait_for_shutdown().await;

    log::info!("Waiting for shutdown");

    runtime.wait_for_tasks().await;

    log::info!("{} was shutdown gracefully", bin_name);

    Ok(())
}
