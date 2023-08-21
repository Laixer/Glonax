// Copyright (C) 2023 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use clap::Parser;

use na::Rotation3;
use nalgebra as na;

mod config;

trait EulerAngles {
    fn from_roll(roll: f32) -> Self;
    fn from_pitch(pitch: f32) -> Self;
    fn from_yaw(pitch: f32) -> Self;
}

impl EulerAngles for Rotation3<f32> {
    fn from_roll(roll: f32) -> Self {
        Rotation3::from_euler_angles(roll, 0.0, 0.0)
    }

    fn from_pitch(pitch: f32) -> Self {
        Rotation3::from_euler_angles(0.0, pitch, 0.0)
    }

    fn from_yaw(yaw: f32) -> Self {
        Rotation3::from_euler_angles(0.0, 0.0, yaw)
    }
}

#[derive(Parser)]
#[command(author = "Copyright (C) 2023 Laixer Equipment B.V.")]
#[command(version, propagate_version = true)]
#[command(about = "Glonax input daemon", long_about = None)]
struct Args {
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

    let (instance, ip) = net_recv_instance().await?;

    let address =
        std::net::SocketAddr::new(ip, glonax::constants::DEFAULT_NETWORK_PORT).to_string();

    let mut config = config::DumpConfig {
        address,
        instance,
        global: glonax::GlobalConfig::default(),
    };

    config.global.bin_name = env!("CARGO_BIN_NAME").to_string();
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

    daemonize(&mut config).await
}

async fn net_recv_instance() -> anyhow::Result<(glonax::core::Instance, std::net::IpAddr)> {
    let broadcast_addr = std::net::SocketAddrV4::new(
        std::net::Ipv4Addr::UNSPECIFIED,
        glonax::constants::DEFAULT_NETWORK_PORT,
    );

    let socket = tokio::net::UdpSocket::bind(broadcast_addr).await?;

    let mut buffer = [0u8; 1024];

    log::debug!("Waiting for instance announcement");

    loop {
        let (size, socket_addr) = socket.recv_from(&mut buffer).await?;
        if let Ok(frame) = glonax::transport::frame::Frame::try_from(&buffer[..size]) {
            if frame.message == glonax::transport::frame::FrameMessage::Instance {
                let instance =
                    glonax::core::Instance::try_from(&buffer[frame.payload_range()]).unwrap();

                log::info!("Instance announcement received: {}", instance);

                return Ok((instance, socket_addr.ip()));
            }
        }
    }
}

