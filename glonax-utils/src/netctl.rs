// Copyright (C) 2024 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use ansi_term::Colour::{Blue, Green, Purple, Red, Yellow};
use clap::Parser;
use glonax::net::*;

use log::{debug, info};

fn style_node(address: u8) -> String {
    Purple.paint(format!("[node 0x{:X?}]", address)).to_string()
}

fn node_address(address: String) -> Result<u8, std::num::ParseIntError> {
    if address.starts_with("0x") {
        u8::from_str_radix(address.trim_start_matches("0x"), 16)
    } else {
        u8::from_str_radix(address.as_str(), 16)
    }
}

fn string_to_bool(str: &str) -> Result<bool, ()> {
    match str.to_lowercase().trim() {
        "yes" => Ok(true),
        "true" => Ok(true),
        "on" => Ok(true),
        "1" => Ok(true),
        "no" => Ok(false),
        "false" => Ok(false),
        "off" => Ok(false),
        "0" => Ok(false),
        _ => Err(()),
    }
}

/// Analyze incoming frames and print their contents to the screen.
async fn analyze_frames(mut router: Router) -> anyhow::Result<()> {
    use glonax::device::{
        EngineManagementSystem, HydraulicControlUnit, J1939ApplicationInspector, KueblerEncoder,
    };

    debug!("Print incoming frames to screen");

    let mut engine_management_service = EngineManagementSystem;
    let mut frame_encoder = KueblerEncoder::new(0x6A);
    let mut boom_encoder = KueblerEncoder::new(0x6B);
    let mut arm_encoder = KueblerEncoder::new(0x6C);
    let mut attachment_encoder = KueblerEncoder::new(0x6D);
    let mut hcu = HydraulicControlUnit::new(0x4A);
    let mut app_inspector = J1939ApplicationInspector;

    loop {
        router.listen().await?;

        if let Some(message) = router.try_accept(&mut engine_management_service) {
            info!(
                "{} {} » {}",
                style_node(router.frame_source().unwrap()),
                Yellow.bold().paint("Engine"),
                message
            );
        } else if let Some(message) = router.try_accept(&mut arm_encoder) {
            info!(
                "{} {} » {}",
                style_node(router.frame_source().unwrap()),
                Yellow.bold().paint("Arm"),
                message
            );
        } else if let Some(message) = router.try_accept(&mut boom_encoder) {
            info!(
                "{} {} » {}",
                style_node(router.frame_source().unwrap()),
                Yellow.bold().paint("Boom"),
                message
            );
        } else if let Some(message) = router.try_accept(&mut frame_encoder) {
            info!(
                "{} {} » {}",
                style_node(router.frame_source().unwrap()),
                Yellow.bold().paint("Frame"),
                message
            );
        } else if let Some(message) = router.try_accept(&mut attachment_encoder) {
            info!(
                "{} {} » {}",
                style_node(router.frame_source().unwrap()),
                Yellow.bold().paint("Attachment"),
                message
            );
        } else if let Some(message) = router.try_accept(&mut hcu) {
            if let Some(actuator_message) = message.0 {
                info!(
                    "{} {} » {}",
                    style_node(router.frame_source().unwrap()),
                    Yellow.bold().paint("HCU"),
                    actuator_message
                );
            } else if let Some(motion_message) = message.1 {
                info!(
                    "{} {} » {}",
                    style_node(router.frame_source().unwrap()),
                    Yellow.bold().paint("HCU"),
                    motion_message
                );
            } else if let Some(status_message) = message.2 {
                info!(
                    "{} {} » {}",
                    style_node(router.frame_source().unwrap()),
                    Yellow.bold().paint("HCU"),
                    status_message
                );
            }
        } else if let Some(message) = router.try_accept(&mut app_inspector) {
            match message {
                glonax::device::J1939Message::SoftwareIndent((major, minor, patch)) => {
                    info!(
                        "{} {} » Software identification: {}.{}.{}",
                        style_node(router.frame_source().unwrap()),
                        Yellow.bold().paint("Inspector"),
                        major,
                        minor,
                        patch
                    );
                }
                glonax::device::J1939Message::RequestPGN(pgn) => {
                    info!(
                        "{} {} » Request for PGN: {}",
                        style_node(router.frame_source().unwrap()),
                        Yellow.bold().paint("Inspector"),
                        pgn
                    );
                }
                glonax::device::J1939Message::AddressClaim(name) => {
                    // info!(
                    //     "{} {} » Adress claimed; Function: {}; Arbitrary address: {}",
                    //     style_node(router.frame_source().unwrap()),
                    //     Yellow.bold().paint("Inspector"),
                    //     function,
                    //     arbitrary_address
                    // );
                    info!(
                        "{} {} » Adress claimed: {:?}",
                        style_node(router.frame_source().unwrap()),
                        Yellow.bold().paint("Inspector"),
                        name
                    );
                }
                glonax::device::J1939Message::Acknowledged(acknowledged) => {
                    info!(
                        "{} {} » Acknowledged: {}",
                        style_node(router.frame_source().unwrap()),
                        Yellow.bold().paint("Inspector"),
                        acknowledged
                    );
                }
                glonax::device::J1939Message::TimeDate(time) => {
                    info!(
                        "{} {} » Time and date: {}",
                        style_node(router.frame_source().unwrap()),
                        Yellow.bold().paint("Inspector"),
                        time
                    );
                }
                glonax::device::J1939Message::ProprietaryB(data) => {
                    debug!(
                        "{} {} » Proprietary B: {:02X?}",
                        style_node(router.frame_source().unwrap()),
                        Yellow.bold().paint("Inspector"),
                        data
                    );
                }
            }
        }
    }
}

