use std::convert::TryInto;

use ansi_term::Colour::{Green, Purple, Red, White, Yellow};
use clap::Parser;
use glonax_j1939::j1939::decode;
use log::{debug, info};

fn style_node(address: u8) -> String {
    Purple.paint(format!("[node 0x{:X?}]", address)).to_string()
}

fn node_address(address: &String) -> Result<u8, std::num::ParseIntError> {
    if address.starts_with("0x") {
        u8::from_str_radix(address.trim_start_matches("0x"), 16)
    } else {
        u8::from_str_radix(address, 16)
    }
}

async fn analyze_frames(net: &glonax::net::ControlNet) -> anyhow::Result<()> {
    debug!("Print incoming frames on screen");

    loop {
        let frame = net.accept().await;

        match frame.id().pgn() {
            61_444 => {
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
            65_242 => {
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
            65_282 => {
                let state = match frame.pdu()[1] {
                    1 => Yellow.paint("boot0").to_string(),
                    5 => Yellow.paint("init core peripherals").to_string(),
                    6 => Yellow.paint("init auxiliary modules").to_string(),
                    20 => Green.paint("nominal").to_string(),
                    255 => Red.paint("faulty").to_string(),
                    _ => White.paint("other").to_string(),
                };

                info!(
                    "{} State: {}; Last error: {}",
                    style_node(frame.id().sa()),
                    state,
                    u16::from_le_bytes(frame.pdu()[6..8].try_into().unwrap())
                );
            }
            _ => {}
        }
    }
}

/// Print frames to screen.
async fn print_frames(net: &glonax::net::ControlNet) -> anyhow::Result<()> {
    debug!("Print incoming frames on screen");

    loop {
        let frame = net.accept().await;

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
    Analyze,
}

#[derive(clap::Subcommand)]
enum NodeCommand {
    /// Enable or disable identification LED.
    LED { toggle: u8 },
    /// Assign the node a new address.
    Assign { address_new: String },
    /// Reset the node.
    Reset,
    /// Report node status.
    Status,
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

    let net = glonax::net::ControlNet::open(args.interface.as_str(), args.address);

    match &args.command {
        Command::Node { address, command } => match command {
            NodeCommand::LED { toggle } => {
                let address_id = node_address(address)?;

                info!(
                    "{} Turn identification LED {}",
                    style_node(address_id),
                    if toggle == &0 {
                        Red.paint("off")
                    } else {
                        Green.paint("on")
                    },
                );

                net.set_led(address_id, toggle == &1).await;
            }
            NodeCommand::Assign { address_new } => {
                let address_id = node_address(address)?;
                let address_new_id = node_address(address_new)?;

                info!("{} Assign 0x{:X?}", style_node(address_id), address_new_id);

                net.set_address(address_id, address_new_id).await;
            }
            NodeCommand::Reset => {
                let address_id = node_address(address)?;

                info!("{} Reset", style_node(address_id));

                net.reset(address_id).await;
            }
            NodeCommand::Status => {
                let address_id = node_address(address)?;

                net.set_led(address_id, true).await;

                let found = false;
                // for _ in 0..3 {
                //     net.request(address_id, 0x18feda00).await;

                //     let frame = net.as_ref().recv_from().await?;

                //     if frame.id().pgn() == 65_242 {
                //         // let mut major = 0;
                //         // let mut minor = 0;
                //         // let mut patch = 0;

                //         // if frame.pdu()[3] != 0xff {
                //         //     major = frame.pdu()[3];
                //         // }
                //         // if frame.pdu()[4] != 0xff {
                //         //     minor = frame.pdu()[4];
                //         // }
                //         // if frame.pdu()[5] != 0xff {
                //         //     patch = frame.pdu()[5];
                //         // }

                //         found = true;
                //         break;
                //     }
                // }

                if found {
                    info!(
                        "{} Reports {} version {}",
                        style_node(address_id),
                        Green.paint("alive"),
                        White.paint(format!("{}.{}.{}", 0, 2, 2))
                    );
                } else {
                    info!("{} Node is {}", style_node(address_id), Red.paint("down"));
                }

                net.set_led(address_id, false).await;
            }
        },
        Command::Dump => {
            print_frames(&net).await?;
        }
        Command::Analyze => {
            analyze_frames(&net).await?;
        }
    }

    Ok(())
}
