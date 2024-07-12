// Copyright (C) 2024 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use clap::{Parser, ValueEnum};

#[derive(Parser)]
#[command(author = "Copyright (C) 2024 Laixer Equipment B.V.")]
#[command(version, propagate_version = true)]
#[command(about = "Glonax input daemon", long_about = None)]
struct Args {
    /// Socket path.
    #[arg(short = 'c', long = "connect", default_value = "/tmp/glonax.sock")]
    path: std::path::PathBuf,
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
    /// GNSS.
    Gnss, // TODO: can be removed
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
}

fn string_to_bool(s: &str) -> Option<bool> {
    match s.to_lowercase().as_str() {
        "on" => Some(true),
        "true" => Some(true),
        "off" => Some(false),
        "false" => Some(false),
        _ => None,
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let bin_name = env!("CARGO_BIN_NAME").to_string();

    use log::LevelFilter;

    let args = Args::parse();

    let mut log_config = simplelog::ConfigBuilder::new();
    log_config.set_time_level(log::LevelFilter::Off);
    log_config.set_thread_level(log::LevelFilter::Off);
    log_config.set_target_level(log::LevelFilter::Off);
    log_config.set_location_level(log::LevelFilter::Off);
    log_config.add_filter_ignore_str("sled");
    log_config.add_filter_ignore_str("mio");

    let log_level = match args.verbose {
        0 => LevelFilter::Info,
        1 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    simplelog::TermLogger::init(
        log_level,
        log_config.build(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )?;

    // TODO: Wrap into run function

    glonax::log_system();

    log::info!("Starting {}", bin_name);
    log::debug!("Runtime version: {}", glonax::consts::VERSION);

    let user_agent = format!("{}/{}", bin_name, glonax::consts::VERSION);
    let (mut client, instance) = glonax::protocol::client::ClientBuilder::new(user_agent)
        .stream(true)
        .unix_connect(&args.path)
        .await?;

    log::debug!("Connected to {}", args.path.display());
    log::info!("{}", instance);

    if !glonax::is_compatibile(instance.version()) {
        return Err(anyhow::anyhow!("Incompatible runtime version"));
    }

    match args.command {
        Command::Watch { filter } => loop {
            use glonax::protocol::Packetize;

            let frame = client.read_frame().await?;

            // TODO: Filter objects
            // TODO: If possible, convert back into an object
            // TODO: Offer: async fn wait_io_sub(&mut self, command_tx: CommandSender, mut signal_rx: SignalReceiver) {
            match frame.message {
                // TODO: Error messages are never sent
                glonax::protocol::frame::SessionError::MESSAGE_TYPE => {
                    let error = client
                        .recv_packet::<glonax::protocol::frame::SessionError>(frame.payload_length)
                        .await?;

                    log::error!("{:?}", error);
                }
                glonax::core::ModuleStatus::MESSAGE_TYPE => {
                    let status = client
                        .recv_packet::<glonax::core::ModuleStatus>(frame.payload_length)
                        .await?;

                    if let Some(filter) = filter {
                        if filter == ObjectFilter::Status {
                            log::info!("{}", status);
                        }
                    } else {
                        log::info!("{}", status);
                    }
                }
                glonax::core::Instance::MESSAGE_TYPE => {
                    let instance = client
                        .recv_packet::<glonax::core::Instance>(frame.payload_length)
                        .await?;

                    log::info!("{}", instance);
                }
                glonax::core::Engine::MESSAGE_TYPE => {
                    let engine = client
                        .recv_packet::<glonax::core::Engine>(frame.payload_length)
                        .await?;

                    if let Some(filter) = filter {
                        if filter == ObjectFilter::Engine {
                            log::info!("{}", engine);
                        }
                    } else {
                        log::info!("{}", engine);
                    }
                }
                glonax::core::Gnss::MESSAGE_TYPE => {
                    let gnss = client
                        .recv_packet::<glonax::core::Gnss>(frame.payload_length)
                        .await?;

                    if let Some(filter) = filter {
                        if filter == ObjectFilter::Gnss {
                            log::info!("{}", gnss);
                        }
                    } else {
                        log::info!("{}", gnss);
                    }
                }
                glonax::core::Motion::MESSAGE_TYPE => {
                    let motion = client
                        .recv_packet::<glonax::core::Motion>(frame.payload_length)
                        .await?;

                    if let Some(filter) = filter {
                        if filter == ObjectFilter::Motion {
                            log::info!("{}", motion);
                        }
                    } else {
                        log::info!("{}", motion);
                    }
                }
                glonax::core::Rotator::MESSAGE_TYPE => {
                    let rotator = client
                        .recv_packet::<glonax::core::Rotator>(frame.payload_length)
                        .await?;

                    if let Some(filter) = filter {
                        if filter == ObjectFilter::Rotator {
                            log::info!("{}", rotator);
                        }
                    } else {
                        log::info!("{}", rotator);
                    }
                }
                glonax::world::Actor::MESSAGE_TYPE => {
                    let actor = client
                        .recv_packet::<glonax::world::Actor>(frame.payload_length)
                        .await?;

                    let bucket_world_location = actor.world_location("bucket");
                    log::info!(
                        "Bucket: world location: X={:.2} Y={:.2} Z={:.2}",
                        bucket_world_location.x,
                        bucket_world_location.y,
                        bucket_world_location.z
                    );
                }
                _ => {
                    log::error!("Unknown message type: 0x{:X}", frame.message);
                }
            }
        },
        Command::Engine { rpm } => {
            log::info!("Requesting engine RPM: {}", rpm);

            let engine = if rpm > 0 {
                glonax::core::Engine::from_rpm(rpm)
            } else {
                glonax::core::Engine::shutdown()
            };

            client.send_packet(&engine).await?;
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
    }

    Ok(())
}