async fn daemonize(config: &config::DumpConfig) -> anyhow::Result<()> {
    use glonax::robot::{Device, DeviceType, Joint, JointType, RobotBuilder, RobotType};

    let robot = RobotBuilder::new(config.instance.id.clone(), RobotType::Excavator)
        .model(config.instance.model.clone())
        .name(config.instance.name.clone())
        .add_device(Device::new(
            "frame_encoder",
            0x6A,
            DeviceType::EncoderAbsoluteMultiTurn,
        ))
        .add_device(Device::new(
            "boom_encoder",
            0x6B,
            DeviceType::EncoderAbsoluteSingleTurn,
        ))
        .add_device(Device::new(
            "arm_encoder",
            0x6C,
            DeviceType::EncoderAbsoluteSingleTurn,
        ))
        .add_device(Device::new(
            "attachment_encoder",
            0x6D,
            DeviceType::EncoderAbsoluteSingleTurn,
        ))
        .add_joint(Joint::new("undercarriage", JointType::Fixed))
        .add_joint(Joint::new("frame", JointType::Continuous).origin_translation(0.0, 0.0, 1.295))
        .add_joint(
            Joint::new("boom", JointType::Revolute)
                .origin_translation(0.16, 0.0, 0.595)
                .origin_rotation(0.0, -1.0472, 0.0),
        )
        .add_joint(Joint::new("arm", JointType::Revolute).origin_translation(6.0, 0.0, 0.0))
        .add_joint(
            Joint::new("attachment", JointType::Revolute)
                .origin_translation(2.97, 0.0, 0.0)
                .origin_rotation(0.0, -0.962, 0.0),
        )
        .add_joint(Joint::new("effector", JointType::Fixed).origin_translation(1.5, 0.0, 0.0))
        .build();

    log::debug!("Configured: {}", robot);

    let point = na::Point3::new(0.0, 0.0, 0.0);

    let mut frame_yaw = 0.0;
    let mut boom_pitch = 0.0;
    let mut arm_pitch = 0.0;
    let mut attachment_pitch = 0.0;

    let frame_encoder = robot.device_by_name("frame_encoder").unwrap();
    let boom_encoder = robot.device_by_name("boom_encoder").unwrap();
    let arm_encoder = robot.device_by_name("arm_encoder").unwrap();
    let attachment_encoder = robot.device_by_name("attachment_encoder").unwrap();

    let frame_joint = robot.joint_by_name("frame").unwrap();
    let boom_joint = robot.joint_by_name("boom").unwrap();
    let arm_joint = robot.joint_by_name("arm").unwrap();
    let attachment_joint = robot.joint_by_name("attachment").unwrap();
    let effector_joint = robot.joint_by_name("effector").unwrap();

    let broadcast_addr = std::net::SocketAddrV4::new(
        std::net::Ipv4Addr::UNSPECIFIED,
        glonax::constants::DEFAULT_NETWORK_PORT,
    );

    let socket = tokio::net::UdpSocket::bind(broadcast_addr).await?;

    let mut buffer = [0u8; 1024];

    log::debug!("Listening for signals");

    while let Ok((size, _)) = socket.recv_from(&mut buffer).await {
        if let Ok(frame) = glonax::transport::frame::Frame::try_from(&buffer[..size]) {
            if frame.message == glonax::transport::frame::FrameMessage::Signal {
                let signal =
                    glonax::core::Signal::try_from(&buffer[frame.payload_range()]).unwrap();

                if let glonax::core::Metric::EncoderAbsAngle((node, value)) = signal.metric {
                    match node {
                        node if frame_encoder.id() == node => frame_yaw = value,
                        node if boom_encoder.id() == node => boom_pitch = value,
                        node if arm_encoder.id() == node => arm_pitch = value,
                        node if attachment_encoder.id() == node => attachment_pitch = value,
                        _ => {}
                    }

                    let link_point = (frame_joint.origin() * Rotation3::from_yaw(frame_yaw))
                        * (boom_joint.origin() * Rotation3::from_pitch(boom_pitch))
                        * (arm_joint.origin() * Rotation3::from_pitch(arm_pitch))
                        * (attachment_joint.origin() * Rotation3::from_pitch(attachment_pitch))
                        * point;

                    let effector_point = (frame_joint.origin() * Rotation3::from_yaw(frame_yaw))
                        * (boom_joint.origin() * Rotation3::from_pitch(boom_pitch))
                        * (arm_joint.origin() * Rotation3::from_pitch(arm_pitch))
                        * (attachment_joint.origin() * Rotation3::from_pitch(attachment_pitch))
                        * effector_joint.origin()
                        * point;

                    log::info!(
                            "F Angle: {:5.2}rad {:5.2}°\tB Angle: {:5.2}rad {:5.2}°\tA Angle: {:5.2}rad {:5.2}°\tT Angle: {:5.2}rad {:5.2}°\tLink: [{:.2}, {:.2}, {:.2}]\tEffector: [{:.2}, {:.2}, {:.2}]",
                            frame_yaw,
                            glonax::core::rad_to_deg(frame_yaw),
                            boom_pitch,
                            glonax::core::rad_to_deg(boom_pitch),
                            arm_pitch,
                            glonax::core::rad_to_deg(arm_pitch),
                            attachment_pitch,
                            glonax::core::rad_to_deg(attachment_pitch),
                            link_point.x, link_point.y, link_point.z,
                            effector_point.x, effector_point.y, effector_point.z
                        );
                }
            }
        }
    }

    Ok(())
}
