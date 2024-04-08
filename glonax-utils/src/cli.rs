// Copyright (C) 2024 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use clap::Parser;

#[derive(Parser)]
#[command(author = "Copyright (C) 2024 Laixer Equipment B.V.")]
#[command(version, propagate_version = true)]
#[command(about = "Glonax input daemon", long_about = None)]
struct Args {
    /// Remote network address.
    #[arg(short = 'c', long = "connect", default_value = "127.0.0.1")]
    address: String,
    /// Level of verbosity.
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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

    // Connect over TCP

    let mut address = args.address.clone();

    log::debug!("Connecting to {}", address);

    if !address.contains(':') {
        address.push(':');
        address.push_str(&glonax::consts::DEFAULT_NETWORK_PORT.to_string());
    }

    log::debug!("Waiting for connection to {}", address);

    let (mut client, instance) = glonax::protocol::client::tcp::connect_with(
        address.to_owned(),
        format!("{}/{}", "glonax-cli", glonax::consts::VERSION),
        true,
        false,
    )
    .await?;

    println!("Connected to {}", address);

    // Connect over Unix socket

    // let (mut client, instance) = glonax::protocol::client::unix::connect(
    //     glonax::consts::DEFAULT_SOCKET_PATH,
    //     format!("{}/{}", "glonax-cli", glonax::consts::VERSION),
    // )
    // .await?;

    // println!("Connected to {}", glonax::consts::DEFAULT_SOCKET_PATH);

    println!("{}", instance);

    fn print_help() {
        println!("Commands:");
        println!("  r  | request <class>");
        println!("  w  | watch <class>");
        println!("  e  | engine <command");
        println!("  p  | ping");
        println!("  qd | quick disconnect <on|off>");
        println!("  l  | lights <on|off>");
        println!("  h  | horn <on|off>");
        println!("  x");
        println!();
        println!("Classes:");
        println!("  s | status");
        println!("  i | instance");
        println!("  e | engine");
        println!("  h | host");
        println!("  g | gps");
        println!("  a | actor");
        println!();
        println!("Engine commands:");
        println!("  i1 | idle 1");
        println!("  i2 | idle 2");
        println!("  f1 | fine 1");
        println!("  f2 | fine 2");
        println!("  f3 | fine 3");
        println!("  g1 | general 1");
        println!("  g2 | general 2");
        println!("  g3 | general 3");
        println!("  h  | high");
        println!("  p  | power max");
        println!("  s  | shutdown");
        println!("Commands:");
        println!("  ? | help");
        println!("  q | quit");
    }

    async fn print_frame<T: tokio::io::AsyncWrite + tokio::io::AsyncRead + Unpin>(
        client: &mut glonax::protocol::Stream<T>,
    ) -> std::io::Result<()> {
        use glonax::protocol::Packetize;

        let frame = client.read_frame().await?;
        match frame.message {
            glonax::core::Status::MESSAGE_TYPE => {
                let status = client
                    .recv_packet::<glonax::core::Status>(frame.payload_length)
                    .await?;

                println!("{}", status);
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

                println!("{}", engine);
            }
            glonax::core::Host::MESSAGE_TYPE => {
                let host = client
                    .recv_packet::<glonax::core::Host>(frame.payload_length)
                    .await?;

                println!("{}", host);
            }
            glonax::core::Gnss::MESSAGE_TYPE => {
                let gnss = client
                    .recv_packet::<glonax::core::Gnss>(frame.payload_length)
                    .await?;

                println!("{}", gnss);
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
                eprintln!("Invalid response from server");
            }
        }

        Ok(())
    }

    fn str_to_class(s: &str) -> Option<u8> {
        use glonax::protocol::Packetize;

        match s {
            "s" | "status" => Some(glonax::core::Status::MESSAGE_TYPE),
            "i" | "instance" => Some(glonax::core::Instance::MESSAGE_TYPE),
            "e" | "engine" => Some(glonax::core::Engine::MESSAGE_TYPE),
            "h" | "host" | "vms" => Some(glonax::core::Host::MESSAGE_TYPE),
            "g" | "gps" | "gnss" => Some(glonax::core::Gnss::MESSAGE_TYPE),
            "a" | "actor" => Some(glonax::world::Actor::MESSAGE_TYPE),
            _ => None,
        }
    }

