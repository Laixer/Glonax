// Copyright (C) 2023 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use clap::Parser;

use na::Rotation3;
use nalgebra as na;

mod config;

// TODO: move to core::algorithm
struct InverseKinematics {
    l1: f32,
    l2: f32,
}

impl InverseKinematics {
    fn new(l1: f32, l2: f32) -> Self {
        Self { l1, l2 }
    }

    fn solve(&self, target: nalgebra::Point3<f32>) -> Option<(f32, f32, f32)> {
        let l4 = (target.x.powi(2) + target.z.powi(2)).sqrt();
        let l5 = (l4.powi(2) + target.y.powi(2)).sqrt();

        let theta_1 = target.z.atan2(target.x);

        let theta_2 = target.y.atan2(l4)
            + ((self.l1.powi(2) + l5.powi(2) - self.l2.powi(2)) / (2.0 * self.l1 * l5)).acos();

        let theta_3 = std::f32::consts::PI
            - ((self.l1.powi(2) + self.l2.powi(2) - l5.powi(2)) / (2.0 * self.l1 * self.l2)).acos();

        if l5 >= self.l1 + self.l2 {
            None
        } else {
            Some((theta_1, theta_2, theta_3))
        }
    }
}

struct MotionProfile {
    scale: f32,
    offset: f32,
    lower_bound: f32,
    inverse: bool,
}

impl MotionProfile {
    pub fn new(scale: f32, offset: f32, lower_bound: f32, inverse: bool) -> Self {
        Self {
            scale,
            offset,
            lower_bound,
            inverse,
        }
    }

    pub fn power(&self, value: f32) -> i32 {
        if self.inverse {
            self.proportional_power_inverse(value)
        } else {
            self.proportional_power(value)
        }
    }

    pub fn proportional_power(&self, value: f32) -> i32 {
        if value.abs() > self.lower_bound {
            let power = self.offset + (value.abs() * self.scale).min(32_767.0 - self.offset);
            if value < 0.0 {
                -power as i32
            } else {
                power as i32
            }
        } else {
            0
        }
    }

    pub fn proportional_power_inverse(&self, value: f32) -> i32 {
        if value.abs() > self.lower_bound {
            let power = value * self.scale;

            if value > 0.0 {
                (-(power.min(32_767.0 - self.offset)) - self.offset) as i32
            } else {
                (power.max(-(32_767.0 - self.offset)) + self.offset) as i32
            }
        } else {
            0
        }
    }
}

// class MotionProfile:
//     def __init__(self, scale, offset, lower_bound, inverse):
//         self.scale = scale
//         self.offset = offset
//         self.lower_bound = lower_bound
//         self.inverse = inverse

//     def power(self, value) -> int:
//         if self.inverse:
//             return int(self.proportional_power_inverse(value))
//         return int(self.proportional_power(value))

//     def proportional_power(self, value) -> int:
//         if abs(value) > self.lower_bound:
//             power = self.offset + min((abs(value) * self.scale), 32_767 - self.offset)
//             if value < 0:
//                 return -power
//             else:
//                 return power
//         else:
//             return 0

//     def proportional_power_inverse(self, value) -> int:
//         if abs(value) > self.lower_bound:
//             power = value * self.scale

//             if value > 0:
//                 return max(-power, -(32_767 - self.offset)) - self.offset
//             else:
//                 return min(-power, 32_767 - self.offset) + self.offset
//         else:
//             return 0

// TODO: move to core
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

