// Copyright (C) 2023 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use clap::Parser;

#[derive(Parser)]
#[command(author = "Copyright (C) 2023 Laixer Equipment B.V.")]
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

    let mut address = args.address.clone();

    if !address.contains(':') {
        address.push(':');
        address.push_str(&glonax::consts::DEFAULT_NETWORK_PORT.to_string());
    }

    let address = std::net::ToSocketAddrs::to_socket_addrs(&address)?
        .next()
        .unwrap();

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

    log::debug!("Waiting for connection to {}", address);

    let mut client = glonax::protocol::Connection::default()
        .connect(
            address.to_owned(),
            format!("{}/{}", "glonax-cli", glonax::consts::VERSION),
        )
        .await?;

    println!("Connected to {}", address);

    let frame = client.read_frame().await?;
    match frame.message {
        glonax::protocol::frame::FrameMessage::Instance => {
            let instance = client
                .recv_packet::<glonax::core::Instance>(frame.payload_length)
                .await?;

            println!("Instance ID: {}", instance.id);
            println!("Instance Model: {}", instance.model);
            println!("Instance Name: {}", instance.name);
        }
        _ => {
            eprintln!("Invalid response from server");
            return Ok(());
        }
    }

    let random_number = glonax::rand::random::<i32>();

    client
        .send_packet(&glonax::protocol::frame::Echo::new(random_number))
        .await?;

    let frame = client.read_frame().await?;
    match frame.message {
        glonax::protocol::frame::FrameMessage::Echo => {
            let echo = client
                .recv_packet::<glonax::protocol::frame::Echo>(frame.payload_length)
                .await?;

            if random_number != echo.payload() {
                eprintln!("Invalid echo response from server");
                return Ok(());
            }
        }
        _ => {
            eprintln!("Invalid response from server");
            return Ok(());
        }
    }

    fn print_help() {
        println!("Commands:");
        println!("  r | request <class>");
        println!("  w | watch");
        println!();
        println!("Classes:");
        println!("  s | status");
        println!("  i | instance");
        println!("  p | pose");
        println!("  e | engine");
        println!("  h | host");
        println!("  g | gps");
        println!();
        println!("Commands:");
        println!("  ? | help");
        println!("  q | quit");
    }

    async fn print_frame(
        client: &mut glonax::protocol::Client<tokio::net::TcpStream>,
    ) -> std::io::Result<()> {
        let frame = client.read_frame().await?;
        match frame.message {
            glonax::protocol::frame::FrameMessage::Status => {
                let status = client
                    .recv_packet::<glonax::core::Status>(frame.payload_length)
                    .await?;

                println!("{}", status);
            }
            glonax::protocol::frame::FrameMessage::Instance => {
                let instance = client
                    .recv_packet::<glonax::core::Instance>(frame.payload_length)
                    .await?;

                println!("ID: {}", instance.id);
                println!("Model: {}", instance.model);
                println!("Name: {}", instance.name);
            }
            glonax::protocol::frame::FrameMessage::Pose => {
                let pose = client
                    .recv_packet::<glonax::core::Pose>(frame.payload_length)
                    .await?;

                println!("{}", pose);
            }
            glonax::protocol::frame::FrameMessage::Engine => {
                let engine = client
                    .recv_packet::<glonax::core::Engine>(frame.payload_length)
                    .await?;

                println!("{}", engine);
            }
            glonax::protocol::frame::FrameMessage::VMS => {
                let host = client
                    .recv_packet::<glonax::core::Host>(frame.payload_length)
                    .await?;

                println!("{}", host);
            }
            glonax::protocol::frame::FrameMessage::GNSS => {
                let gnss = client
                    .recv_packet::<glonax::core::Gnss>(frame.payload_length)
                    .await?;

                println!("{}", gnss);
            }
            _ => {
                eprintln!("Invalid response from server");
            }
        }

        Ok(())
    }

    fn str_to_class(s: &str) -> Option<glonax::protocol::frame::FrameMessage> {
        match s {
            "s" | "status" => Some(glonax::protocol::frame::FrameMessage::Status),
            "i" | "instance" => Some(glonax::protocol::frame::FrameMessage::Instance),
            "p" | "pose" => Some(glonax::protocol::frame::FrameMessage::Pose),
            "e" | "engine" => Some(glonax::protocol::frame::FrameMessage::Engine),
            "h" | "host" | "vms" => Some(glonax::protocol::frame::FrameMessage::VMS),
            "g" | "gps" | "gnss" => Some(glonax::protocol::frame::FrameMessage::GNSS),
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

                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    }
                } else {
                    eprintln!("Invalid request");
                    continue;
                }
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
