use std::convert::TryInto;

use ansi_term::Colour::{Green, Purple, Red, White, Yellow};
use clap::Parser;
use glonax_j1939::{j1939::decode, J1939Socket};
use log::{debug, info};

async fn analyze_frames(socket: &J1939Socket) -> anyhow::Result<()> {
    debug!("Print incoming frames on screen");

    loop {
        let frame = socket.recv_from().await?;

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
                    Purple.paint(format!("[node 0x{:X?}]", frame.id().sa())),
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
                    Purple.paint(format!("[node 0x{:X?}]", frame.id().sa())),
                    state,
                    u16::from_le_bytes(frame.pdu()[6..8].try_into().unwrap())
                );
            }
            _ => {}
        }
    }
}

/// Print frames to screen.
async fn print_frames(socket: &J1939Socket) -> anyhow::Result<()> {
    debug!("Print incoming frames on screen");

    loop {
        let frame = socket.recv_from().await?;

        info!("{}", frame);
    }
}

async fn _control(socket: &mut J1939Socket) -> anyhow::Result<()> {
    let mut pad = glonax_gamepad::Gamepad::new(std::path::Path::new("/dev/input/js0")).await?;

    loop {
        match pad.next_event().await {
            Ok(event) => match event {
                glonax_gamepad::Event {
                    ty: glonax_gamepad::EventType::Axis(glonax_gamepad::Axis::RightStickY),
                    ..
                } => {
                    println!("RightStickY {}", event.value());

                    let rpm_ba = &event.value().to_le_bytes()[..2];
                    let id = 0x18A04A00;
                    // let id2 = glonax_j1939::j1939::IdBuilder::from_pgn(0).da(0x0).build();
                    let frame = glonax_j1939::j1939::Frame::new(
                        glonax_j1939::j1939::Id::new(id),
                        [0x00, 0x00, 0x00, 0x00, rpm_ba[0], rpm_ba[1], 0x00, 0x00],
                    );

                    socket.send_to(&frame).await?;
                }
                glonax_gamepad::Event {
                    ty: glonax_gamepad::EventType::Axis(glonax_gamepad::Axis::RightStickX),
                    ..
                } => {
                    println!("RightStickX {}", event.value());

                    let rpm_ba = &event.value().to_le_bytes()[..2];
                    let id = 0x18A14A00;
                    // let id2 = glonax_j1939::j1939::IdBuilder::from_pgn(0).da(0x0).build();
                    let frame = glonax_j1939::j1939::Frame::new(
                        glonax_j1939::j1939::Id::new(id),
                        [0x00, 0x00, 0x00, 0x00, rpm_ba[0], rpm_ba[1], 0x00, 0x00],
                    );

                    socket.send_to(&frame).await?;
                }
                glonax_gamepad::Event {
                    ty: glonax_gamepad::EventType::Axis(glonax_gamepad::Axis::LeftStickY),
                    ..
                } => {
                    println!("RightStickX {}", event.value());

                    let rpm_ba = &event.value().to_le_bytes()[..2];
                    let id = 0x18A14A00;
                    // let id2 = glonax_j1939::j1939::IdBuilder::from_pgn(0).da(0x0).build();
                    let frame = glonax_j1939::j1939::Frame::new(
                        glonax_j1939::j1939::Id::new(id),
                        [0x00, 0x00, 0x00, 0x00, 0x0, 0x0, rpm_ba[0], rpm_ba[1]],
                    );

                    socket.send_to(&frame).await?;
                }
                _ => {}
            },
            Err(_) => {}
        }
    }
}

async fn report_node(socket: &mut J1939Socket, target_node: u8) -> anyhow::Result<()> {
    Ok(())
}

#[derive(Parser)]
#[clap(name = "netshock")]
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

    /// Destination target.
    #[clap(short, long)]
    target: Option<u8>,

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
        address: u8,
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
    Assign { address_new: u8 },
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

    let socket = glonax_j1939::J1939Socket::bind(args.interface.as_str(), args.address)?;
    socket.set_broadcast(true)?;

    match &args.command {
        Command::Node { address, command } => match command {
            NodeCommand::LED { toggle } => {
                info!(
                    "{} Turn identification LED {}",
                    Purple.paint(format!("[node 0x{:X?}]", address)),
                    if toggle == &0 {
                        Red.paint("off")
                    } else {
                        Green.paint("on")
                    },
                );

                let frame = glonax_j1939::j1939::Frame::new(
                    glonax_j1939::j1939::IdBuilder::from_pgn(45_312)
                        .da(*address)
                        .build(),
                    [
                        b'Z',
                        b'C',
                        if toggle == &0 { 0x0 } else { 0x1 },
                        0xff,
                        0xff,
                        0xff,
                        0xff,
                        0xff,
                    ],
                );

                socket.send_to(&frame).await?;
            }
            NodeCommand::Assign { address_new } => {
                info!(
                    "{} Assign 0x{:X?}",
                    Purple.paint(format!("[node 0x{:X?}]", address)),
                    *address_new
                );

                let frame = glonax_j1939::j1939::Frame::new(
                    glonax_j1939::j1939::IdBuilder::from_pgn(45_568)
                        .da(*address)
                        .build(),
                    [b'Z', b'C', *address_new, 0xff, 0xff, 0xff, 0xff, 0xff],
                );

                socket.send_to(&frame).await?;
            }
            NodeCommand::Reset => {
                info!("{} Reset", Purple.paint(format!("[node 0x{:X?}]", address)));

                let frame = glonax_j1939::j1939::Frame::new(
                    glonax_j1939::j1939::IdBuilder::from_pgn(45_312)
                        .da(*address)
                        .build(),
                    [b'Z', b'C', 0xff, 0x69, 0xff, 0xff, 0xff, 0xff],
                );

                socket.send_to(&frame).await?;
            }
            NodeCommand::Status => {
                let frame = glonax_j1939::j1939::Frame::new(
                    glonax_j1939::j1939::IdBuilder::from_pgn(45_312)
                        .da(*address)
                        .build(),
                    [b'Z', b'C', 0x1, 0xff, 0xff, 0xff, 0xff, 0xff],
                );

                socket.send_to(&frame).await?;

                //

                loop {
                    let frame = glonax_j1939::j1939::Frame::new(
                        glonax_j1939::j1939::IdBuilder::from_pgn(59_904)
                            .da(*address)
                            .build(),
                        [0xfe, 0x18, 0xda, 0xff, 0xff, 0xff, 0xff, 0xff],
                    );

                    socket.send_to(&frame).await?;
                    // 18EA7B00 # FE 18 DA # Software Identification

                    //

                    let frame = socket.recv_from().await?;

                    if frame.id().pgn() == 65_242 {
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
                            "{} Reports {} version {}",
                            Purple.paint(format!("[node 0x{:X?}]", address)),
                            Green.paint("alive"),
                            White.paint(format!("{}.{}.{}", major, minor, patch))
                        );

                        break;
                    }
                }

                //

                let frame = glonax_j1939::j1939::Frame::new(
                    glonax_j1939::j1939::IdBuilder::from_pgn(45_312)
                        .da(*address)
                        .build(),
                    [b'Z', b'C', 0x0, 0xff, 0xff, 0xff, 0xff, 0xff],
                );

                socket.send_to(&frame).await?;
            }
        },
        Command::Dump => {
            print_frames(&socket).await?;
        }
        Command::Analyze => {
            analyze_frames(&socket).await?;
        }
    }

    Ok(())
}