// TODO: Move to runtime
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
            Joint::new("boom", JointType::Revolute).origin_translation(0.16, 0.0, 0.595), // .origin_rotation(0.0, -1.0472, 0.0),
        )
        .add_joint(Joint::new("arm", JointType::Revolute).origin_translation(6.0, 0.0, 0.0))
        .add_joint(
            Joint::new("attachment", JointType::Revolute).origin_translation(2.97, 0.0, 0.0), // .origin_rotation(0.0, -0.962, 0.0),
        )
        .add_joint(Joint::new("effector", JointType::Fixed).origin_translation(1.5, 0.0, 0.0))
        .build();

    log::debug!("Configured: {}", robot);

    let frame_joint = robot.joint_by_name("frame").unwrap();
    let boom_joint = robot.joint_by_name("boom").unwrap();
    let arm_joint = robot.joint_by_name("arm").unwrap();
    let attachment_joint = robot.joint_by_name("attachment").unwrap();
    let effector_joint = robot.joint_by_name("effector").unwrap();

    let frame_encoder = robot.device_by_name("frame_encoder").unwrap();
    let boom_encoder = robot.device_by_name("boom_encoder").unwrap();
    let arm_encoder = robot.device_by_name("arm_encoder").unwrap();
    let attachment_encoder = robot.device_by_name("attachment_encoder").unwrap();

    let frame_power = MotionProfile::new(7000.0, 12000.0, 0.01, false);
    let boom_power = MotionProfile::new(15000.0, 12000.0, 0.01, false);
    let arm_power = MotionProfile::new(15000.0, 12000.0, 0.01, true);
    let attachment_power = MotionProfile::new(15000.0, 12000.0, 0.01, false);

    let mut perception_chain = glonax::robot::Chain::new();
    perception_chain.add_joint(frame_joint.clone());
    perception_chain.add_joint(boom_joint.clone());
    perception_chain.add_joint(arm_joint.clone());
    perception_chain.add_joint(attachment_joint.clone());

    let mut projection_chain = glonax::robot::Chain::new();
    projection_chain.add_joint(frame_joint.clone());
    projection_chain.add_joint(boom_joint.clone());
    projection_chain.add_joint(arm_joint.clone());
    projection_chain.add_joint(attachment_joint.clone());

    let solver = InverseKinematics::new(6.0, 2.97);

    let target = na::Point3::new(5.21, 0.0, 0.0);

    let (p_frame_yaw, p_boom_pitch, p_arm_pitch) = solver.solve(target).unwrap();

    log::debug!(
        "Projection angles: {:.2} {:.2} {:.2}",
        p_frame_yaw,
        p_boom_pitch,
        p_arm_pitch
    );

    projection_chain.set_joint_positions(vec![
        Rotation3::from_yaw(p_frame_yaw),
        Rotation3::from_pitch(-p_boom_pitch),
        Rotation3::from_pitch(p_arm_pitch),
    ]);

    let projection_point = projection_chain.world_transformation() * na::Point3::origin();

    log::debug!(
        "Projection point: [{:.2}, {:.2}, {:.2}]",
        projection_point.x,
        projection_point.y,
        projection_point.z
    );

    let mut client = glonax::transport::ConnectionOptions::new()
        .read(false)
        .write(true)
        .connect(
            config.address.to_owned(),
            config.global.bin_name.to_string(),
        )
        .await?;

    log::info!("Connected to {}", config.address);

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
                        node if frame_encoder.id() == node => {
                            perception_chain.set_joint_position("frame", Rotation3::from_yaw(value))
                        }
                        node if boom_encoder.id() == node => perception_chain
                            .set_joint_position("boom", Rotation3::from_pitch(value)),
                        node if arm_encoder.id() == node => {
                            perception_chain.set_joint_position("arm", Rotation3::from_pitch(value))
                        }
                        node if attachment_encoder.id() == node => perception_chain
                            .set_joint_position("attachment", Rotation3::from_pitch(value)),
                        _ => {}
                    }

                    if perception_chain
                        .joint_by_name("frame")
                        .unwrap()
                        .rotation()
                        .axis_angle()
                        .is_some()
                        && perception_chain
                            .joint_by_name("boom")
                            .unwrap()
                            .rotation()
                            .axis_angle()
                            .is_some()
                        && perception_chain
                            .joint_by_name("arm")
                            .unwrap()
                            .rotation()
                            .axis_angle()
                            .is_some()
                    {
                        let perception_point =
                            perception_chain.world_transformation() * na::Point3::origin();

                        let frame_rot_angle = perception_chain
                            .joint_by_name("frame")
                            .unwrap()
                            .rotation_angle()
                            .unwrap_or(0.0);
                        let boom_rot_angle = perception_chain
                            .joint_by_name("boom")
                            .unwrap()
                            .rotation_angle()
                            .unwrap_or(0.0);
                        let arm_rot_angle = perception_chain
                            .joint_by_name("arm")
                            .unwrap()
                            .rotation_angle()
                            .unwrap_or(0.0);

                        log::info!(
                            "Frame {:5.2}rad {:5.2}° Boom {:5.2}rad {:5.2}° Arm {:5.2}rad {:5.2}°\tPerception point: [{:.2}, {:.2}, {:.2}]",
                            frame_rot_angle,
                            glonax::core::rad_to_deg(frame_rot_angle),
                            boom_rot_angle,
                            glonax::core::rad_to_deg(boom_rot_angle),
                            arm_rot_angle,
                            glonax::core::rad_to_deg(arm_rot_angle),
                            perception_point.x,
                            perception_point.y,
                            perception_point.z
                        );
                    }

                    let error = projection_chain.vector_error(&perception_chain);
                    log::debug!(
                        "Euler error  [{:.2}, {:.2}, {:.2}]",
                        error.x,
                        error.y,
                        error.z
                    );

                    let error_chain = perception_chain.error(&projection_chain);

                    let mut done = true;
                    for (joint, rot_angle) in error_chain
                        .iter()
                        .filter(|(_, e)| e.axis().is_some())
                        .map(|(j, e)| (j, e.angle()))
                    {
                        log::debug!(
                            " - Abs. {:10} {:5.2}rad {:5.2}°",
                            joint.name(),
                            rot_angle,
                            glonax::core::rad_to_deg(rot_angle)
                        );

                        if rot_angle.abs() > 0.01 {
                            done = false;
                        }
                    }

                    log::debug!("Done: {}", done);

                    if !done {
                        let mut motion_list = vec![];

                        for (joint, rot_error) in &error_chain {
                            if joint.name() == "frame" && rot_error.axis().is_some() {
                                let axis = rot_error.axis().unwrap();
                                let axis_rot_error_angle = (axis.x * rot_error.angle())
                                    + (axis.y * rot_error.angle())
                                    + (axis.z * rot_error.angle());
                                let axis_rot_error_power = frame_power.power(axis_rot_error_angle);

                                if rot_error.angle() > 0.01 {
                                    log::debug!(
                                        " * Frame error angle        {:5.2}rad {:5.2}° Power: {}",
                                        axis_rot_error_angle,
                                        glonax::core::rad_to_deg(axis_rot_error_angle),
                                        axis_rot_error_power
                                    );

                                    motion_list.push(glonax::core::Motion::new(
                                        glonax::core::Actuator::Slew,
                                        axis_rot_error_power as i16,
                                    ));
                                } else {
                                    log::debug!(" * Frame error angle        -");
                                }
                            } else if joint.name() == "boom" && rot_error.axis().is_some() {
                                let axis = rot_error.axis().unwrap();
                                let axis_rot_error_angle = (axis.x * rot_error.angle())
                                    + (axis.y * rot_error.angle())
                                    + (axis.z * rot_error.angle());
                                let axis_rot_error_power = frame_power.power(axis_rot_error_angle);

                                if rot_error.angle() > 0.01 {
                                    log::debug!(
                                        " * Boom error angle         {:5.2}rad {:5.2}° Power: {}",
                                        axis_rot_error_angle,
                                        glonax::core::rad_to_deg(axis_rot_error_angle),
                                        axis_rot_error_power
                                    );

                                    motion_list.push(glonax::core::Motion::new(
                                        glonax::core::Actuator::Boom,
                                        axis_rot_error_power as i16,
                                    ));
                                } else {
                                    log::debug!(" * Boom error angle         -");
                                }
                            } else if joint.name() == "arm" && rot_error.axis().is_some() {
                                let axis = rot_error.axis().unwrap();
                                let axis_rot_error_angle = (axis.x * rot_error.angle())
                                    + (axis.y * rot_error.angle())
                                    + (axis.z * rot_error.angle());
                                let axis_rot_error_power = frame_power.power(axis_rot_error_angle);

                                if rot_error.angle() > 0.01 {
                                    log::debug!(
                                        " * Arm error angle          {:5.2}rad {:5.2}° Power: {}",
                                        axis_rot_error_angle,
                                        glonax::core::rad_to_deg(axis_rot_error_angle),
                                        axis_rot_error_power
                                    );

                                    motion_list.push(glonax::core::Motion::new(
                                        glonax::core::Actuator::Arm,
                                        axis_rot_error_power as i16,
                                    ));
                                } else {
                                    log::debug!(" * Arm error angle          -");
                                }
                            } else if joint.name() == "attachment" && rot_error.axis().is_some() {
                                let axis = rot_error.axis().unwrap();
                                let axis_rot_error_angle = (axis.x * rot_error.angle())
                                    + (axis.y * rot_error.angle())
                                    + (axis.z * rot_error.angle());
                                let axis_rot_error_power = frame_power.power(axis_rot_error_angle);

                                if rot_error.angle() > 0.01 {
                                    log::debug!(
                                        " * Attachment error angle   {:5.2}rad {:5.2}° Power: {}",
                                        axis_rot_error_angle,
                                        glonax::core::rad_to_deg(axis_rot_error_angle),
                                        axis_rot_error_power
                                    );

                                    motion_list.push(glonax::core::Motion::new(
                                        glonax::core::Actuator::Attachment,
                                        axis_rot_error_power as i16,
                                    ));
                                } else {
                                    log::debug!(" * Attachment error angle   -");
                                }
                            }
                        }

                        for motion in motion_list {
                            client.send_motion(motion).await?;
                        }
                    }

                    // let link_point = (frame_joint.origin() * Rotation3::from_yaw(frame_yaw))
                    //     * (boom_joint.origin() * Rotation3::from_pitch(boom_pitch))
                    //     * (arm_joint.origin() * Rotation3::from_pitch(arm_pitch))
                    //     * (attachment_joint.origin() * Rotation3::from_pitch(attachment_pitch))
                    //     * point;

                    // let effector_point = (frame_joint.origin() * Rotation3::from_yaw(frame_yaw))
                    //     * (boom_joint.origin() * Rotation3::from_pitch(boom_pitch))
                    //     * (arm_joint.origin() * Rotation3::from_pitch(arm_pitch))
                    //     * (attachment_joint.origin() * Rotation3::from_pitch(attachment_pitch))
                    //     * effector_joint.origin()
                    //     * point;
                }
            }
        }
    }

    // if let Some((p_frame_yaw, p_boom_pitch, p_arm_pitch)) = solver.solve(target) {
    //     let error = projection_chain.vector_error(&perception_chain);
    //     log::debug!("Error: [{:.2}, {:.2}, {:.2}]", error.x, error.y, error.z);

    //     let error_chain = perception_chain.error(&projection_chain);

    //     let mut done = true;
    //     for (joint, rot_angle) in error_chain
    //         .iter()
    //         .filter(|(_, e)| e.axis().is_some())
    //         .map(|(j, e)| (j, e.angle()))
    //     {
    //         log::debug!("{} \t=> {:?}", joint.name(), rot_angle);

    //         if rot_angle.abs() > 0.01 {
    //             done = false;
    //         }
    //     }

    //     log::debug!("Done: {}", done);

    // let mut motion_list = vec![];

    // for (joint, rot_error) in &error_chain {
    // log::debug!("{} \t=> {:?}", joint.name(), rot_error.axis_angle());

    // if joint.name() == "frame" && rot_error.axis().is_some() {
    //     // let power = frame_power.power(rot_error.angle());
    //     // log::debug!("Frame power: {}", power);

    //     let axis_rot_error_angle = rot_error.axis().unwrap().z * rot_error.angle();

    //     let axis_rot_error_power = frame_power.power(axis_rot_error_angle);

    //     log::debug!("Frame power: {:?}", rot_error.axis_angle());
    //     log::debug!("Frame power: {:?}", axis_rot_error_angle);
    //     log::debug!("Frame power: {:?}", axis_rot_error_power);

    //     motion_list.push(glonax::core::Motion::new(
    //         glonax::core::Actuator::Slew,
    //         axis_rot_error_power as i16,
    //     ));
    // }
    // } else if joint.name() == "boom" {
    //     let power = boom_power.power(rot_error.angle());
    //     log::debug!("Boom power: {}", power);
    // } else if joint.name() == "arm" {
    //     let power = arm_power.power(rot_error.angle());
    //     log::debug!("Arm power: {}", power);
    // } else if joint.name() == "attachment" {
    //     let power = attachment_power.power(rot_error.angle());
    //     log::debug!("Attachment power: {}", power);
    // }
    // }
    // } else {
    //     println!("Target out of range");
    // }

    return Ok(());
}
