use std::convert::TryInto;

use clap::Parser;
use glonax_j1939::{j1939::decode, J1939Socket};
use log::info;

async fn analyze_frames(socket: &J1939Socket) -> anyhow::Result<()> {
    use ansi_term::Colour::{Green, Red, White, Yellow};

    info!("Print incoming frames on screen");

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
                    "[device {}] Software identification: {}.{}.{}",
                    White.paint(format!("0x{:X?}", frame.id().sa())),
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
                    "[device {}] State: {}; Last error: {}",
                    White.paint(format!("0x{:X?}", frame.id().sa())),
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
    info!("Print incoming frames on screen");

    loop {
        let frame = socket.recv_from().await?;

        info!("{}", frame);
    }
}

#[derive(Parser)]
#[clap(name = "netshock")]
#[clap(author = "Copyright (C) 2022 Laixer Equipment B.V.")]
#[clap(version)]
#[clap(about = "Network diagnosis and system analyzer", long_about = None)]
struct Args {
    /// CAN network interface.
    // #[clap(short, long)]
    interface: String,

    /// Local network address.
    #[clap(long, default_value_t = 0x9e)]
    address: u8,

    /// Show raw frames on screen.
    #[clap(short, long)]
    dump: bool,

    /// Level of verbosity.
    #[clap(short, long, parse(from_occurrences))]
    verbose: usize,
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

    info!("Binding to interface {}", args.interface);

    let socket = glonax_j1939::J1939Socket::bind(args.interface.as_str(), args.address)?;
    socket.set_broadcast(true)?;

    if args.dump {
        print_frames(&socket).await?;
    } else {
        analyze_frames(&socket).await?;
    }

    Ok(())
}
