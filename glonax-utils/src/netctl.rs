// Copyright (C) 2024 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use ansi_term::Colour::{Blue, Green, Purple, Red, Yellow};
use clap::Parser;
use glonax::net::*;

use log::{debug, info};

pub(crate) mod consts {
    /// On-Board Data Logger J1939 address.
    pub const J1939_ADDRESS_OBDL: u8 = 0xFB;
    /// Volvo VECU J1939 address.
    pub const J1939_ADDRESS_VOLVO_VECU: u8 = 0x11;
    /// Engine J1939 address.
    pub const J1939_ADDRESS_ENGINE0: u8 = 0x0;
    /// Hydraulic Control Unit J1939 address.
    pub const J1939_ADDRESS_HCU0: u8 = 0x4A;
    /// Vehicle Control Unit J1939 address.
    pub const J1939_ADDRESS_VCU0: u8 = 0x12;
    /// Kuebler Encoder 0 J1939 address.
    pub const J1939_ADDRESS_ENCODER0: u8 = 0x6A;
    /// Kuebler Encoder 1 J1939 address.
    pub const J1939_ADDRESS_ENCODER1: u8 = 0x6B;
    /// Kuebler Encoder 2 J1939 address.
    pub const J1939_ADDRESS_ENCODER2: u8 = 0x6C;
    /// Kuebler Encoder 3 J1939 address.
    pub const J1939_ADDRESS_ENCODER3: u8 = 0x6D;
    /// Kuebler Inclinometer 0 J1939 address.
    pub const J1939_ADDRESS_IMU0: u8 = 0x7A;

    /// J1939 name manufacturer code.
    pub const J1939_NAME_MANUFACTURER_CODE: u16 = 0x717;
    /// J1939 name function instance.
    pub const J1939_NAME_FUNCTION_INSTANCE: u8 = 6;
    /// J1939 name ECU instance.
    pub const J1939_NAME_ECU_INSTANCE: u8 = 0;
    /// J1939 name function.
    pub const J1939_NAME_FUNCTION: u8 = 0x1C;
    /// J1939 name vehicle system.
    pub const J1939_NAME_VEHICLE_SYSTEM: u8 = 2;
}

fn style_address(address: u8) -> String {
    Purple.paint(format!("[0x{:X?}]", address)).to_string()
}

// TODO: Move to j1939 crate if possible
fn j1939_address(address: String) -> Result<u8, std::num::ParseIntError> {
    if address.starts_with("0x") {
        u8::from_str_radix(address.trim_start_matches("0x"), 16)
    } else {
        u8::from_str_radix(address.as_str(), 16)
    }
}

struct Interval {
    interval: Option<tokio::time::Interval>,
}

impl Interval {
    fn new(interval: u64) -> Self {
        if interval == 0 {
            Self { interval: None }
        } else {
            Self {
                interval: Some(tokio::time::interval(std::time::Duration::from_millis(
                    interval,
                ))),
            }
        }
    }

    async fn tick(&mut self) {
        if let Some(interval) = &mut self.interval {
            interval.tick().await;
        }
    }
}

