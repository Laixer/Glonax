// Copyright (C) 2024 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use clap::{Parser, ValueEnum, ValueHint};

mod config;

#[derive(Parser)]
#[command(author = "Copyright (C) 2024 Laixer Equipment B.V.")]
#[command(version, propagate_version = true)]
#[command(about = "Glonax input daemon", long_about = None)]
struct Args {
    /// Configuration file.
    #[arg(
        short = 'c',
        long = "config",
        alias = "conf",
        default_value = "/etc/glonax.conf",
        value_name = "FILE",
        value_hint = ValueHint::FilePath
    )]
    config: std::path::PathBuf,
    /// Socket path.
    #[arg(
        short = 's',
        long = "socket",
        value_hint = ValueHint::FilePath
    )]
    path: Option<std::path::PathBuf>,
    /// Level of verbosity.
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
    /// Commands.
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum ObjectFilter {
    /// Control.
    Control,
    /// Engine.
    Engine,
    // /// GNSS.
    // Gnss, // TODO: can be removed
    /// Motion.
    Motion,
    /// Target.
    Target,
    /// Rotator.
    Rotator,
    /// Module status.
    Status,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Watch for glonax messages.
    Watch {
        /// Filter messages.
        #[arg(short, long)]
        filter: Option<ObjectFilter>,
    },
    /// Engine commands.
    Engine {
        /// RPM
        rpm: u16,
    },
    /// Shutdown engine.
    Shutdown,
    /// Motion lock commands.
    MotionLock {
        /// On or off.
        toggle: String,
    },
    /// Lights commands.
    Lights {
        /// On or off.
        toggle: String,
    },
    /// Horn commands.
    Horn {
        /// On or off.
        toggle: String,
    },
    /// Quick disconnect commands.
    QuickDisconnect {
        /// On or off.
        toggle: String,
    },
    /// Queue target.
    Target { x: f32, y: f32, z: f32 },
    /// Instance information.
    Info,
}

