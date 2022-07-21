use std::convert::TryInto;

use ansi_term::Colour::{Blue, Green, Purple, Red, White, Yellow};
use clap::Parser;
use glonax::net::ControlNet;
use glonax_j1939::decode;
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

fn string_to_bool(str: &String) -> bool {
    match str.to_lowercase().trim() {
        "yes" => true,
        "true" => true,
        "on" => true,
        "1" => true,
        "no" => false,
        "false" => false,
        "off" => false,
        "0" => false,
        _ => false,
    }
}

/// Parameter group number.
pub enum ParameterGroupNumber {
    /// Electronic Engine Controller 1.
    EEC1,
    /// Electronic Engine Controller 2.
    EEC2,
    /// Software Identification.
    SOFT,
    /// Other PGN.
    Other(u16),
}

impl From<u16> for ParameterGroupNumber {
    fn from(value: u16) -> Self {
        match value {
            61_443 => ParameterGroupNumber::EEC2,
            61_444 => ParameterGroupNumber::EEC1,
            65_242 => ParameterGroupNumber::SOFT,
            _ => ParameterGroupNumber::Other(value),
        }
    }
}

async fn analyze_frames(net: &ControlNet, pgn_filter: Option<u16>) -> anyhow::Result<()> {
    debug!("Print incoming frames on screen");

    loop {
        let frame = loop {
            match tokio::time::timeout(std::time::Duration::from_millis(500), net.accept()).await {
                Ok(x) => break x?,
                Err(_) => net.announce_status().await,
            }
        };

        let pgn = frame.id().pgn();
        if let Some(pgn_filter) = pgn_filter {
            if pgn_filter != pgn {
                continue;
            }
        }

        if let Some(node_filter) = node_filter {
            if node_filter != frame.id().sa() {
                continue;
            }
        }

        match pgn.into() {
            ParameterGroupNumber::EEC1 => {
                if let Some(engine_torque_mode) = decode::spn899(frame.pdu()[0]) {
                    info!("Torque mode: {:?}", engine_torque_mode);
                }
                if let Some(driver_demand) = decode::spn512(frame.pdu()[1]) {
                    info!("Drivers Demand: {}%", driver_demand);
                }
                if let Some(actual_engine) = decode::spn513(frame.pdu()[2]) {
                    info!("Actual Engine: {}%", actual_engine);
                }
                if let Some(rpm) = decode::spn190(&frame.pdu()[3..5].try_into().unwrap()) {
                    info!("RPM: {}", rpm)
                }
                if let Some(source_addr) = decode::spn1483(frame.pdu()[5]) {
                    info!("Source Address: {:?}", source_addr);
                }
                if let Some(starter_mode) = decode::spn1675(frame.pdu()[6]) {
                    info!("Starter mode: {:?}", starter_mode);
                }
            }
            ParameterGroupNumber::SOFT => {
                let mut major = 0;
                let mut minor = 0;
                let mut patch = 0;

                if frame.pdu()[3] != 0xff {
                    major = frame.pdu()[3];
                }
                if frame.pdu()[4] != 0xff {
                    minor = frame.pdu()[4];
                }
                if frame.pdu()[5] != 0xff {
                    patch = frame.pdu()[5];
                }

                info!(
                    "{} Software identification: {}.{}.{}",
                    style_node(frame.id().sa()),
                    major,
                    minor,
                    patch
                );
            }
            ParameterGroupNumber::Other(65_282) => {
                // TODO: Change to new state
                let state = match frame.pdu()[1] {
                    1 => Yellow.paint("boot0").to_string(),
                    5 => Yellow.paint("init core peripherals").to_string(),
                    6 => Yellow.paint("init auxiliary modules").to_string(),
                    20 => Green.paint("nominal").to_string(),
                    255 => Red.paint("faulty").to_string(),
                    _ => White.paint("other").to_string(),
                };

                let firmware_version =
                    glonax::net::spn_firmware_version(frame.pdu()[2..5].try_into().unwrap());

                let last_error = glonax::net::spn_last_error(frame.pdu()[6..8].try_into().unwrap());

                info!(
                    "{} State: {}; Version: {}; Last error: {}",
                    style_node(frame.id().sa()),
                    state,
                    firmware_version.map_or_else(
                        || "-".to_owned(),
                        |f| { format!("{}.{}.{}", f.0, f.1, f.2) }
                    ),
                    last_error.map_or_else(|| "-".to_owned(), |f| { f.to_string() })
                );
            }
            ParameterGroupNumber::Other(64_258) => {
                // if frame.pdu()[..4] != [0xff; 4] {
                let data_x = u32::from_le_bytes(frame.pdu()[..4].try_into().unwrap());
                // let data_y = i16::from_le_bytes(frame.pdu()[2..4].try_into().unwrap());
                // let data_z = i16::from_le_bytes(frame.pdu()[4..6].try_into().unwrap());

                // let vec_x = data_x as f32;
                // let vec_y = data_y as f32;
                // let vec_z = data_z as f32;
                info!("data: {}", data_x);
                // let signal_angle = vec_x.atan2(-vec_y);
                // debug!("XY Angle: {:>+5.2}", signal_angle);

                // let fk_x = (6.0 * 0.349066_f32.cos()) + (2.97 * signal_angle.cos());
                // let fk_y = (6.0 * 0.349066_f32.sin()) + (2.97 * signal_angle.sin()); // + super::FRAME_HEIGHT;

                // let fk_x = 2.97 * signal_angle.cos();
                // let fk_y = 2.97 * signal_angle.sin();

                // info!(
                //     "{} X: {:>+5} Y: {:>+5} Z: {:>+5}    Angle: {:>+5.2}    {:>+5.2} {:>+5.2}",
                //     style_node(frame.id().sa()),
                //     data_x,
                //     data_y,
                //     data_z,
                //     signal_angle,
                //     fk_x,
                //     fk_y,
                // );
                // }
            }
            // 65_505 => {
            // if frame.pdu()[..6] != [0xff; 6] {
            //     let data_x = i16::from_le_bytes(frame.pdu()[..2].try_into().unwrap());
            //     let data_y = i16::from_le_bytes(frame.pdu()[2..4].try_into().unwrap());
            //     let data_z = i16::from_le_bytes(frame.pdu()[4..6].try_into().unwrap());

            //     let vec_x = data_x as f32;
            //     let vec_y = data_y as f32;
            //     let vec_z = data_z as f32;

            //     let signal_angle = vec_x.atan2(-vec_y);
            //     debug!("XY Angle: {:>+5.2}", signal_angle);

            //     let fk_x = (6.0 * 0.349066_f32.cos()) + (2.97 * signal_angle.cos());
            //     let fk_y = (6.0 * 0.349066_f32.sin()) + (2.97 * signal_angle.sin());
            // + super::FRAME_HEIGHT;

            // let fk_x = 2.97 * signal_angle.cos();
            // let fk_y = 2.97 * signal_angle.sin();

            // info!(
            //     "{} X: {:>+5} Y: {:>+5} Z: {:>+5}    Angle: {:>+5.2}    {:>+5.2} {:>+5.2}",
            //     style_node(frame.id().sa()),
            //     data_x,
            //     data_y,
            //     data_z,
            //     signal_angle,
            //     fk_x,
            //     fk_y,
            // );
            //     }
            // }
            // 65_515 => {
            // if frame.pdu()[..6] != [0xff; 6] {
            // let data_x = i16::from_le_bytes(frame.pdu()[..2].try_into().unwrap());
            // let data_y = i16::from_le_bytes(frame.pdu()[2..4].try_into().unwrap());
            // let data_z = i16::from_le_bytes(frame.pdu()[4..6].try_into().unwrap());

            // let vec_x = data_x as f32;
            // let vec_y = data_y as f32;
            // let vec_z = data_z as f32;

            // let signal_angle = vec_x.atan2(vec_y);

            // debug!("XY Angle: {:>+5.2}", signal_angle);
            // let heading = signal_angle * 180.0 / std::f32::consts::PI;

            // let heading = if heading < 0.0 {
            //     heading + 360.0
            // } else {
            //     heading - 360.0
            // };
            // // if (heading > 360) heading -= 360;
            // let heading = -heading;

            // info!(
            //     "{} X: {:>+5} Y: {:>+5} Z: {:>+5}    Angle: {:>+5.2}  Heading: {:>+5.2}",
            //     style_node(frame.id().sa()),
            //     data_x,
            //     data_y,
            //     data_z,
            //     signal_angle,
            //     heading
            // );
            //     }
            // }
            // 65_535 => {
            //     if frame.pdu()[..2] != [0xff, 0xff] {
            //         let data = u16::from_le_bytes(frame.pdu()[..2].try_into().unwrap());

            //         info!("{} Encoder 0: {}", style_node(frame.id().sa()), data,);
            //     }
            //     if frame.pdu()[2..4] != [0xff, 0xff] {
            //         let data = u16::from_le_bytes(frame.pdu()[2..4].try_into().unwrap());

            //         info!("{} Encoder 1: {}", style_node(frame.id().sa()), data,);
            //     }
            // }
            _ => {}
        }
    }
}

