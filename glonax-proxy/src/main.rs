// Copyright (C) 2023 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use clap::Parser;

mod config;

#[derive(Parser)]
#[command(author = "Copyright (C) 2023 Laixer Equipment B.V.")]
#[command(version, propagate_version = true)]
#[command(about = "Glonax proxy daemon", long_about = None)]
struct Args {
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
    use tokio::net::TcpListener;
    use tokio::sync::broadcast::{self, Sender};

    let listener = TcpListener::bind("0.0.0.0:30051").await?;

    log::debug!("Starting proxy services");

    let (tx, _rx) = broadcast::channel(8);

    let sender: Sender<glonax::core::Signal> = tx.clone();

    tokio::spawn(async move {
        let file = tokio::fs::OpenOptions::new()
            .read(true)
            .open("signal")
            .await
            .unwrap();

        let mut protocol = glonax::transport::Protocol::new(file);

        while let Ok(message) = protocol.read_frame().await {
            if let glonax::transport::Message::Signal(signal) = message {
                // log::debug!("Received signal: {}", signal);

                if let Err(e) = sender.send(signal) {
                    log::error!("Failed to send signal: {}", e);
                }
            }
        }
    });

    loop {
        let (stream, addr) = listener.accept().await?;

        log::info!("Accepted connection from: {}", addr);

        let (stream_reader, stream_writer) = stream.into_split();

        let mut rx = tx.subscribe();

        tokio::spawn(async move {
            let mut protocol_out = glonax::transport::Protocol::new(stream_writer);

            while let Ok(signal) = rx.recv().await {
                if let Err(e) = protocol_out.write_frame6(signal).await {
                    // log::error!("Failed to write to socket: {}", e);
                    break;
                }
            }

            log::info!("Signal listener shutdown");
        });

        tokio::spawn(async move {
            let mut session_name = String::new();

            let file = tokio::fs::OpenOptions::new()
                .write(true)
                .open("motion")
                .await
                .unwrap();

            let mut protocol_out = glonax::transport::Protocol::new(file);

            let mut protocol_in = glonax::transport::Protocol::new(stream_reader);

            while let Ok(message) = protocol_in.read_frame().await {
                match message {
                    glonax::transport::Message::Start(session) => {
                        log::info!("Session started for: {}", session.name());
                        session_name = session.name().to_string();
                    }
                    glonax::transport::Message::Shutdown => {
                        log::info!("Session shutdown for: {}", session_name);
                        break;
                    }
                    glonax::transport::Message::Motion(motion) => {
                        log::debug!("Received motion: {}", motion);

                        if let Err(e) = protocol_out.write_frame5(motion).await {
                            log::error!("Failed to write to socket: {}", e);
                        }
                    }
                    _ => {}
                }
            }

            log::info!("Connection closed for: {}", addr);
        });
    }

    // log::debug!("{} was shutdown gracefully", config.global.bin_name);

    // Ok(())
}