fn string_to_bool(s: &str) -> Option<bool> {
    match s.to_lowercase().as_str() {
        "1" => Some(true),
        "on" => Some(true),
        "true" => Some(true),
        "0" => Some(false),
        "off" => Some(false),
        "false" => Some(false),
        _ => None,
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use log::LevelFilter;

    let args = Args::parse();

    let config: config::Config = glonax::from_file(&args.config)?;

    let mut log_config = simplelog::ConfigBuilder::new();
    log_config.set_time_level(log::LevelFilter::Off);
    log_config.set_thread_level(log::LevelFilter::Off);
    log_config.set_target_level(log::LevelFilter::Off);
    log_config.set_location_level(log::LevelFilter::Off);
    log_config.add_filter_ignore_str("sled");
    log_config.add_filter_ignore_str("mio");

    let log_level = match args.verbose {
        0 => LevelFilter::Error,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    simplelog::TermLogger::init(
        log_level,
        log_config.build(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )?;

    log::trace!("{:#?}", config);

    run(config, args).await
}

async fn run(config: config::Config, args: Args) -> anyhow::Result<()> {
    use glonax::consts::*;

    let bin_name = env!("CARGO_BIN_NAME").to_string();

    let socket_path = args
        .path
        .unwrap_or_else(|| config.unix_listener.path.clone());

    glonax::log_system();

    log::info!("Starting {}", bin_name);
    log::debug!("Runtime version: {}", VERSION);

    let user_agent = format!("{}/{}", bin_name, VERSION);
    let (mut client, instance) = glonax::protocol::client::ClientBuilder::new(user_agent)
        .stream(true)
        .unix_connect(&socket_path)
        .await?;

    log::debug!("Connected to {}", socket_path.display());
    log::info!("{}", instance);

    if instance.id().is_nil() {
        log::warn!("Instance ID is not set or invalid");
    }

    if !glonax::is_compatibile(instance.version()) {
        return Err(anyhow::anyhow!("Incompatible runtime version"));
    }

    match args.command {
        Command::Watch { filter } => loop {
            use glonax::protocol::Packetize;

            let frame = client.read_frame().await?;

            // TODO: If possible, convert back into an object
            // TODO: Offer: async fn wait_io_sub(&mut self, command_tx: CommandSender, mut signal_rx: SignalReceiver) {
            match frame.message {
                glonax::core::ModuleStatus::MESSAGE_TYPE => {
                    let status = client
                        .recv_packet::<glonax::core::ModuleStatus>(frame.payload_length)
                        .await?;

                    if let Some(filter) = filter {
                        if filter == ObjectFilter::Status {
                            if let Some(error) = &status.error {
                                println!(
                                    "name={} state={} error={}",
                                    status.name, status.state, error
                                );
                            } else {
                                println!("name={} state={}", status.name, status.state);
                            }
                        }
                    } else {
                        println!("Status: {}", status);
                    }
                }
                glonax::core::Instance::MESSAGE_TYPE => {
                    let instance = client
                        .recv_packet::<glonax::core::Instance>(frame.payload_length)
                        .await?;

                    println!("{}", instance);
                }
                glonax::core::Engine::MESSAGE_TYPE => {
                    let engine = client
                        .recv_packet::<glonax::core::Engine>(frame.payload_length)
                        .await?;

                    if let Some(filter) = filter {
                        if filter == ObjectFilter::Engine {
                            println!(
                                "driver_demand={} actual_engine={} rpm={} state={:?}",
                                engine.driver_demand,
                                engine.actual_engine,
                                engine.rpm,
                                engine.state
                            );
                        }
                    } else {
                        println!("Engine: {}", engine);
                    }
                }
                // glonax::core::Gnss::MESSAGE_TYPE => {
                //     let gnss = client
                //         .recv_packet::<glonax::core::Gnss>(frame.payload_length)
                //         .await?;

                //     if let Some(filter) = filter {
                //         if filter == ObjectFilter::Gnss {
                //             println!("{}", gnss);
                //         }
                //     } else {
                //         println!("GNSS: {}", gnss);
                //     }
                // }
                glonax::core::Motion::MESSAGE_TYPE => {
                    let motion = client
                        .recv_packet::<glonax::core::Motion>(frame.payload_length)
                        .await?;

                    if let Some(filter) = filter {
                        if filter == ObjectFilter::Motion {
                            println!("{}", motion);
                        }
                    } else {
                        println!("Motion: {}", motion);
                    }
                }
                glonax::core::Rotator::MESSAGE_TYPE => {
                    let rotator = client
                        .recv_packet::<glonax::core::Rotator>(frame.payload_length)
                        .await?;

                    if let Some(filter) = filter {
                        if filter == ObjectFilter::Rotator {
                            println!(
                                "source={} reference={:?} roll={:.2} pitch={:.2} yaw={:.2}",
                                rotator.source,
                                rotator.reference,
                                rotator.rotator.euler_angles().0.to_degrees(),
                                rotator.rotator.euler_angles().1.to_degrees(),
                                rotator.rotator.euler_angles().2.to_degrees()
                            );
                        }
                    } else {
                        println!("Rotator: {}", rotator);
                    }
                }
                glonax::world::Actor::MESSAGE_TYPE => {
                    let actor = client
                        .recv_packet::<glonax::world::Actor>(frame.payload_length)
                        .await?;

                    let bucket_world_location = actor.world_location("bucket");
                    println!(
                        "Bucket: world location: X={:.2} Y={:.2} Z={:.2}",
                        bucket_world_location.x, bucket_world_location.y, bucket_world_location.z
                    );
                }
                _ => {
                    eprintln!("Unknown message type: 0x{:X}", frame.message);
                }
            }
        },
        Command::Engine { rpm } => {
            log::info!("Requesting engine RPM: {}", rpm);

            // TODO: Clap should have a way to validate this
            if rpm > 0 && rpm <= 10_000 {
                let engine = glonax::core::Engine::from_rpm(rpm);
                client.send_packet(&engine).await?;
            }
        }
        Command::Shutdown => {
            log::info!("Shutting down engine");

            let engine = glonax::core::Engine::shutdown();
            client.send_packet(&engine).await?;
        }
        Command::MotionLock { toggle } => {
            let toggle = string_to_bool(&toggle)
                .ok_or_else(|| anyhow::anyhow!("Invalid value for motion lock"))?;

            log::info!("Setting motion lock: {}", if toggle { "on" } else { "off" });

            let motion = if toggle {
                glonax::core::Motion::StopAll
            } else {
                glonax::core::Motion::ResumeAll
            };

            client.send_packet(&motion).await?;
        }
        Command::Lights { toggle } => {
            let toggle = string_to_bool(&toggle)
                .ok_or_else(|| anyhow::anyhow!("Invalid value for lights"))?;

            log::info!("Setting lights: {}", if toggle { "on" } else { "off" });

            let control = glonax::core::Control::MachineIllumination(toggle);
            client.send_packet(&control).await?;
        }
        Command::Horn { toggle } => {
            let toggle =
                string_to_bool(&toggle).ok_or_else(|| anyhow::anyhow!("Invalid value for horn"))?;

            log::info!("Setting horn: {}", if toggle { "on" } else { "off" });

            let control = glonax::core::Control::MachineHorn(toggle);
            client.send_packet(&control).await?;
        }
        Command::QuickDisconnect { toggle } => {
            let toggle = string_to_bool(&toggle)
                .ok_or_else(|| anyhow::anyhow!("Invalid value for quick disconnect"))?;

            log::info!(
                "Setting quick disconnect: {}",
                if toggle { "on" } else { "off" }
            );

            let control = glonax::core::Control::HydraulicQuickDisconnect(toggle);
            client.send_packet(&control).await?;
        }
        Command::Target { x, y, z } => {
            let target = glonax::core::Target::from_point(x, y, z);

            log::info!("Queue target: {}", target);

            client.send_packet(&target).await?;
        }
        Command::Info => {
            println!(
                "{} {} {:?} {} {}",
                instance.id(),
                instance.model(),
                instance.ty(),
                instance.version_string(),
                instance.serial_number()
            );
        }
    }

    Ok(())
}
