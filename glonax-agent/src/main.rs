// Copyright (C) 2023 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut log_config = simplelog::ConfigBuilder::new();

    simplelog::TermLogger::init(
        log::LevelFilter::Trace,
        log_config.build(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )?;

    use glonax::transport::frame::FrameMessage;

    let bin_name = env!("CARGO_BIN_NAME");

    let address = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "localhost:30051".to_string());

    log::debug!("Starting host service");

    let mut client = glonax::transport::ConnectionOptions::new()
        .read(true)
        .connect(
            address.clone(),
            format!("{}/{}", bin_name, glonax::consts::VERSION),
        )
        .await?;

    log::info!("Connected to {}", address);

    client.send_request(FrameMessage::Instance).await?;
    read_message(&mut client).await?;

    client.send_request(FrameMessage::Status).await?;
    read_message(&mut client).await?;

    // tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // loop {
    for _ in 0..100 {
        client.send_request(FrameMessage::Pose).await?;
        read_message(&mut client).await?;

        // client.send_request(FrameMessage::Engine).await?;
        // read_message(&mut client).await?;

        client.send_request(FrameMessage::Instance).await?;
        read_message(&mut client).await?;

        client.send_request(FrameMessage::Status).await?;
        read_message(&mut client).await?;

        client.send_request(FrameMessage::VMS).await?;
        read_message(&mut client).await?;

        client.send_request(FrameMessage::GNSS).await?;
        read_message(&mut client).await?;

        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    }

    client.send_request(FrameMessage::Shutdown).await?;

    Ok(())
}

async fn read_message(
    client: &mut glonax::transport::Client<tokio::net::TcpStream>,
) -> std::io::Result<()> {
    use glonax::transport::frame::FrameMessage;

    let frame = client.read_frame().await?;

    match frame.message {
        FrameMessage::Null => {
            log::info!("Received null");
        }
        FrameMessage::Shutdown => {
            log::info!("Received shutdown");
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Received shutdown",
            ))?;
        }
        FrameMessage::Status => {
            let status = client
                .packet::<glonax::core::Status>(frame.payload_length)
                .await?;
            log::info!("Received status: {}", status);
        }
        FrameMessage::Instance => {
            let instance = client
                .packet::<glonax::core::Instance>(frame.payload_length)
                .await?;
            log::info!("Received instance: {}", instance);
        }
        FrameMessage::Pose => {
            let pose = client
                .packet::<glonax::core::Pose>(frame.payload_length)
                .await?;
            log::info!("Received pose: {}", pose);
        }
        FrameMessage::VMS => {
            let vms = client
                .packet::<glonax::core::Host>(frame.payload_length)
                .await?;
            log::info!("Received vms: {}", vms);
        }
        FrameMessage::GNSS => {
            let gnss = client
                .packet::<glonax::core::Gnss>(frame.payload_length)
                .await?;
            log::info!("Received gnss: {}", gnss);
        }
        _ => {}
    }

    Ok(())
}