/// Analyze incoming frames and print their contents to the screen.
async fn analyze_frames(mut network: ControlNetwork) -> anyhow::Result<()> {
    use glonax::driver::{
        HydraulicControlUnit, J1939ApplicationInspector, J1939Message, KueblerEncoder,
        KueblerInclinometer, VehicleControlUnit, VolvoD7E,
    };

    debug!("Print incoming frames to screen");

    let mut ems0 = VolvoD7E::new(
        network.interface(),
        consts::J1939_ADDRESS_ENGINE0,
        consts::J1939_ADDRESS_OBDL,
    );
    let mut enc0 = KueblerEncoder::new(
        network.interface(),
        consts::J1939_ADDRESS_ENCODER0,
        consts::J1939_ADDRESS_OBDL,
    );
    let mut enc1 = KueblerEncoder::new(
        network.interface(),
        consts::J1939_ADDRESS_ENCODER1,
        consts::J1939_ADDRESS_OBDL,
    );
    let mut enc2 = KueblerEncoder::new(
        network.interface(),
        consts::J1939_ADDRESS_ENCODER2,
        consts::J1939_ADDRESS_OBDL,
    );
    let mut enc3 = KueblerEncoder::new(
        network.interface(),
        consts::J1939_ADDRESS_ENCODER3,
        consts::J1939_ADDRESS_OBDL,
    );
    let mut imu0 = KueblerInclinometer::new(
        network.interface(),
        consts::J1939_ADDRESS_IMU0,
        consts::J1939_ADDRESS_OBDL,
    );
    let mut hcu0 = HydraulicControlUnit::new(
        network.interface(),
        consts::J1939_ADDRESS_HCU0,
        consts::J1939_ADDRESS_OBDL,
    );
    let mut vcu0 = VehicleControlUnit::new(
        network.interface(),
        consts::J1939_ADDRESS_VCU0,
        consts::J1939_ADDRESS_OBDL,
    );
    let mut jis0 = J1939ApplicationInspector;

    loop {
        network.recv().await?;

        if let Some(message) = network.try_accept(&mut ems0) {
            match message {
                glonax::driver::EngineMessage::TorqueSpeedControl(control) => {
                    info!(
                        "{} {} {} » Torque speed control: {}",
                        chrono::Utc::now().format("%T%.3f"),
                        style_address(network.frame_source().unwrap()),
                        Yellow.bold().paint("Engine"),
                        control
                    );
                }
                glonax::driver::EngineMessage::BrakeController1(controller) => {
                    info!(
                        "{} {} {} » Brake controller: {}",
                        chrono::Utc::now().format("%T%.3f"),
                        style_address(network.frame_source().unwrap()),
                        Yellow.bold().paint("Engine"),
                        controller
                    );
                }
                glonax::driver::EngineMessage::EngineController1(controller) => {
                    info!(
                        "{} {} {} » Engine controller: {}",
                        chrono::Utc::now().format("%T%.3f"),
                        style_address(network.frame_source().unwrap()),
                        Yellow.bold().paint("Engine"),
                        controller
                    );
                }
                glonax::driver::EngineMessage::EngineController2(controller) => {
                    info!(
                        "{} {} {} » Engine controller: {}",
                        chrono::Utc::now().format("%T%.3f"),
                        style_address(network.frame_source().unwrap()),
                        Yellow.bold().paint("Engine"),
                        controller
                    );
                }
                glonax::driver::EngineMessage::EngineController3(controller) => {
                    info!(
                        "{} {} {} » Engine controller: {}",
                        chrono::Utc::now().format("%T%.3f"),
                        style_address(network.frame_source().unwrap()),
                        Yellow.bold().paint("Engine"),
                        controller
                    );
                }
                glonax::driver::EngineMessage::FanDrive(fan) => {
                    info!(
                        "{} {} {} » Fan drive: {}",
                        chrono::Utc::now().format("%T%.3f"),
                        style_address(network.frame_source().unwrap()),
                        Yellow.bold().paint("Engine"),
                        fan
                    );
                }
                glonax::driver::EngineMessage::VehicleDistance(distance) => {
                    info!(
                        "{} {} {} » Vehicle distance: {}",
                        chrono::Utc::now().format("%T%.3f"),
                        style_address(network.frame_source().unwrap()),
                        Yellow.bold().paint("Engine"),
                        distance
                    );
                }
                glonax::driver::EngineMessage::Shutdown(shutdown) => {
                    info!(
                        "{} {} {} » Shutdown: {}",
                        chrono::Utc::now().format("%T%.3f"),
                        style_address(network.frame_source().unwrap()),
                        Yellow.bold().paint("Engine"),
                        shutdown
                    );
                }
                glonax::driver::EngineMessage::EngineTemperature1(temperature) => {
                    info!(
                        "{} {} {} » Engine temperature: {}",
                        chrono::Utc::now().format("%T%.3f"),
                        style_address(network.frame_source().unwrap()),
                        Yellow.bold().paint("Engine"),
                        temperature
                    );
                }
                glonax::driver::EngineMessage::EngineFluidLevelPressure1(fluid) => {
                    info!(
                        "{} {} {} » Engine fluid level pressure: {}",
                        chrono::Utc::now().format("%T%.3f"),
                        style_address(network.frame_source().unwrap()),
                        Yellow.bold().paint("Engine"),
                        fluid
                    );
                }
                glonax::driver::EngineMessage::EngineFluidLevelPressure2(fluid) => {
                    info!(
                        "{} {} {} » Engine fluid level pressure: {}",
                        chrono::Utc::now().format("%T%.3f"),
                        style_address(network.frame_source().unwrap()),
                        Yellow.bold().paint("Engine"),
                        fluid
                    );
                }
                glonax::driver::EngineMessage::FuelEconomy(economy) => {
                    info!(
                        "{} {} {} » Fuel economy: {}",
                        chrono::Utc::now().format("%T%.3f"),
                        style_address(network.frame_source().unwrap()),
                        Yellow.bold().paint("Engine"),
                        economy
                    );
                }
                glonax::driver::EngineMessage::FuelConsumption(consumption) => {
                    info!(
                        "{} {} {} » Fuel consumption: {}",
                        chrono::Utc::now().format("%T%.3f"),
                        style_address(network.frame_source().unwrap()),
                        Yellow.bold().paint("Engine"),
                        consumption
                    );
                }
                glonax::driver::EngineMessage::AmbientConditions(conditions) => {
                    info!(
                        "{} {} {} » Ambient conditions: {}",
                        chrono::Utc::now().format("%T%.3f"),
                        style_address(network.frame_source().unwrap()),
                        Yellow.bold().paint("Engine"),
                        conditions
                    );
                }
                glonax::driver::EngineMessage::PowerTakeoffInformation(info) => {
                    info!(
                        "{} {} {} » Power takeoff information: {}",
                        chrono::Utc::now().format("%T%.3f"),
                        style_address(network.frame_source().unwrap()),
                        Yellow.bold().paint("Engine"),
                        info
                    );
                }
                glonax::driver::EngineMessage::TankInformation1(info) => {
                    info!(
                        "{} {} {} » Tank information: {}",
                        chrono::Utc::now().format("%T%.3f"),
                        style_address(network.frame_source().unwrap()),
                        Yellow.bold().paint("Engine"),
                        info
                    );
                }
                glonax::driver::EngineMessage::VehicleElectricalPower(power) => {
                    info!(
                        "{} {} {} » Vehicle electrical power: {}",
                        chrono::Utc::now().format("%T%.3f"),
                        style_address(network.frame_source().unwrap()),
                        Yellow.bold().paint("Engine"),
                        power
                    );
                }
                glonax::driver::EngineMessage::InletExhaustConditions1(conditions) => {
                    info!(
                        "{} {} {} » Inlet exhaust conditions: {}",
                        chrono::Utc::now().format("%T%.3f"),
                        style_address(network.frame_source().unwrap()),
                        Yellow.bold().paint("Engine"),
                        conditions
                    );
                }
            }
        } else if let Some(message) = network.try_accept(&mut enc2) {
            if let glonax::driver::net::encoder::EncoderMessage::ProcessData(data) = message {
                info!(
                    "{} {} {} » {}",
                    chrono::Utc::now().format("%T%.3f"),
                    style_address(network.frame_source().unwrap()),
                    Yellow.bold().paint("Arm"),
                    data
                );
            }
        } else if let Some(message) = network.try_accept(&mut enc1) {
            if let glonax::driver::net::encoder::EncoderMessage::ProcessData(data) = message {
                info!(
                    "{} {} {} » {}",
                    chrono::Utc::now().format("%T%.3f"),
                    style_address(network.frame_source().unwrap()),
                    Yellow.bold().paint("Boom"),
                    data
                );
            }
        } else if let Some(message) = network.try_accept(&mut enc0) {
            if let glonax::driver::net::encoder::EncoderMessage::ProcessData(data) = message {
                info!(
                    "{} {} {} » {}",
                    chrono::Utc::now().format("%T%.3f"),
                    style_address(network.frame_source().unwrap()),
                    Yellow.bold().paint("Frame"),
                    data
                );
            }
        } else if let Some(message) = network.try_accept(&mut enc3) {
            if let glonax::driver::net::encoder::EncoderMessage::ProcessData(data) = message {
                info!(
                    "{} {} {} » {}",
                    chrono::Utc::now().format("%T%.3f"),
                    style_address(network.frame_source().unwrap()),
                    Yellow.bold().paint("Attachment"),
                    data
                );
            }
        } else if let Some(message) = network.try_accept(&mut imu0) {
            if let glonax::driver::net::inclino::InclinoMessage::ProcessData(data) = message {
                info!(
                    "{} {} {} » {}",
                    chrono::Utc::now().format("%T%.3f"),
                    style_address(network.frame_source().unwrap()),
                    Yellow.bold().paint("Inclinometer"),
                    data
                );
            }
        } else if let Some(message) = network.try_accept(&mut hcu0) {
            match message {
                glonax::driver::net::hydraulic::HydraulicMessage::Actuator(actuator) => {
                    info!(
                        "{} {} {} » Actuator: {}",
                        chrono::Utc::now().format("%T%.3f"),
                        style_address(network.frame_source().unwrap()),
                        Yellow.bold().paint("Hydraulic"),
                        actuator
                    );
                }
                glonax::driver::net::hydraulic::HydraulicMessage::MotionConfig(motion) => {
                    info!(
                        "{} {} {} » Motion config: {}",
                        chrono::Utc::now().format("%T%.3f"),
                        style_address(network.frame_source().unwrap()),
                        Yellow.bold().paint("Hydraulic"),
                        motion
                    );
                }
                glonax::driver::net::hydraulic::HydraulicMessage::VecraftConfig(config) => {
                    info!(
                        "{} {} {} » Vecraft config: {}",
                        chrono::Utc::now().format("%T%.3f"),
                        style_address(network.frame_source().unwrap()),
                        Yellow.bold().paint("Hydraulic"),
                        config
                    );
                }
                glonax::driver::net::hydraulic::HydraulicMessage::Status(status) => {
                    info!(
                        "{} {} {} » Status: {}",
                        chrono::Utc::now().format("%T%.3f"),
                        style_address(network.frame_source().unwrap()),
                        Yellow.bold().paint("Hydraulic"),
                        status
                    );
                }
                _ => {}
            }
        } else if let Some(message) = network.try_accept(&mut vcu0) {
            match message {
                glonax::driver::net::vcu::VehicleMessage::VecraftConfig(config) => {
                    info!(
                        "{} {} {} » Vecraft config: {}",
                        chrono::Utc::now().format("%T%.3f"),
                        style_address(network.frame_source().unwrap()),
                        Yellow.bold().paint("Vehicle"),
                        config
                    );
                }
                glonax::driver::net::vcu::VehicleMessage::Status(status) => {
                    info!(
                        "{} {} {} » Status: {}",
                        chrono::Utc::now().format("%T%.3f"),
                        style_address(network.frame_source().unwrap()),
                        Yellow.bold().paint("Vehicle"),
                        status
                    );
                }
                _ => {}
            }
        } else if let Some(message) = network.try_accept(&mut jis0) {
            match message {
                J1939Message::SoftwareIndent((major, minor, patch)) => {
                    info!(
                        "{} {} {} » Software identification: {}.{}.{}",
                        chrono::Utc::now().format("%T%.3f"),
                        style_address(network.frame_source().unwrap()),
                        Yellow.bold().paint("J1939"),
                        major,
                        minor,
                        patch
                    );
                }
                J1939Message::RequestPGN(pgn) => {
                    info!(
                        "{} {} {} » Request for PGN: {:?}",
                        chrono::Utc::now().format("%T%.3f"),
                        style_address(network.frame_source().unwrap()),
                        Yellow.bold().paint("J1939"),
                        pgn
                    );
                }
                J1939Message::AddressClaim(name) => {
                    info!(
                        "{} {} {} » Name: {}",
                        chrono::Utc::now().format("%T%.3f"),
                        style_address(network.frame_source().unwrap()),
                        Yellow.bold().paint("J1939"),
                        name
                    );
                }
                J1939Message::Acknowledged(acknowledged) => {
                    info!(
                        "{} {} {} » Acknowledged: {}",
                        chrono::Utc::now().format("%T%.3f"),
                        style_address(network.frame_source().unwrap()),
                        Yellow.bold().paint("J1939"),
                        acknowledged
                    );
                }
                J1939Message::TimeDate(time) => {
                    info!(
                        "{} {} {} » Time and date: {}",
                        chrono::Utc::now().format("%T%.3f"),
                        style_address(network.frame_source().unwrap()),
                        Yellow.bold().paint("J1939"),
                        time
                    );
                }
                J1939Message::ActiveDiagnosticTroubleCodes(diagnostic) => {
                    info!(
                        "{} {} {} » Active diagnostic trouble codes: SPN: {} FMI {}",
                        chrono::Utc::now().format("%T%.3f"),
                        style_address(network.frame_source().unwrap()),
                        Yellow.bold().paint("J1939"),
                        diagnostic.suspect_parameter_number,
                        diagnostic.failure_mode_identifier
                    );
                }
                J1939Message::ProprietaryB(data) => {
                    debug!(
                        "{} {} {} » Proprietary B: {:02X?}",
                        chrono::Utc::now().format("%T%.3f"),
                        style_address(network.frame_source().unwrap()),
                        Yellow.bold().paint("J1939"),
                        data
                    );
                }
            }
        }
    }
}

