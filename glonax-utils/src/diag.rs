// Copyright (C) 2023 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use clap::Parser;

use log::info;

#[derive(Parser)]
#[command(author = "Copyright (C) 2023 Laixer Equipment B.V.")]
#[command(version, propagate_version = true)]
#[command(about = "Glonax network diagnosis and system analyzer", long_about = None)]
struct Args {
    /// Level of verbosity.
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
    /// Commands.
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Scan the network for nodes.
    Scan,
    // / Show raw frames on screen.
    // Dump,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let log_config = simplelog::ConfigBuilder::new()
        .set_time_level(log::LevelFilter::Off)
        .set_thread_level(log::LevelFilter::Off)
        .set_target_level(log::LevelFilter::Off)
        .set_location_level(log::LevelFilter::Off)
        .add_filter_ignore_str("sled")
        .add_filter_ignore_str("mio")
        .build();

    let log_level = match args.verbose {
        0 => log::LevelFilter::Info,
        1 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };

    simplelog::TermLogger::init(
        log_level,
        log_config,
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )?;

    match args.command {
        Command::Scan => {
            let broadcast_addr = std::net::SocketAddrV4::new(
                std::net::Ipv4Addr::UNSPECIFIED,
                glonax::consts::DEFAULT_NETWORK_PORT,
            );

            let socket = tokio::net::UdpSocket::bind(broadcast_addr).await?;

            let mut buffer = [0u8; 1024];

            loop {
                let (size, socket_addr) = socket.recv_from(&mut buffer).await?;
                if let Ok(frame) = glonax::transport::frame::Frame::try_from(&buffer[..size]) {
                    if frame.message == glonax::transport::frame::FrameMessage::Instance {
                        let instance =
                            glonax::core::Instance::try_from(&buffer[frame.payload_range()])
                                .unwrap();

                        info!("{} » {}", socket_addr.ip(), instance);
                    }
                }
            }
        }
        // Command::Dump => {
            // let broadcast_addr = std::net::SocketAddrV4::new(
            //     std::net::Ipv4Addr::UNSPECIFIED,
            //     glonax::consts::DEFAULT_NETWORK_PORT,
            // );

            // let socket = tokio::net::UdpSocket::bind(broadcast_addr).await?;

            // let mut buffer = [0u8; 1024];

            // loop {
            //     let (size, _) = socket.recv_from(&mut buffer).await?;
            //     if let Ok(frame) = glonax::transport::frame::Frame::try_from(&buffer[..size]) {
            //         if frame.message == glonax::transport::frame::FrameMessage::Signal {
            //             let signal =
            //                 glonax::core::Signal::try_from(&buffer[frame.payload_range()]).unwrap();

            //             info!("{}", signal.metric);
            //         }
            //     }
            // }
        // }
    }
}
