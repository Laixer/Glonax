// Copyright (C) 2023 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use clap::Parser;

mod config;

const DEVICE_NET_LOCAL_ADDR: u8 = 0x9e;

#[derive(Parser)]
#[command(author = "Copyright (C) 2023 Laixer Equipment B.V.")]
#[command(version, propagate_version = true)]
#[command(about = "Glonax ECU daemon", long_about = None)]
struct Args {
    /// Bind address.
    #[arg(short = 'b', long = "bind", default_value = "[::1]:50051")]
    address: String,
    /// CAN network interface.
    interface: String,
    /// Daemonize the service.
    #[arg(long)]
    daemon: bool,
    /// Level of verbosity.
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let bin_name = env!("CARGO_BIN_NAME");

    let mut config = config::EcuConfig {
        address: args.address,
        interface: args.interface,
        global: glonax::GlobalConfig::default(),
    };

    config.global.bin_name = bin_name.to_string();
    config.global.daemon = args.daemon;

    let mut log_config = simplelog::ConfigBuilder::new();
    if args.daemon {
        log_config.set_time_level(log::LevelFilter::Off);
        log_config.set_thread_level(log::LevelFilter::Off);
    } else {
        log_config.set_time_offset_to_local().ok();
        log_config.set_time_format_rfc2822();
    }

    log_config.set_target_level(log::LevelFilter::Off);
    log_config.set_location_level(log::LevelFilter::Off);
    log_config.add_filter_ignore_str("sled");
    log_config.add_filter_ignore_str("mio");

    let log_level = if args.daemon {
        log::LevelFilter::Info
    } else {
        match args.verbose {
            0 => log::LevelFilter::Error,
            1 => log::LevelFilter::Info,
            2 => log::LevelFilter::Debug,
            _ => log::LevelFilter::Trace,
        }
    };

    let color_choice = if args.daemon {
        simplelog::ColorChoice::Never
    } else {
        simplelog::ColorChoice::Auto
    };

    simplelog::TermLogger::init(
        log_level,
        log_config.build(),
        simplelog::TerminalMode::Mixed,
        color_choice,
    )?;

    if args.daemon {
        log::debug!("Running service as daemon");
    }

    log::trace!("{:#?}", config);

    daemonize(&config).await
}

use glonax::{net::J1939Network, Configurable};
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{transport::Server, Request, Response, Status};

struct VehicleManagemetService {
    net: J1939Network,
    // motion_device: Arc<Mutex<glonax::net::ActuatorService>>,
    motion_device: Arc<glonax::net::ActuatorService>,
    signal_writer: glonax::channel::BroadcastChannelWriter<glonax::transport::Signal>,
}

impl VehicleManagemetService {
    pub fn new(
        config: config::EcuConfig,
        signal_writer: glonax::channel::BroadcastChannelWriter<glonax::transport::Signal>,
    ) -> Self {
        let net = J1939Network::new(&config.interface, DEVICE_NET_LOCAL_ADDR).unwrap();
        // let service = glonax::net::ActuatorService::new(net, 0x4A);
        let service = glonax::net::ActuatorService::new(0x4A);

        Self {
            net,
            // motion_device: Arc::new(Mutex::new(service)),
            motion_device: Arc::new(service),
            signal_writer,
        }
    }
}

#[tonic::async_trait]
impl glonax::transport::vehicle_management_server::VehicleManagement for VehicleManagemetService {
    /// Sends a motion command
    async fn motion_command(
        &self,
        request: Request<glonax::transport::Motion>,
    ) -> Result<Response<glonax::transport::Empty>, Status> {
        let motion = request.into_inner();

        log::trace!("Vehicle management: {}", motion);

        match motion.r#type() {
            glonax::transport::motion::MotionType::None => (),
            glonax::transport::motion::MotionType::StopAll => {
                self.net
                    .send_vectored(&self.motion_device.lock())
                    .await
                    .unwrap();
            }
            glonax::transport::motion::MotionType::ResumeAll => {
                self.net
                    .send_vectored(&self.motion_device.unlock())
                    .await
                    .unwrap();
            }
            glonax::transport::motion::MotionType::Change => {
                self.net
                    .send_vectored(
                        &self.motion_device.actuator_command(
                            motion
                                .changes
                                .iter()
                                .map(|changeset| (changeset.actuator as u8, changeset.value as i16))
                                .collect(),
                        ),
                    )
                    .await
                    .unwrap();
            }
        }

        Ok(Response::new(glonax::transport::Empty {}))
    }

    type ListenSignalStream = std::pin::Pin<
        Box<dyn futures_core::Stream<Item = Result<glonax::transport::Signal, Status>> + Send>,
    >;

    /// Listen for signal updates.
    async fn listen_signal(
        &self,
        _request: tonic::Request<glonax::transport::Empty>,
    ) -> Result<tonic::Response<Self::ListenSignalStream>, tonic::Status> {
        let mut signal_reader = self.signal_writer.subscribe();

        log::debug!("Client subscribed to signal updates");

        let output = async_stream::try_stream! {
            while let Ok(signal) = signal_reader.recv().await {
                log::trace!("Vehicle management: {:?}", signal);
                yield signal;
            }

            log::debug!("Client unsubscribed from signal updates");
        };

        Ok(Response::new(Box::pin(output) as Self::ListenSignalStream))
    }
}

// TODO: Even though the same confiig is used as for the motion command, the signal listeners
// should be able to listen on a different network.
async fn signal_listener(
    config: config::EcuConfig,
    writer: glonax::channel::BroadcastChannelWriter<glonax::transport::Signal>,
) {
    use glonax::channel::BroadcastSource;
    use glonax::net::{EngineManagementSystem, KueblerEncoderService};

    // TODO: Assign new network ID to each J1939 network.
    let network = J1939Network::new(&config.interface, DEVICE_NET_LOCAL_ADDR).unwrap();
    let mut router = glonax::net::Router::new(network);

    let mut engine_management_service = EngineManagementSystem::new(0x0);
    let mut encoder_list = vec![
        KueblerEncoderService::new(0x6A),
        KueblerEncoderService::new(0x6B),
        KueblerEncoderService::new(0x6C),
        KueblerEncoderService::new(0x6D),
    ];

    log::debug!("Listening for service signals");

    loop {
        if let Err(e) = router.listen().await {
            log::error!("{}", e);
        };

        if let Some(engine_message) = router.try_accept2(&mut engine_management_service) {
            log::trace!(
                "0x{:X?} » {}",
                router.frame_source().unwrap(),
                engine_message
            );

            engine_message.fetch(&writer)
        }

        for encoder in &mut encoder_list {
            if router.try_accept(encoder) {
                log::trace!("0x{:X?} » {}", router.frame_source().unwrap(), encoder);

                encoder.fetch(&writer);
            }
        }
    }
}

async fn daemonize(config: &config::EcuConfig) -> anyhow::Result<()> {
    let addr = config.address.parse()?;

    let runtime = glonax::RuntimeBuilder::from_config(config)?
        .with_shutdown()
        .build();

    let signal_writer = glonax::channel::broadcast_channel(10);

    runtime.spawn_background_task(signal_listener(config.clone(), signal_writer.clone()));

    Server::builder()
        .add_service(
            glonax::transport::vehicle_management_server::VehicleManagementServer::new(
                VehicleManagemetService::new(config.clone(), signal_writer),
            ),
        )
        .serve_with_shutdown(addr, async {
            runtime.shutdown_signal().recv().await.unwrap();
        })
        .await?;

    log::debug!("{} was shutdown gracefully", config.global().bin_name);

    Ok(())
}