/// Print frames to screen.
async fn print_frames(net: &ControlNet) -> anyhow::Result<()> {
    debug!("Print incoming frames on screen");

    loop {
        let frame = net.accept().await?;

        info!("{}", frame);
    }
}

#[derive(Parser)]
#[clap(author = "Copyright (C) 2022 Laixer Equipment B.V.")]
#[clap(version)]
#[clap(about = "Network diagnosis and system analyzer", long_about = None)]
struct Args {
    /// CAN network interface.
    #[clap(short, long, default_value = "can0")]
    interface: String,

    /// Local network address.
    #[clap(long, default_value_t = 0x9e)]
    address: u8,

    /// Level of verbosity.
    #[clap(short, long, parse(from_occurrences))]
    verbose: usize,

    /// Node commands.
    #[clap(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Target node.
    Node {
        /// Target node address.
        address: String,

        #[clap(subcommand)]
        command: NodeCommand,
    },
    /// Show raw frames on screen.
    Dump,
    /// Analyze network frames.
    Analyze {
        /// Filter on PGN.
        #[clap(long)]
        pgn: Option<u16>,

        /// Filter on node.
        #[clap(long)]
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
    /// Enable or disable encoders.
    Encoder { encoder: u8, encoder_on: u8 },
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

    debug!("Binding to interface {}", args.interface);

    let net = ControlNet::new(args.interface.as_str(), args.address)?;

    match args.command {
        Command::Node { address, command } => match command {
            NodeCommand::Led { toggle } => {
                let node = node_address(address)?;

                info!(
                    "{} Turn identification LED {}",
                    style_node(node),
                    if string_to_bool(&toggle) {
                        Green.paint("on")
                    } else {
                        Red.paint("off")
                    },
                );

                net.set_led(node, string_to_bool(&toggle)).await;
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

                info!(
                    "{} Turn motion {}",
                    style_node(node),
                    if string_to_bool(&toggle) {
                        Green.paint("on")
                    } else {
                        Red.paint("off")
                    },
                );

                net.set_motion_lock(node, string_to_bool(&toggle)).await;
            }
            NodeCommand::Encoder {
                encoder,
                encoder_on,
            } => {
                let node = node_address(address)?;

                info!(
                    "{} Turn encoder {} {}",
                    style_node(node),
                    encoder,
                    if encoder_on == 0 {
                        Red.paint("off")
                    } else {
                        Green.paint("on")
                    },
                );

                net.enable_encoder(node, encoder, encoder_on == 1).await;
            }
            NodeCommand::Actuator { actuator, value } => {
                let node = node_address(address)?;

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

                net.actuator_control(node, [(actuator.clone(), value.clone())].into())
                    .await;
            }
        },
        Command::Dump => {
            print_frames(&net).await?;
        }
        Command::Analyze { pgn } => {
            analyze_frames(&net, pgn).await?;
        }
    }

    Ok(())
}