/// Print raw frames to standard output.
async fn print_frames(socket: CANSocket, filter: Filter) -> anyhow::Result<()> {
    debug!("Print incoming frames to screen");

    let mut rx_last = std::time::Instant::now();

    loop {
        let frame = socket.recv().await?;
        if filter.matches(frame.id()) {
            // TODO: Move to j1939 crate
            let specification_part = match frame.id().pgn() {
                glonax::j1939::PGN::ProprietaryA => "PA",
                glonax::j1939::PGN::ProprietaryB(_) => "PB",
                glonax::j1939::PGN::Other(_) => "OT",
                _ => "71",
            };

            println!(
                "{} {:4}ms {} {}",
                chrono::Utc::now().format("%T%.3f"),
                rx_last.elapsed().as_millis(),
                specification_part,
                frame
            );

            rx_last = std::time::Instant::now();
        };
    }
}

async fn diagnose(mut network: ControlNetwork) -> anyhow::Result<()> {
    debug!("Print incoming frames to screen");

    let mut jis0 = glonax::driver::J1939ApplicationInspector;
    let mut prb0 = glonax::driver::net::probe::Probe::default();

    loop {
        network.recv().await?;

        if let Some(message) = network.try_accept(&mut jis0) {
            match message {
                glonax::driver::J1939Message::AddressClaim(name) => {
                    println!(
                        "{} Name: {}",
                        Purple.paint(format!("[0x{:X?}]", network.frame_source().unwrap())),
                        name
                    );
                }
                glonax::driver::J1939Message::SoftwareIndent((major, minor, patch)) => {
                    println!(
                        "{} Software identification: {}.{}.{}",
                        style_address(network.frame_source().unwrap()),
                        major,
                        minor,
                        patch
                    );
                }
                glonax::driver::J1939Message::TimeDate(time) => {
                    println!(
                        "{} Time and date: {}",
                        style_address(network.frame_source().unwrap()),
                        time
                    );
                }
                _ => {}
            }
        } else if let Some(message) = network.try_accept(&mut prb0) {
            if let Some(address) = message.destination_address {
                info!(
                    "{} {} Destination address: {}",
                    chrono::Utc::now().format("%T%.3f"),
                    style_address(network.frame_source().unwrap()),
                    style_address(address),
                );

                prb0.send_probe(address, consts::J1939_ADDRESS_OBDL, &mut network)
                    .await?;
            }
            if let Some(address) = message.source_address {
                info!(
                    "{} {} Source address: {}",
                    chrono::Utc::now().format("%T%.3f"),
                    style_address(network.frame_source().unwrap()),
                    style_address(address),
                );

                prb0.send_probe(address, consts::J1939_ADDRESS_OBDL, &mut network)
                    .await?;
            }
        }
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
    #[arg(long, default_value_t = consts::J1939_ADDRESS_OBDL)]
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
        /// Target address.
        #[arg(short, long, default_value = "0x4A")]
        address: String,
        /// HCU commands.
        #[command(subcommand)]
        command: HCUCommand,
    },
    /// Vecraft control unit commands.
    Vecraft {
        /// Target address.
        #[arg(short, long, default_value = "0x11")]
        address: String,
        /// VCU commands.
        #[command(subcommand)]
        command: VCUCommand,
    },
    /// Engine control unit commands.
    Engine {
        /// Engine driver.
        #[arg(short, long, default_value = "j1939")]
        driver: String,
        /// Target address.
        #[arg(short, long, default_value = "0x0")]
        address: String,
        /// Engine commands.
        #[command(subcommand)]
        command: EngineCommand,
    },
    /// Request data from a unit.
    #[clap(alias("req"))]
    Request {
        /// Target address.
        #[arg(short, long)]
        address: String,
        /// PGN to request.
        pgn: u32,
    },
    /// Send raw frames.
    Send {
        /// Message interval in milliseconds.
        #[arg(short, long, default_value_t = 10)]
        interval: u64,
        /// Frame ID.
        id: String,
        /// Raw data to send.
        data: String,
    },
    /// Broadcast raw frames.
    Broadcast {
        /// PGN to broadcast.
        pgn: u32,
        /// Raw data to send.
        data: String,
    },
    /// Frame fuzzer.
    Fuzzer {
        /// Message interval in milliseconds.
        #[arg(short, long)]
        interval: Option<u64>,
        /// Frame ID.
        id: String,
    },
    /// Diagnose network.
    #[clap(alias("diag"))]
    Diagnostic,
    /// Show raw frames on screen.
    Dump {
        /// Exclude matched frames.
        #[arg(short, long)]
        exclude: bool,
        /// Filter on PGN.
        #[arg(long)]
        pgn: Vec<u32>,
        /// Filter on priority.
        #[arg(long)]
        priority: Vec<u8>,
        /// Filter on address.
        #[arg(short, long)]
        address: Vec<String>,
    },
    /// Analyze network frames.
    Analyze {
        /// Exclude matched frames.
        #[arg(short, long)]
        exclude: bool,
        /// Filter on PGN.
        #[arg(long)]
        pgn: Vec<u32>,
        /// Filter on priority.
        #[arg(long)]
        priority: Vec<u8>,
        /// Filter on address.
        #[arg(short, long)]
        address: Vec<String>,
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
    /// Motion reset.
    MotionReset,
    /// Enable or disable motion lock.
    Lock { toggle: String },
    /// Actuator motion.
    Actuator { actuator: u8, value: i16 },
}

#[derive(clap::Subcommand)]
enum VCUCommand {
    /// Enable or disable identification mode.
    Ident { toggle: String },
    /// Assign the unit a new address.
    Assign { address_new: String },
    /// Reboot the unit.
    Reboot,
    /// Factory reset the unit.
    FactoryReset,
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
            let destination_address = j1939_address(address)?;
            let socket = CANSocket::bind(&SockAddrCAN::new(args.interface.as_str()))?;

            let hcu0 = glonax::driver::HydraulicControlUnit::new(
                args.interface.as_str(),
                destination_address,
                consts::J1939_ADDRESS_OBDL,
            );

            match command {
                HCUCommand::MotionReset => {
                    info!("{} Motion reset", style_address(destination_address));

                    socket.send(&hcu0.motion_reset()).await?;
                }
                HCUCommand::Lock { toggle } => {
                    info!(
                        "{} Turn lock {}",
                        style_address(destination_address),
                        if toggle.parse::<bool>()? {
                            Green.paint("on")
                        } else {
                            Red.paint("off")
                        },
                    );

                    let frames = if toggle.parse::<bool>()? {
                        hcu0.lock()
                    } else {
                        hcu0.unlock()
                    };

                    socket.send(&frames).await?;
                }
                HCUCommand::Actuator { actuator, value } => {
                    info!(
                        "{} Set actuator {} to {}",
                        style_address(destination_address),
                        actuator,
                        if value.is_positive() {
                            Blue.paint(value.to_string())
                        } else {
                            Green.paint(value.abs().to_string())
                        },
                    );

                    loop {
                        socket
                            .send_vectored(&hcu0.actuator_command([(actuator, value)].into()))
                            .await?;
                        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                    }
                }
            }
        }
        Command::Vecraft { address, command } => {
            let destination_address = j1939_address(address)?;
            let socket = CANSocket::bind(&SockAddrCAN::new(args.interface.as_str()))?;

            // TODO: Using HCU as Vecraft for now. Need to implement Vecraft driver.
            let hcu0 = glonax::driver::HydraulicControlUnit::new(
                args.interface.as_str(),
                destination_address,
                consts::J1939_ADDRESS_OBDL,
            );

            match command {
                VCUCommand::Ident { toggle } => {
                    info!(
                        "{} Turn identification mode {}",
                        style_address(destination_address),
                        if toggle.parse::<bool>()? {
                            Green.paint("on")
                        } else {
                            Red.paint("off")
                        },
                    );

                    socket
                        .send(&hcu0.set_ident(toggle.parse::<bool>()?))
                        .await?;
                }
                VCUCommand::Reboot => {
                    info!("{} Reboot", style_address(destination_address));

                    socket.send(&hcu0.reboot()).await?;
                }
                VCUCommand::Assign { address_new } => {
                    let destination_address_new = j1939_address(address_new)?;

                    let name = glonax::j1939::NameBuilder::default()
                        .identity_number(0x1)
                        .manufacturer_code(consts::J1939_NAME_MANUFACTURER_CODE)
                        .function_instance(consts::J1939_NAME_FUNCTION_INSTANCE)
                        .ecu_instance(consts::J1939_NAME_ECU_INSTANCE)
                        .function(consts::J1939_NAME_FUNCTION)
                        .vehicle_system(consts::J1939_NAME_VEHICLE_SYSTEM)
                        .build();

                    info!(
                        "{} Assign 0x{:X?}",
                        style_address(destination_address),
                        destination_address_new
                    );

                    socket
                        .send_vectored(
                            &glonax::j1939::protocol::commanded_address(
                                consts::J1939_ADDRESS_OBDL,
                                &name,
                                destination_address_new,
                            )
                            .into(),
                        )
                        .await?;
                }
                VCUCommand::FactoryReset => {
                    info!("{} Factory reset", style_address(destination_address));

                    socket.send(&hcu0.factory_reset()).await?;
                }
            }
        }
        Command::Engine {
            driver,
            address,
            command,
        } => {
            use glonax::driver::net::engine::Engine;

            let destination_address = j1939_address(address)?;
            let socket = CANSocket::bind(&SockAddrCAN::new(args.interface.as_str()))?;

            // TODO: Replace string with enum
            let ems = if driver == "volvo" {
                Box::new(glonax::driver::VolvoD7E::new(
                    args.interface.as_str(),
                    destination_address,
                    consts::J1939_ADDRESS_VOLVO_VECU,
                )) as Box<dyn Engine>
            } else {
                Box::new(glonax::driver::net::engine::EngineManagementSystem::new(
                    args.interface.as_str(),
                    destination_address,
                    consts::J1939_ADDRESS_OBDL,
                )) as Box<dyn Engine>
            };

            match command {
                EngineCommand::Rpm { rpm } => {
                    info!("{} Set RPM to {}", style_address(destination_address), rpm);

                    loop {
                        socket.send(&ems.request(rpm)).await?;
                        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                    }
                }
                EngineCommand::Start => {
                    info!("{} Start engine", style_address(destination_address));

                    loop {
                        socket.send(&ems.start(700)).await?;
                        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                    }
                }
                EngineCommand::Stop => {
                    info!("{} Stop engine", style_address(destination_address));

                    loop {
                        socket.send(&ems.stop(700)).await?;
                        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                    }
                }
            }
        }
        Command::Request { address, pgn } => {
            use glonax::j1939::{protocol, PGN};

            let destination_address = j1939_address(address)?;
            let socket = CANSocket::bind(&SockAddrCAN::new(args.interface.as_str()))?;

            let pgn = PGN::from(pgn);

            info!("{} Request {:?}", style_address(destination_address), pgn);

            socket
                .send(&protocol::request(
                    destination_address,
                    consts::J1939_ADDRESS_OBDL,
                    pgn,
                ))
                .await?;
        }
        Command::Send { interval, id, data } => {
            let socket = CANSocket::bind(&SockAddrCAN::new(args.interface.as_str()))?;

            let mut tick = Interval::new(interval);

            let frame = glonax::j1939::FrameBuilder::new(glonax::j1939::Id::new(
                u32::from_str_radix(id.as_str(), 16)?,
            ))
            .copy_from_slice(&hex::decode(data)?)
            .build();

            loop {
                tick.tick().await;
                socket.send(&frame).await?;
            }
        }
        Command::Broadcast { pgn, data } => {
            use glonax::j1939::PGN;

            let data = hex::decode(data)?;
            if data.len() <= glonax::j1939::PDU_MAX_LENGTH {
                return Err(anyhow::anyhow!("Data length is too short"));
            }

            let socket = CANSocket::bind(&SockAddrCAN::new(args.interface.as_str()))?;

            let pgn = PGN::from(pgn);

            // let frames = destination_specific(
            //     destination_address,
            //     consts::J1939_ADDRESS_OBDL,
            //     pgn,
            //     &[
            //         0x64, 0x00, 0x02, 0x01, 0x00, 0x00, 0x02, 0x01, 0x00, 0x00, 0x32, 0x00, 0x20,
            //         0x00, 0x01, 0x06,
            //     ],
            // );

            let mut bam =
                glonax::j1939::transport::BroadcastTransport::new(consts::J1939_ADDRESS_OBDL, pgn)
                    .with_data(&data);

            for _ in 0..bam.packet_count() + 1 {
                let frame = bam.next_frame();
                socket.send(&frame).await?;

                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }
        }
        Command::Fuzzer { interval, id } => {
            use glonax::rand::Rng;

            let socket = CANSocket::bind(&SockAddrCAN::new(args.interface.as_str()))?;
            let fuz0 = glonax::driver::Fuzzer::new(glonax::j1939::Id::new(u32::from_str_radix(
                id.as_str(),
                16,
            )?));

            let mut tick = Interval::new(
                interval.unwrap_or_else(|| glonax::rand::thread_rng().gen_range(0..=50)),
            );

            loop {
                tick.tick().await;
                socket.send(&fuz0.gen_frame()).await?;
            }
        }
        Command::Diagnostic => {
            let name = glonax::j1939::NameBuilder::default()
                .identity_number(0x1)
                .manufacturer_code(consts::J1939_NAME_MANUFACTURER_CODE)
                .function_instance(consts::J1939_NAME_FUNCTION_INSTANCE)
                .ecu_instance(consts::J1939_NAME_ECU_INSTANCE)
                .function(consts::J1939_NAME_FUNCTION)
                .vehicle_system(consts::J1939_NAME_VEHICLE_SYSTEM)
                .build();
            let network = ControlNetwork::bind(&args.interface, &name)?;

            diagnose(network).await?;
        }
        Command::Dump {
            exclude,
            pgn,
            priority,
            address,
        } => {
            let mut filter = if exclude {
                Filter::reject()
            } else {
                Filter::accept()
            };

            for pgn in pgn {
                filter.push(FilterItem::with_pgn(pgn));
            }
            for priority in priority {
                filter.push(FilterItem::with_priority(priority));
            }
            for addr in address
                .iter()
                .map(|s| j1939_address(s.to_owned()))
                .filter(|a| a.is_ok())
            {
                filter.push(FilterItem::with_source_address(addr?));
            }

            let socket = CANSocket::bind(&SockAddrCAN::new(args.interface.as_str()))?;

            print_frames(socket, filter).await?;
        }
        Command::Analyze {
            exclude,
            pgn,
            priority,
            address,
        } => {
            let mut filter = if exclude {
                Filter::reject()
            } else {
                Filter::accept()
            };

            for pgn in pgn {
                filter.push(FilterItem::with_pgn(pgn));
            }
            for priority in priority {
                filter.push(FilterItem::with_priority(priority));
            }
            for addr in address
                .iter()
                .map(|s| j1939_address(s.to_owned()))
                .filter(|a| a.is_ok())
            {
                filter.push(FilterItem::with_source_address(addr?));
            }

            let name = glonax::j1939::NameBuilder::default()
                .identity_number(0x1)
                .manufacturer_code(consts::J1939_NAME_MANUFACTURER_CODE)
                .function_instance(consts::J1939_NAME_FUNCTION_INSTANCE)
                .ecu_instance(consts::J1939_NAME_ECU_INSTANCE)
                .function(consts::J1939_NAME_FUNCTION)
                .vehicle_system(consts::J1939_NAME_VEHICLE_SYSTEM)
                .build();
            let network = ControlNetwork::bind(&args.interface, &name)?.with_filter(filter);

            analyze_frames(network).await?;
        }
    }

    Ok(())
}
