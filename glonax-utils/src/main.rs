// Copyright (C) 2023 Laixer Equipment B.V.
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

async fn analyze_frames(mut router: Router) -> anyhow::Result<()> {
    debug!("Print incoming frames to screen");

    let mut engine_management_service = EngineManagementSystem::new(0x0);
    let mut frame_encoder = KueblerEncoderService::new(0x6A);
    let mut boom_encoder = KueblerEncoderService::new(0x6B);
    let mut arm_encoder = KueblerEncoderService::new(0x6C);
    let mut attachment_encoder = KueblerEncoderService::new(0x6D);
    // let mut actuator = ActuatorService::new(net.clone(), 0x4A);

    let mut app_inspector = J1939ApplicationInspector::new();

    loop {
        router.listen().await?;

        if let Some(ems) = router.try_accept2(&mut engine_management_service) {
            info!(
                "{} {} » {}",
                style_node(router.frame_source().unwrap()),
                Yellow.bold().paint("Engine"),
                ems
            );
        }

        if router.try_accept(&mut arm_encoder) {
            info!(
                "{} {} » {}",
                style_node(arm_encoder.node),
                Yellow.bold().paint("Arm"),
                arm_encoder
            );
        }

        if router.try_accept(&mut boom_encoder) {
            info!(
                "{} {} » {}",
                style_node(boom_encoder.node),
                Yellow.bold().paint("Boom"),
                boom_encoder
            );
        }

        if router.try_accept(&mut frame_encoder) {
            info!(
                "{} {} » {}",
                style_node(frame_encoder.node),
                Yellow.bold().paint("Frame"),
                frame_encoder
            );
        }

        if router.try_accept(&mut attachment_encoder) {
            info!(
                "{} {} » {}",
                style_node(attachment_encoder.node),
                Yellow.bold().paint("Attachment"),
                attachment_encoder
            );
        }

        // if router.try_accept(&mut actuator) {
        //     info!(
        //         "{} {} » {}",
        //         style_node(router.frame_source().unwrap()),
        //         Yellow.bold().paint("Hydraulic"),
        //         actuator
        //     );
        // }

        if router.try_accept(&mut app_inspector) {
            if let Some((major, minor, patch)) = app_inspector.software_identification() {
                info!(
                    "{} {} » Software identification: {}.{}.{}",
                    style_node(router.frame_source().unwrap()),
                    Yellow.bold().paint("Inspector"),
                    major,
                    minor,
                    patch
                );
            }
            if let Some(pgn) = app_inspector.request() {
                info!(
                    "{} {} » Request for PGN: {}",
                    style_node(router.frame_source().unwrap()),
                    Yellow.bold().paint("Inspector"),
                    pgn
                );
            }
            if let Some((function, arbitrary_address)) = app_inspector.address_claimed() {
                info!(
                    "{} {} » Adress claimed; Function: {}; Arbitrary address: {}",
                    style_node(router.frame_source().unwrap()),
                    Yellow.bold().paint("Inspector"),
                    function,
                    arbitrary_address
                );
            }

            if let Some(acknowledged) = app_inspector.acknowledged() {
                info!(
                    "{} {} » Acknowledged: {}",
                    style_node(router.frame_source().unwrap()),
                    Yellow.bold().paint("Inspector"),
                    acknowledged
                );
            }
        }
    }
}

async fn scan_nodes(mut router: Router) -> anyhow::Result<()> {
    loop {
        router.listen().await?;

        print!("{}c", 27 as char);

        for (node, last_seen) in router.node_table() {
            let x = if last_seen.elapsed().as_secs() < 1 {
                "now".to_owned()
            } else {
                format!("{} seconds ago", last_seen.elapsed().as_secs())
            };

            println!("Node: 0x{:X?} Last seen: {}", node, x);
        }
    }
}

/// Print frames to screen.
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
#[command(author = "Copyright (C) 2023 Laixer Equipment B.V.")]
#[command(version, propagate_version = true)]
#[command(about = "Glonax network diagnosis and system analyzer", long_about = None)]
struct Args {
    /// CAN network interface.
    #[arg(short, long, default_value = "can0")]
    interface: String,