    use std::io::Write;

    loop {
        let mut input = String::new();

        print!("glonax> ");
        std::io::stdout().flush().unwrap();

        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        match input {
            s if s.starts_with("request ") || s.starts_with("r ") => {
                let mut parts = s.split_whitespace();
                parts.next();

                if let Some(class) = str_to_class(parts.next().unwrap()) {
                    client.send_request(class).await?;
                    print_frame(&mut client).await?;
                } else {
                    eprintln!("Invalid request");
                    continue;
                }
            }
            s if s.starts_with("watch ") || s.starts_with("w ") => {
                let mut parts = s.split_whitespace();
                parts.next();

                if let Some(class) = str_to_class(parts.next().unwrap()) {
                    loop {
                        client.send_request(class).await?;
                        print_frame(&mut client).await?;

                        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                    }
                }

                eprintln!("Invalid request");
                continue;
            }
            s if s.starts_with("engine ") || s.starts_with("e ") => {
                let mut parts = s.split_whitespace();
                parts.next();

                let control = match parts.next() {
                    Some("i1") => glonax::core::Control::EngineRequest(800),
                    Some("i2") => glonax::core::Control::EngineRequest(1000),
                    Some("f1") => glonax::core::Control::EngineRequest(1200),
                    Some("f2") => glonax::core::Control::EngineRequest(1300),
                    Some("f3") => glonax::core::Control::EngineRequest(1400),
                    Some("g1") => glonax::core::Control::EngineRequest(1500),
                    Some("g2") => glonax::core::Control::EngineRequest(1600),
                    Some("g3") => glonax::core::Control::EngineRequest(1700),
                    Some("h") => glonax::core::Control::EngineRequest(1800),
                    Some("p") => glonax::core::Control::EngineRequest(1900),
                    Some("s") => glonax::core::Control::EngineShutdown,
                    _ => {
                        eprintln!("Invalid engine command");
                        continue;
                    }
                };

                client.send_packet(&control).await?;
            }
            s if s.starts_with("quick disconnect ") || s.starts_with("qd ") => {
                let mut parts = s.split_whitespace();
                parts.next();

                let control = match parts.next() {
                    Some("on") => glonax::core::Control::HydraulicQuickDisconnect(true),
                    Some("off") => glonax::core::Control::HydraulicQuickDisconnect(false),
                    _ => {
                        eprintln!("Invalid quick disconnect command");
                        continue;
                    }
                };

                client.send_packet(&control).await?;
            }
            s if s.starts_with("lights ") || s.starts_with("l ") => {
                let mut parts = s.split_whitespace();
                parts.next();

                let control = match parts.next() {
                    Some("on") => glonax::core::Control::MachineIllumination(true),
                    Some("off") => glonax::core::Control::MachineIllumination(false),
                    _ => {
                        eprintln!("Invalid lights command");
                        continue;
                    }
                };

                client.send_packet(&control).await?;
            }
            s if s.starts_with("horn ") || s.starts_with("h ") => {
                let mut parts = s.split_whitespace();
                parts.next();

                let control = match parts.next() {
                    Some("on") => glonax::core::Control::MachineHorn(true),
                    Some("off") => glonax::core::Control::MachineHorn(false),
                    _ => {
                        eprintln!("Invalid horn command");
                        continue;
                    }
                };

                client.send_packet(&control).await?;
            }
            "p" | "ping" => loop {
                let time_elapsed = client.probe().await?;

                println!("Echo response time: {} ms", time_elapsed.as_millis());

                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            },
            "test" => {
                let target = glonax::core::Target::from_point(300.0, 400.0, 330.0);
                client.send_packet(&target).await?;
            }
            "q" | "quit" => {
                return Ok(());
            }
            _ => {
                print_help();
            }
        }
    }
}
