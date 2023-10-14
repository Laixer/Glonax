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

    log::debug!("Starting host service");

    let mut client = glonax::transport::ConnectionOptions::new()
        .read(true)
        .connect("localhost:30051", "glonax-agent/0.1.0")
        .await
        .unwrap();

    log::info!("Connected to {}", "localhost:30051");

    client.send_request(FrameMessage::Instance).await.unwrap();

    let frame = client.read_frame().await.unwrap();
    match frame.message {
        FrameMessage::Null => {
            log::info!("Received null");
        }
        FrameMessage::Status => {
            let status = client
                .packet::<glonax::core::Status>(frame.payload_length)
                .await
                .unwrap();
            log::info!("Received status: {}", status);
        }
        FrameMessage::Instance => {
            let instance = client
                .packet::<glonax::core::Instance>(frame.payload_length)
                .await
                .unwrap();
            log::info!("Received instance: {}", instance);
        }
        _ => {}
    }

    client.send_request(FrameMessage::Status).await.unwrap();

    let frame = client.read_frame().await.unwrap();
    match frame.message {
        FrameMessage::Null => {
            log::info!("Received null");
        }
        FrameMessage::Status => {
            let status = client
                .packet::<glonax::core::Status>(frame.payload_length)
                .await
                .unwrap();
            log::info!("Received status: {}", status);
        }
        FrameMessage::Instance => {
            let instance = client
                .packet::<glonax::core::Instance>(frame.payload_length)
                .await
                .unwrap();
            log::info!("Received instance: {}", instance);
        }
        _ => {}
    }

    // tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    loop {
        client.send_request(FrameMessage::Pose).await.unwrap();

        let frame = client.read_frame().await.unwrap();
        match frame.message {
            FrameMessage::Null => {
                log::info!("Received null");
            }
            FrameMessage::Status => {
                let status = client
                    .packet::<glonax::core::Status>(frame.payload_length)
                    .await
                    .unwrap();
                log::info!("Received status: {}", status);
            }
            FrameMessage::Instance => {
                let instance = client
                    .packet::<glonax::core::Instance>(frame.payload_length)
                    .await
                    .unwrap();
                log::info!("Received instance: {}", instance);
            }
            FrameMessage::Pose => {
                let pose = client
                    .packet::<glonax::core::Pose>(frame.payload_length)
                    .await
                    .unwrap();
                log::info!("Received pose: {}", pose);
            }
            FrameMessage::Engine => {
                let engine = client
                    .packet::<glonax::core::Engine>(frame.payload_length)
                    .await
                    .unwrap();
                log::info!("Received engine: {}", engine);
            }
            _ => {}
        }
    }

    //     client.send_request(FrameMessage::Engine).await.unwrap();

    //     let frame = client.read_frame().await.unwrap();
    //     match frame.message {
    //         FrameMessage::Null => {
    //             log::info!("Received null");
    //         }
    //         FrameMessage::Status => {
    //             let status = client
    //                 .packet::<glonax::core::Status>(frame.payload_length)
    //                 .await
    //                 .unwrap();
    //             log::info!("Received status: {}", status);
    //         }
    //         FrameMessage::Instance => {
    //             let instance = client
    //                 .packet::<glonax::core::Instance>(frame.payload_length)
    //                 .await
    //                 .unwrap();
    //             log::info!("Received instance: {}", instance);
    //         }
    //         FrameMessage::Pose => {
    //             let pose = client
    //                 .packet::<glonax::core::Pose>(frame.payload_length)
    //                 .await
    //                 .unwrap();
    //             log::info!("Received pose: {}", pose);
    //         }
    //         FrameMessage::Engine => {
    //             let engine = client
    //                 .packet::<glonax::core::Engine>(frame.payload_length)
    //                 .await
    //                 .unwrap();
    //             log::info!("Received engine: {}", engine);
    //         }
    //         _ => {}
    //     }

    //     client.send_request(FrameMessage::Status).await.unwrap();

    //     let frame = client.read_frame().await.unwrap();
    //     match frame.message {
    //         FrameMessage::Null => {
    //             log::info!("Received null");
    //         }
    //         FrameMessage::Status => {
    //             let status = client
    //                 .packet::<glonax::core::Status>(frame.payload_length)
    //                 .await
    //                 .unwrap();
    //             log::info!("Received status: {}", status);
    //         }
    //         FrameMessage::Instance => {
    //             let instance = client
    //                 .packet::<glonax::core::Instance>(frame.payload_length)
    //                 .await
    //                 .unwrap();
    //             log::info!("Received instance: {}", instance);
    //         }
    //         FrameMessage::Pose => {
    //             let pose = client
    //                 .packet::<glonax::core::Pose>(frame.payload_length)
    //                 .await
    //                 .unwrap();
    //             log::info!("Received pose: {}", pose);
    //         }
    //         _ => {}
    //     }

    //     client.send_request(FrameMessage::VMS).await.unwrap();

    //     let frame = client.read_frame().await.unwrap();
    //     match frame.message {
    //         FrameMessage::Null => {
    //             log::info!("Received null");
    //         }
    //         FrameMessage::Status => {
    //             let status = client
    //                 .packet::<glonax::core::Status>(frame.payload_length)
    //                 .await
    //                 .unwrap();
    //             log::info!("Received status: {}", status);
    //         }
    //         FrameMessage::Instance => {
    //             let instance = client
    //                 .packet::<glonax::core::Instance>(frame.payload_length)
    //                 .await
    //                 .unwrap();
    //             log::info!("Received instance: {}", instance);
    //         }
    //         FrameMessage::Pose => {
    //             let pose = client
    //                 .packet::<glonax::core::Pose>(frame.payload_length)
    //                 .await
    //                 .unwrap();
    //             log::info!("Received pose: {}", pose);
    //         }
    //         FrameMessage::VMS => {
    //             let vms = client
    //                 .packet::<glonax::core::Host>(frame.payload_length)
    //                 .await
    //                 .unwrap();
    //             log::info!("Received vms: {}", vms);
    //         }
    //         _ => {}
    //     }

    //     client.send_request(FrameMessage::GNSS).await.unwrap();

    //     let frame = client.read_frame().await.unwrap();
    //     match frame.message {
    //         FrameMessage::Null => {
    //             log::info!("Received null");
    //         }
    //         FrameMessage::Status => {
    //             let status = client
    //                 .packet::<glonax::core::Status>(frame.payload_length)
    //                 .await
    //                 .unwrap();
    //             log::info!("Received status: {}", status);
    //         }
    //         FrameMessage::Instance => {
    //             let instance = client
    //                 .packet::<glonax::core::Instance>(frame.payload_length)
    //                 .await
    //                 .unwrap();
    //             log::info!("Received instance: {}", instance);
    //         }
    //         FrameMessage::Pose => {
    //             let pose = client
    //                 .packet::<glonax::core::Pose>(frame.payload_length)
    //                 .await
    //                 .unwrap();
    //             log::info!("Received pose: {}", pose);
    //         }
    //         FrameMessage::VMS => {
    //             let vms = client
    //                 .packet::<glonax::core::Host>(frame.payload_length)
    //                 .await
    //                 .unwrap();
    //             log::info!("Received vms: {}", vms);
    //         }
    //         FrameMessage::GNSS => {
    //             let gnss = client
    //                 .packet::<glonax::core::Gnss>(frame.payload_length)
    //                 .await
    //                 .unwrap();
    //             log::info!("Received gnss: {}", gnss);
    //         }
    //         _ => {}
    //     }

    // tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    // }

    // Ok(())
}
