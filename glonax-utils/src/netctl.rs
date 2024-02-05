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

/// Analyze incoming frames and print their contents to the screen.
async fn analyze_frames(mut router: Router) -> anyhow::Result<()> {
    use glonax::device::{
        EngineManagementSystem, HydraulicControlUnit, J1939ApplicationInspector, KueblerEncoder, J1939Message
    };

    debug!("Print incoming frames to screen");

    let mut ems0 = EngineManagementSystem;
    let mut enc0 = KueblerEncoder::new(0x6A);
    let mut enc1 = KueblerEncoder::new(0x6B);
    let mut enc2 = KueblerEncoder::new(0x6C);
    let mut enc3 = KueblerEncoder::new(0x6D);
    let mut hcu0 = HydraulicControlUnit::new(0x4A);
    let mut app_inspector = J1939ApplicationInspector;

    loop {
        router.listen().await?;

        if let Some(message) = router.try_accept(&mut ems0) {
            info!(
                "{} {} » {}",
                style_node(router.frame_source().unwrap()),
                Yellow.bold().paint("Engine"),
                message
            );
        } else if let Some(message) = router.try_accept(&mut enc2) {
            info!(
                "{} {} » {}",
                style_node(router.frame_source().unwrap()),
                Yellow.bold().paint("Arm"),
                message
            );
        } else if let Some(message) = router.try_accept(&mut enc1) {
            info!(
                "{} {} » {}",
                style_node(router.frame_source().unwrap()),
                Yellow.bold().paint("Boom"),
                message
            );
        } else if let Some(message) = router.try_accept(&mut enc0) {
            info!(
                "{} {} » {}",
                style_node(router.frame_source().unwrap()),
                Yellow.bold().paint("Frame"),
                message
            );
        } else if let Some(message) = router.try_accept(&mut enc3) {
            info!(
                "{} {} » {}",
                style_node(router.frame_source().unwrap()),
                Yellow.bold().paint("Attachment"),
                message
            );
        } else if let Some(message) = router.try_accept(&mut hcu0) {
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
                J1939Message::SoftwareIndent((major, minor, patch)) => {
                    info!(
                        "{} {} » Software identification: {}.{}.{}",
                        style_node(router.frame_source().unwrap()),
                        Yellow.bold().paint("Inspector"),
                        major,
                        minor,
                        patch
                    );
                }
                J1939Message::RequestPGN(pgn) => {
                    info!(
                        "{} {} » Request for PGN: {}",
                        style_node(router.frame_source().unwrap()),
                        Yellow.bold().paint("Inspector"),
                        pgn
                    );
                }
                J1939Message::AddressClaim(name) => {
                    info!(
                        "{} {} » Identity number: 0x{:X}; Manufacturer code: 0x{:X}; Function instance: 0x{:X}; ECU instance: 0x{:X}; Function: 0x{:X}; Vehicle system: 0x{:X}; Vehicle system instance: 0x{:X}; Industry group: {:X}; Arbitrary address: {}",
                        style_node(router.frame_source().unwrap()),
                        Yellow.bold().paint("Inspector"),
                        name.identity_number,
                        name.manufacturer_code,
                        name.function_instance,
                        name.ecu_instance,
                        name.function,
                        name.vehicle_system,
                        name.vehicle_system_instance,
                        name.industry_group,
                        name.arbitrary_address 
                    );
                }
                J1939Message::Acknowledged(acknowledged) => {
                    info!(
                        "{} {} » Acknowledged: {}",
                        style_node(router.frame_source().unwrap()),
                        Yellow.bold().paint("Inspector"),
                        acknowledged
                    );
                }
                J1939Message::TimeDate(time) => {
                    info!(
                        "{} {} » Time and date: {}",
                        style_node(router.frame_source().unwrap()),
                        Yellow.bold().paint("Inspector"),
                        time
                    );
                }
                J1939Message::ProprietaryB(data) => {
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
        /// Target node address.
        #[arg(long, default_value = "0x0")]
        address: String,
        /// Engine commands.
        #[command(subcommand)]
        command: EngineCommand,
    },
    Request {
        /// Target node address.
        #[arg(long)]
        address: String,
        /// Request commands.
        #[command(subcommand)]
        command: RequestCommand,
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

#[derive(clap::Subcommand, PartialEq, Eq)]
enum RequestCommand {
    /// Request node name.
    Name,
    /// Request node software version.
    Software,
    /// Request node component identification.
    Component,
    /// Request node vehicle identification.
    Vehicle,
    /// Request node time and date.
    Time,
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
    /// Enable or disable identification mode.
    Ident { toggle: String },
    /// Assign the node a new address.
    Assign { address_new: String },
    /// Reboot the node.
    Reboot,
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
            let socket = CANSocket::bind(&SockAddrCAN::new(args.interface.as_str()))?;
            let hcu0 = glonax::device::HydraulicControlUnit::new(node);

            match command {
                HCUCommand::Ident { toggle } => {
                    info!(
                        "{} Turn identification mode {}",
                        style_node(node),
                        if toggle.parse::<bool>()? {
                            Green.paint("on")
                        } else {
                            Red.paint("off")
                        },
                    );

                    socket.send_vectored(&hcu0.set_ident(toggle.parse::<bool>()?)).await?;
                }
                HCUCommand::Reboot => {
                    info!("{} Reboot", style_node(node));

                    socket.send_vectored(&hcu0.reboot()).await?;
                }
                HCUCommand::MotionReset => {
                    info!("{} Motion reset", style_node(node));

                    socket.send_vectored(&hcu0.motion_reset()).await?;
                }
                HCUCommand::Lock { toggle } => {
                    info!(
                        "{} Turn lock {}",
                        style_node(node),
                        if toggle.parse::<bool>()? {
                            Green.paint("on")
                        } else {
                            Red.paint("off")
                        },
                    );

                    if toggle.parse::<bool>()? {
                        socket.send_vectored(&hcu0.lock()).await?;
                    } else {
                        socket.send_vectored(&hcu0.unlock()).await?;
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

                    socket.send_vectored(&hcu0.actuator_command([(actuator, value)].into())).await?;
                }
                HCUCommand::Assign { address_new } => {
                    let node_new = node_address(address_new)?;

                    info!("{} Assign 0x{:X?}", style_node(node), node_new);

                    socket.send_vectored(&commanded_address(node, node_new)).await?;
                }
            }
        }
        Command::Engine { address, command } => {
            let node = node_address(address)?;
            let socket = CANSocket::bind(&SockAddrCAN::new(args.interface.as_str()))?;
            let ems0 = glonax::device::EngineManagementSystem;

            match command {
                EngineCommand::Rpm { rpm } => {
                    info!("{} Set RPM to {}", style_node(node), rpm);

                    let mut tick = tokio::time::interval(std::time::Duration::from_millis(10));

                    loop {
                        tick.tick().await;
                        socket.send_vectored(&ems0.speed_request(rpm)).await?;
                    }
                }
                EngineCommand::Start => {
                    info!("{} Start engine", style_node(node));

                    let mut tick = tokio::time::interval(std::time::Duration::from_millis(10));

                    loop {
                        tick.tick().await;
                        socket.send_vectored(&ems0.start()).await?;
                    }
                }
                EngineCommand::Stop => {
                    info!("{} Stop engine", style_node(node));

                    let mut tick = tokio::time::interval(std::time::Duration::from_millis(10));

                    loop {
                        tick.tick().await;
                        socket.send_vectored(&ems0.shutdown()).await?;
                    }
                }
            }
        }
        Command::Request { address, command } => {
            use glonax::j1939::{PGN, protocol};

            let node = node_address(address)?;
            let socket = CANSocket::bind(&SockAddrCAN::new(args.interface.as_str()))?;

            let pgn = match command {
                RequestCommand::Name => PGN::AddressClaimed,
                RequestCommand::Software => PGN::SoftwareIdentification,
                RequestCommand::Component => PGN::ComponentIdentification,
                RequestCommand::Vehicle => PGN::VehicleIdentification,
                RequestCommand::Time => PGN::TimeDate,
            };

            info!("{} Request {:?}", style_node(node), pgn);

            socket.send(&protocol::request(node, pgn)).await?;
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
                router.add_node_filter(node?);
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
                router.add_node_filter(node?);
            }

            analyze_frames(router).await?;
        }
    }

    Ok(())
}