    /// Local network address.
    #[arg(long, default_value_t = 0x9e)]
    address: u8,

    /// Level of verbosity.
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Node commands.
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Target node.
    Node {
        /// Target node address.
        address: String,

        #[command(subcommand)]
        command: NodeCommand,
    },
    /// Continuously scan for network nodes.
    Scan,
    /// Show raw frames on screen.
    Dump {
        /// Filter on PGN.
        #[arg(long)]
        pgn: Option<u32>,

        /// Filter on node.
        #[arg(long)]
        node: Option<String>,
    },
    /// Analyze network frames.
    Analyze {
        /// Filter on PGN.
        #[arg(long)]
        pgn: Option<u32>,

        /// Filter on node.
        #[arg(long)]
        node: Option<String>,
    },
}

#[derive(clap::Subcommand)]
enum NodeCommand {
    /// Enable or disable identification LED.
    Led { toggle: String },
    /// Assign the node a new address.
    Assign { address_new: String },
    /// Reset the node.
    Reset,
    /// Enable or disable motion lock.
    Motion { toggle: String },
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
        Command::Node { address, command } => match command {
            NodeCommand::Led { toggle } => {
                let node = node_address(address)?;

                info!(
                    "{} Turn identification LED {}",
                    style_node(node),
                    if string_to_bool(&toggle).unwrap() {
                        Green.paint("on")
                    } else {
                        Red.paint("off")
                    },
                );

                let net = J1939Network::new(args.interface.as_str(), args.address)?;
                net.send_vectored(&ActuatorService::set_led(string_to_bool(&toggle).unwrap()))
                    .await
                    .unwrap();
            }
            NodeCommand::Assign { address_new } => {
                let node = node_address(address)?;
                let node_new = node_address(address_new)?;

                info!("{} Assign 0x{:X?}", style_node(node), node_new);

                let net = J1939Network::new(args.interface.as_str(), args.address)?;
                net.set_address(node, node_new).await;
            }
            NodeCommand::Reset => {
                let node = node_address(address)?;

                info!("{} Reset", style_node(node));

                let net = J1939Network::new(args.interface.as_str(), args.address)?;
                net.send_vectored(&ActuatorService::reset()).await.unwrap();
            }
            NodeCommand::Motion { toggle } => {
                let node = node_address(address)?;

                info!(
                    "{} Turn motion {}",
                    style_node(node),
                    if string_to_bool(&toggle).unwrap() {
                        Green.paint("on")
                    } else {
                        Red.paint("off")
                    },
                );

                let net = J1939Network::new(args.interface.as_str(), args.address)?;
                if string_to_bool(&toggle).unwrap() {
                    net.send_vectored(&ActuatorService::lock()).await.unwrap();
                } else {
                    net.send_vectored(&ActuatorService::unlock()).await.unwrap();
                }
            }
            NodeCommand::Actuator { actuator, value } => {
                let node = node_address(address)?;

                let net = J1939Network::new(args.interface.as_str(), args.address)?;
                let mut service = ActuatorService::new(net, node);

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

                service.actuator_control([(actuator, value)].into()).await;
            }
        },
        Command::Scan => {
            let net = J1939Network::new(args.interface.as_str(), args.address)?;
            net.set_promisc_mode(true)?;

            scan_nodes(Router::new(net)).await?;
        }
        Command::Dump { pgn, node } => {
            let net = J1939Network::new(args.interface.as_str(), args.address)?;
            net.set_promisc_mode(true)?;

            let mut router = Router::new(net);

            if let Some(pgn) = pgn {
                router.add_pgn_filter(pgn);
            }
            if let Some(node) = node.map(|s| node_address(s).unwrap()) {
                router.add_node_filter(node);
            }

            print_frames(router).await?;
        }
        Command::Analyze { pgn, node } => {
            let net = J1939Network::new(args.interface.as_str(), args.address)?;
            net.set_promisc_mode(true)?;

            let mut router = Router::new(net);

            if let Some(pgn) = pgn {
                router.add_pgn_filter(pgn);
            }
            if let Some(node) = node.map(|s| node_address(s).unwrap()) {
                router.add_node_filter(node);
            }

            analyze_frames(router).await?;
        }
    }

    Ok(())
}
