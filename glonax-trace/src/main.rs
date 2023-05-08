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
    /// CAN network interfaces.
    interface: Vec<String>,
    /// Trace items per file.
    #[arg(long, default_value_t = 100_000)]
    items_per_file: u32,
    /// Run motion requests slow.
    #[arg(long)]
    slow_motion: bool,
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

    let mut config = config::TraceConfig {
        interface: args.interface,
        items_per_file: args.items_per_file,
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

// TODO: Even though the same confiig is used as for the motion command, the signal listeners
// should be able to listen on a different network.
async fn net_listener(
    config: config::TraceConfig,
    writer: glonax::channel::BroadcastChannelWriter<glonax::transport::Signal>,
) {
    use glonax::channel::BroadcastSource;
    use glonax::net::{EncoderService, EngineManagementSystem, J1939Network};

    // TODO: Assign new network ID to each J1939 network.
    let mut router = glonax::net::Router::from_iter(
        config
            .interface
            .iter()
            .map(|iface| J1939Network::new(iface, DEVICE_NET_LOCAL_ADDR).unwrap()),
    );

    let mut engine_management_service = EngineManagementSystem::new(0x0);
    let mut encoder_list = vec![
        EncoderService::new(0x6A),
        EncoderService::new(0x6B),
        EncoderService::new(0x6C),
        EncoderService::new(0x6D),
    ];

    log::debug!("Starting network services");

    loop {
        if let Err(e) = router.listen().await {
            log::error!("{}", e);
        };

        if let Some(message) = router.try_accept(&mut engine_management_service) {
            log::trace!("0x{:X?} » {}", router.frame_source().unwrap(), message);

            message.fetch(&writer)
        }

        for encoder in &mut encoder_list {
            if let Some(message) = router.try_accept(encoder) {
                log::trace!("0x{:X?} » {}", router.frame_source().unwrap(), message);

                message.fetch(&writer);
            }
        }
    }
}

async fn daemonize(config: &config::TraceConfig) -> anyhow::Result<()> {
    use std::io::BufWriter;

    let runtime = glonax::RuntimeBuilder::from_config(config)?
        .with_shutdown()
        .build();

    let (signal_writer, mut signal_reader) = glonax::channel::broadcast_bichannel(10);

    runtime.spawn_background_task(net_listener(config.clone(), signal_writer));

    #[derive(serde::Serialize, Debug)]
    enum Metric {
        /// Temperature in celcius.
        Temperature(f32),
        /// Angle in radians.
        Angle(f32),
        /// Speed in meters per second.
        Speed(f32),
        /// Revolutions per minute.
        Rpm(i32),
        /// Acceleration in mg.
        Acceleration((f32, f32, f32)),
        /// Percentage.
        Percent(i32),
    }

    #[derive(serde::Serialize, Debug)]
    struct SignalRecord2 {
        timestamp: String,
        address: u32,
        function: u32,
        metric: Metric,
    }

    /// Create new file for output data.
    ///
    /// The file name is based on the current timestamp.
    fn create_file() -> anyhow::Result<BufWriter<std::fs::File>> {
        let file_name = format!(
            "trace/{}.json",
            chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
        );
        log::debug!("Open output file: {}", file_name);

        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_name)?;

        let writer = BufWriter::new(file);

        Ok(writer)
    }

    std::fs::create_dir_all(std::path::Path::new("trace"))?;

    let items_per_file = config.items_per_file;

    runtime.spawn_background_task(async move {
        use std::io::Write;

        let mut file_output = create_file().unwrap();

        let mut i = 0;
        while let Ok(signal) = signal_reader.recv().await {
            let signal_record = SignalRecord2 {
                timestamp: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                address: signal.address,
                function: signal.function,
                metric: match signal.metric.unwrap() {
                    glonax::transport::signal::Metric::Temperature(x) => Metric::Temperature(x),
                    glonax::transport::signal::Metric::Angle(x) => Metric::Angle(x),
                    glonax::transport::signal::Metric::Speed(x) => Metric::Speed(x),
                    glonax::transport::signal::Metric::Rpm(x) => Metric::Rpm(x),
                    glonax::transport::signal::Metric::Acceleration(x) => {
                        Metric::Acceleration((x.x, x.y, x.z))
                    }
                    glonax::transport::signal::Metric::Percent(x) => Metric::Percent(x),
                },
            };

            log::trace!("Writing to file {:?}", signal_record);

            if i > items_per_file {
                file_output.flush().unwrap();
                file_output = create_file().unwrap();
                i = 0;
            }

            serde_json::to_writer(&mut file_output, &signal_record).unwrap();
            file_output.write_all(b"\n").unwrap();

            i += 1;
        }
    });

    runtime.wait_for_shutdown().await;

    Ok(())
}
