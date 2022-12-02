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

fn string_to_bool(str: &String) -> Result<bool, ()> {
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

async fn analyze_frames(net: std::sync::Arc<ControlNet>, mut router: Router) -> anyhow::Result<()> {
    debug!("Print incoming frames to screen");

    let mut engine_service = EngineService::new(0x0);
    let mut arm_encoder = KueblerEncoderService::new(net.clone(), 0x6C);
    let mut boom_encoder = KueblerEncoderService::new(net.clone(), 0x6A);
    let mut turn_encoder = KueblerEncoderService::new(net.clone(), 0x20);
    let mut actuator = ActuatorService::new(net.clone(), 0x4A);

    let mut app_inspector = J1939ApplicationInspector::new();

    loop {
        router.accept().await?;

        if router.try_accept(&mut engine_service) {
            info!(
                "{} {} » {}",
                style_node(router.frame_source().unwrap()),
                Yellow.bold().paint("Engine"),
                engine_service
            );
        }

        if router.try_accept(&mut arm_encoder) {
            info!(
                "{} {} » {}",
                style_node(router.frame_source().unwrap()),
                Yellow.bold().paint("Arm"),
                arm_encoder
            );
        }

        if router.try_accept(&mut boom_encoder) {
            info!(
                "{} {} » {}",
                style_node(router.frame_source().unwrap()),
                Yellow.bold().paint("Boom"),
                boom_encoder
            );
        }

        if router.try_accept(&mut turn_encoder) {
            info!(
                "{} {} » {}",
                style_node(router.frame_source().unwrap()),
                Yellow.bold().paint("Turn"),
                turn_encoder
            );
        }

        if router.try_accept(&mut actuator) {
            info!(
                "{} {} » {}",
                style_node(router.frame_source().unwrap()),
                Yellow.bold().paint("Hydraulic"),
                actuator
            );
        }

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

/// Print frames to screen.
async fn print_frames(mut router: Router) -> anyhow::Result<()> {
    debug!("Print incoming frames to screen");

    loop {
        router.accept().await?;

        if let Some(frame) = router.take() {
            info!("{}", frame);
        };
    }
}

#[derive(Parser)]
#[command(author = "Copyright (C) 2022 Laixer Equipment B.V.")]
#[command(version, propagate_version = true)]
#[command(about = "Network diagnosis and system analyzer", long_about = None)]
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
        .set_target_level(log::LevelFilter::Off)
        .set_thread_level(log::LevelFilter::Off)
        .build();

    let log_level = match args.verbose {
        0 => log::LevelFilter::Info,
        1 => log::LevelFilter::Debug,
        2 | _ => log::LevelFilter::Trace,
    };

    simplelog::TermLogger::init(
        log_level,
        log_config,
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )?;

    debug!("Bind to interface {}", args.interface);

    let net = ControlNet::new(args.interface.as_str(), args.address)?;

    match args.command {
        Command::Node { address, command } => match command {
            NodeCommand::Led { toggle } => {
                let node = node_address(address)?;

                let service = StatusService::new(std::sync::Arc::new(net), node);

                info!(
                    "{} Turn identification LED {}",
                    style_node(node),
                    if string_to_bool(&toggle).unwrap() {
                        Green.paint("on")
                    } else {
                        Red.paint("off")
                    },
                );

                service.set_led(string_to_bool(&toggle).unwrap()).await;
            }
            NodeCommand::Assign { address_new } => {
                let node = node_address(address)?;
                let node_new = node_address(address_new)?;

                info!("{} Assign 0x{:X?}", style_node(node), node_new);

                net.set_address(node, node_new).await;
            }
            NodeCommand::Reset => {
                let node = node_address(address)?;

                info!("{} Reset", style_node(node));

                net.reset(node).await;
            }
            NodeCommand::Motion { toggle } => {
                let node = node_address(address)?;

                let mut service = ActuatorService::new(std::sync::Arc::new(net), node);

                info!(
                    "{} Turn motion {}",
                    style_node(node),
                    if string_to_bool(&toggle).unwrap() {
                        Green.paint("on")
                    } else {
                        Red.paint("off")
                    },
                );

                if string_to_bool(&toggle).unwrap() {
                    service.lock().await;
                } else {
                    service.unlock().await;
                }
            }
            NodeCommand::Actuator { actuator, value } => {
                let node = node_address(address)?;

                let mut service = ActuatorService::new(std::sync::Arc::new(net), node);

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

                service
                    .actuator_control([(actuator.clone(), value.clone())].into())
                    .await;
            }
        },
        Command::Dump { pgn, node } => {
            let mut router = Router::new(std::sync::Arc::new(net));

            if let Some(pgn) = pgn {
                router.add_pgn_filter(pgn);
            }
            if let Some(node) = node.map(|s| node_address(s).unwrap()) {
                router.add_node_filter(node);
            }

            print_frames(router).await?;
        }
        Command::Analyze { pgn, node } => {
            let net = std::sync::Arc::new(net);

            let mut router = Router::new(net.clone());

            if let Some(pgn) = pgn {
                router.add_pgn_filter(pgn);
            }
            if let Some(node) = node.map(|s| node_address(s).unwrap()) {
                router.add_node_filter(node);
            }

            analyze_frames(net, router).await?;
        }
    }

    Ok(())
}