/// Print raw frames to standard output.
async fn print_frames(mut router: Router) -> anyhow::Result<()> {
    debug!("Print incoming frames to screen");

    loop {
        router.listen().await?;

        if let Some(frame) = router.take() {
            println!("{}", frame);
        };
    }
}

#[derive(Parser)]
#[command(author = "Copyright (C) 2024 Laixer Equipment B.V.")]
#[command(version, propagate_version = true)]
#[command(about = "Glonax network diagnosis and system analyzer", long_about = None)]
struct Args {
    /// CAN network interface.
    #[arg(short = 'i', long, default_value = "can0")]
    interface: String,
    /// Local network address.
    #[arg(long, default_value_t = 0x9e)]
    address: u8,
    /// Level of verbosity.
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
    /// Commands.
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Hydraulics control unit commands.
    Hcu {
        /// Target node address.
        #[arg(long, default_value = "0x4A")]
        address: String,
        /// Node commands.
        #[command(subcommand)]
        command: HCUCommand,
    },
    /// Engine control unit commands.
    Engine {
        /// Engine commands.
        #[command(subcommand)]
        command: EngineCommand,
    },
    /// Show raw frames on screen.
    Dump {
        /// Filter on PGN.
        #[arg(long)]
        pgn: Vec<u32>,
        /// Filter on node.
        #[arg(long)]
        node: Vec<String>,
    },
    /// Analyze network frames.
    Analyze {
        /// Filter on PGN.
        #[arg(long)]
        pgn: Vec<u32>,
        /// Filter on node.
        #[arg(long)]
        node: Vec<String>,
    },
}

#[derive(clap::Subcommand)]
enum EngineCommand {
    /// Request engine RPM.
    Rpm { rpm: u16 },
    /// Request engine start.
    Start,
    /// Request engine stop.
    Stop,
}

#[derive(clap::Subcommand)]
enum HCUCommand {
    /// Enable or disable identification LED.
    Led { toggle: String },
    /// Assign the node a new address.
    Assign { address_new: String },
    /// Reset the node.
    Reset,
    /// Motion reset.
    MotionReset,
    /// Enable or disable motion lock.
    Lock { toggle: String },
    /// Actuator motion.
    Actuator { actuator: u8, value: i16 },
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

