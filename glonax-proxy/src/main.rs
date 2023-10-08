// Copyright (C) 2023 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use clap::Parser;

mod component;
mod config;

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
    #[arg(long, default_value_t = 500)]
    host_interval: u64,
    /// Configuration file.
    #[arg(long = "config", default_value = "/etc/glonax.conf")]
    config: std::path::PathBuf,
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

    let bin_name = env!("CARGO_BIN_NAME");

    let mut config = config::ProxyConfig {
        address: args.address,
        interface: args.interface,
        interface2: args.interface2,
        host_interval: args.host_interval,
        instance: glonax::from_file(args.config)?,
        global: glonax::GlobalConfig::default(),
    };

    config.global.bin_name = bin_name.to_string();
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

async fn daemonize(config: &config::ProxyConfig) -> anyhow::Result<()> {
    use std::sync::Arc;
    use tokio::sync::RwLock;

    log::info!("Starting proxy services");

    log::info!("Instance ID: {}", config.instance.id);
    log::info!("Instance Model: {}", config.instance.model);
    log::info!("Instance Name: {}", config.instance.name);

    if config.instance.id.starts_with("00000000") {
        log::warn!("Instance ID is not set or invalid");
    }

    let mut runtime = glonax::RuntimeBuilder::from_config(config)?
        // .with_shutdown()
        .build();

    let machine_state = Arc::new(RwLock::new(glonax::MachineState::new()));

    runtime
        .motion_tx
        .send(glonax::core::Motion::ResetAll)
        .await?;

    runtime.spawn_signal_service(component::service_host);
    runtime.spawn_signal_service(component::service_fifo);
    runtime.spawn_signal_service(component::service_gnss);
    runtime.spawn_signal_service(component::service_net_encoder);
    runtime.spawn_signal_service(component::service_net_ems);

    runtime.spawn_middleware_service(&machine_state, component::service_remote_probe);
    runtime.spawn_middleware_signal_sink(config, &machine_state, component::sink_proxy);

    runtime.spawn_motion_sink(component::sink_net_actuator);

    runtime
        .run_motion_service(&machine_state, component::service_remote_server)
        .await;

    // runtime.wait_for_shutdown().await;

    log::debug!("{} was shutdown gracefully", config.global.bin_name);

    Ok(())
}