    debug!("Bind to interface {}", args.interface);

    match args.command {
        Command::Hcu { address, command } => {
            let node = node_address(address)?;
            let service = glonax::device::HydraulicControlUnit::new(node);
            let net = J1939Network::new(args.interface.as_str(), args.address)?;

            match command {
                HCUCommand::Led { toggle } => {
                    info!(
                        "{} Turn identification LED {}",
                        style_node(node),
                        if string_to_bool(&toggle).unwrap() {
                            Green.paint("on")
                        } else {
                            Red.paint("off")
                        },
                    );

                    net.send_vectored(&service.set_led(string_to_bool(&toggle).unwrap()))
                        .await
                        .unwrap();
                }
                HCUCommand::Reset => {
                    info!("{} Reset", style_node(node));

                    net.send_vectored(&service.reset()).await.unwrap();
                }
                HCUCommand::MotionReset => {
                    info!("{} Motion reset", style_node(node));

                    net.send_vectored(&service.motion_reset()).await.unwrap();
                }
                HCUCommand::Lock { toggle } => {
                    info!(
                        "{} Turn lock {}",
                        style_node(node),
                        if string_to_bool(&toggle).unwrap() {
                            Green.paint("on")
                        } else {
                            Red.paint("off")
                        },
                    );

                    if string_to_bool(&toggle).unwrap() {
                        net.send_vectored(&service.lock()).await.unwrap();
                    } else {
                        net.send_vectored(&service.unlock()).await.unwrap();
                    }
                }
                HCUCommand::Actuator { actuator, value } => {
                    info!(
                        "{} Set actuator {} to {}",
                        style_node(node),
                        actuator,
                        if value.is_positive() {
                            Blue.paint(value.to_string())
                        } else {
                            Green.paint(value.abs().to_string())
                        },
                    );

                    net.send_vectored(&service.actuator_command([(actuator, value)].into()))
                        .await
                        .unwrap();
                }
                HCUCommand::Assign { address_new } => {
                    let node_new = node_address(address_new)?;

                    info!("{} Assign 0x{:X?}", style_node(node), node_new);

                    net.commanded_address(node, node_new).await;
                }
            }
        }
        Command::Engine { command } => {
            let service = glonax::device::EngineManagementSystem;
            let net = J1939Network::new(args.interface.as_str(), args.address)?;

            match command {
                EngineCommand::Rpm { rpm } => {
                    info!("{} Set RPM to {}", style_node(0x0), rpm);

                    let mut tick = tokio::time::interval(std::time::Duration::from_millis(10));

                    loop {
                        tick.tick().await;
                        net.send_vectored(&service.set_rpm(rpm)).await.unwrap();
                    }
                }
                EngineCommand::Start => {
                    info!("{} Start engine", style_node(0x0));

                    // net.send_vectored(&service.start()).await.unwrap();
                }
                EngineCommand::Stop => {
                    info!("{} Stop engine", style_node(0x0));

                    // net.send_vectored(&service.stop()).await.unwrap();
                }
            }
        }
        Command::Dump { pgn, node } => {
            let net = J1939Network::new(args.interface.as_str(), args.address)?;
            let mut router = Router::new(net);

            for pgn in pgn {
                router.add_pgn_filter(pgn);
            }
            for node in node
                .iter()
                .map(|s| node_address(s.to_owned()))
                .filter(|a| a.is_ok())
            {
                router.add_node_filter(node.unwrap());
            }

            print_frames(router).await?;
        }
        Command::Analyze { pgn, node } => {
            let net = J1939Network::new(args.interface.as_str(), args.address)?;
            let mut router = Router::new(net);

            for pgn in pgn {
                router.add_pgn_filter(pgn);
            }
            for node in node
                .iter()
                .map(|s| node_address(s.to_owned()))
                .filter(|a| a.is_ok())
            {
                router.add_node_filter(node.unwrap());
            }

            analyze_frames(router).await?;
        }
    }

    Ok(())
}
