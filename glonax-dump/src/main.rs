// Copyright (C) 2023 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use clap::Parser;

use na::{Rotation2, Rotation3};
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
        let local_z = target.z - 0.595 - 1.295;
        log::debug!("Local Z:            {:.2}", local_z);

        let theta_1 = target.y.atan2(target.x);

        let offset = 0.16;
        let offset_x = offset * theta_1.cos();
        // let offset_x = 0.0;
        // let offset_y = 0.0;
        let offset_y = offset * theta_1.sin();

        log::debug!("Vector offset:      [{:.2}, {:.2}]", offset_x, offset_y);

        let local_x = target.x - offset_x;
        let local_y = target.y - offset_y;
        log::debug!("Local X:            {:.2}", local_x);
        log::debug!("Local Y:            {:.2}", local_y);

        // L4 is the leg between the origin and the target projected on the XY plane (ground).
        let l4 = (local_x.powi(2) + local_y.powi(2)).sqrt();
        log::debug!("Vector length L4:   {:.2}", l4);
        // L5 is the leg between the origin and the target (vector).
        let l5 = (l4.powi(2) + local_z.powi(2)).sqrt();
        log::debug!("Vector length L5:   {:.2}", l5);

        let theta_2p1 = local_z.atan2(l4);
        log::debug!(
            "theta_2p1:         {:5.2}rad {:5.2}° ",
            theta_2p1,
            glonax::core::rad_to_deg(theta_2p1)
        );
        let theta_2p2 =
            ((self.l1.powi(2) + l5.powi(2) - self.l2.powi(2)) / (2.0 * self.l1 * l5)).acos();
        log::debug!(
            "theta_2p2:         {:5.2}rad {:5.2}° ",
            theta_2p2,
            glonax::core::rad_to_deg(theta_2p2)
        );

        let theta_2 = local_z.atan2(l4)
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
                (-power.max(-(32_767.0 - self.offset)) - self.offset) as i32
            } else {
                (-power.min(32_767.0 - self.offset) + self.offset) as i32
            }
        } else {
            0
        }
    }
}

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

    let kinematic_epsilon = 0.0001;
    let angular_tolerance = 0.01;

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
                .origin_rotation(0.0, glonax::core::deg_to_rad(-59.35), 0.0),
        )
        .add_joint(Joint::new("arm", JointType::Revolute).origin_translation(6.0, 0.0, 0.0))
        .add_joint(
            Joint::new("attachment", JointType::Revolute)
                .origin_translation(2.97, 0.0, 0.0)
                .origin_rotation(0.0, glonax::core::deg_to_rad(-55.0), 0.0),
        )
        .add_joint(Joint::new("effector", JointType::Fixed).origin_translation(1.5, 0.0, 0.0))
        .build();

    log::debug!("Configured: {}", robot);

    let frame_joint = robot.joint_by_name("frame").unwrap();
    let boom_joint = robot.joint_by_name("boom").unwrap();
    let arm_joint = robot.joint_by_name("arm").unwrap();
    let attachment_joint = robot.joint_by_name("attachment").unwrap();
    // let effector_joint = robot.joint_by_name("effector").unwrap();

    let frame_encoder = robot.device_by_name("frame_encoder").unwrap();
    let boom_encoder = robot.device_by_name("boom_encoder").unwrap();
    let arm_encoder = robot.device_by_name("arm_encoder").unwrap();
    let attachment_encoder = robot.device_by_name("attachment_encoder").unwrap();

    let frame_power = MotionProfile::new(7_000.0, 12_000.0, 0.01, false);
    let boom_power = MotionProfile::new(15_000.0, 12_000.0, 0.01, false);
    let arm_power = MotionProfile::new(15_000.0, 12_000.0, 0.01, true);
    let attachment_power = MotionProfile::new(15_000.0, 12_000.0, 0.01, false);

    let mut perception_chain = glonax::robot::Chain::new();
    perception_chain
        .add_joint(frame_joint.clone())
        .add_joint(boom_joint.clone())
        .add_joint(arm_joint.clone())
        .add_joint(attachment_joint.clone());

    let mut projection_chain = glonax::robot::Chain::new();
    projection_chain
        .add_joint(frame_joint.clone())
        .add_joint(boom_joint.clone())
        .add_joint(arm_joint.clone())
        .add_joint(attachment_joint.clone());

    ///////////////////////////////////////////

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

    // log::debug!("Listening for signals");

    ///////////////////////////////////////////
    ///////////////////////////////////////////

    let targets = [na::Point3::new(5.21 + 0.16, 2.50, 1.295 + 0.595)];

    // let targets = [
    //     na::Point3::new(5.56, 0.00, 1.65),
    //     na::Point3::new(6.27, 0.00, 3.58),
    //     na::Point3::new(7.63, 0.00, 4.45),
    //     na::Point3::new(8.05, 0.00, 2.19),
    //     na::Point3::new(7.14, 0.00, 1.44),
    //     na::Point3::new(5.85, 0.00, 1.85),
    //     na::Point3::new(3.55, 4.60, 2.58),
    //     na::Point3::new(4.85, 6.26, 1.96),
    //     na::Point3::new(6.27, 0.00, 3.58),
    // ];

    for target in targets {
        projection_chain.reset();

        client.send_motion(glonax::core::Motion::ResumeAll).await?;

        log::debug!(
            "Target point:       [{:.2}, {:.2}, {:.2}]",
            target.x,
            target.y,
            target.z
        );

        ///////////////////////////////////
        log::debug!("IK");

        let rot = na::Rotation2::rotation_between(&na::Vector2::x(), &target.xy().coords);

        log::debug!(
            "Angle θ1           {:5.2}rad {:5.2}°",
            rot.angle(),
            glonax::core::rad_to_deg(rot.angle() as f32)
        );

        let offset = rot * na::Point2::new(0.16, 0.0);
        log::debug!("Vector point:       [{:.3}, {:.3}]", offset.x, offset.y);
        let lv = na::distance(&na::Point2::new(0.0, 0.0), &offset);
        log::debug!("Vector L:           {:.2}", lv);

        let l1 = na::distance(&na::Point2::new(0.0, 0.0), &target.xy());
        log::debug!("L1 distance:        {:.2}", l1);
        let l4 = na::distance(&offset, &target.xy());
        log::debug!("L4 distance:        {:.2}", l4);

        let offset = na::Point3::new(offset.x, offset.y, 0.595 + 1.295);
        let l5 = na::distance(&offset, &target);
        log::debug!("L5 distance:        {:.2}", l5);

        // let q = target - offset;
        // log::debug!("Vector point:       [{:.3}, {:.3}, {:.3}]", q.x, q.y, q.z);

        // let rot = na::Rotation3::rotation_between(&na::Vector3::new(1.0,0.0, 0.0), &na::Vector3::new(5.226, 2.491, 0.010) ).unwrap();
        // log::debug!(
        //     "Angle θ2p1          {:?} {:?} {:?}",
        //     rot.axis(),
        //     rot.angle(),
        //     glonax::core::rad_to_deg(rot.angle() as f32)
        // );

        // let q = target - offset;

        // log::debug!("Vector point:       [{:.3}, {:.3}, {:.3}]", q.x, q.y, q.z);

        // let rot = na::Rotation2::rotation_between(&na::Vector2::new(1.0,0.01), &na::Vector2::new(5.226,0.2) );
        // log::debug!(
        //     "Angle θ2p1          {:?} {:?}",
        //     rot.angle(),
        //     glonax::core::rad_to_deg(rot.angle() as f32)
        // );

        // let l5 = na::distance(&na::Point2::new(0.16, 0.0), &target.xz());
        // log::debug!("L5 distance: {:.2}", l5);

        // let theta_2p2 = ((6.0_f32.powi(2) + (l5 as f32).powi(2) - 2.97_f32.powi(2))
        //     / (2.0 * 6.0_f32 * (l5 as f32)))
        //     .acos();

        // log::debug!(
        //     "Angle (θp2): {:5.2}rad {:5.2}°",
        //     theta_2p2,
        //     glonax::core::rad_to_deg(theta_2p2)
        // );

        // let theta_3 = std::f32::consts::PI
        //     - ((6.0_f32.powi(2) + 2.97_f32.powi(2) - (l5 as f32).powi(2)) / (2.0 * 6.0_f32 * 2.97_f32))
        //         .acos();

        // log::debug!(
        //     "Angle (θ3): {:5.2}rad {:5.2}°",
        //     theta_3,
        //     glonax::core::rad_to_deg(theta_3)
        // );

        // return Ok(());

        ///////////////////////////////////
        log::debug!("IK2");

        let solver = InverseKinematics::new(6.0, 2.97);

        let (p_frame_yaw, p_boom_pitch, p_arm_pitch) = solver.solve(target).unwrap();
        log::debug!(
            "IK angles:         {:5.2}rad {:5.2}° {:5.2}rad {:5.2}°  {:5.2}rad {:5.2}°",
            p_frame_yaw,
            glonax::core::rad_to_deg(p_frame_yaw),
            p_boom_pitch,
            glonax::core::rad_to_deg(p_boom_pitch),
            p_arm_pitch,
            glonax::core::rad_to_deg(p_arm_pitch)
        );

        projection_chain.set_joint_positions(vec![
            Rotation3::from_yaw(p_frame_yaw),
            Rotation3::from_pitch((-p_boom_pitch) + glonax::core::deg_to_rad(59.35)),
            Rotation3::from_pitch(p_arm_pitch),
        ]);

        // log::debug!(
        //     "Projection angles: {:5.2}rad {:5.2}° {:5.2}rad {:5.2}°  {:5.2}rad {:5.2}°",
        //     projection_chain
        //         .joint_rotation_angle("frame")
        //         .unwrap_or_default(),
        //     glonax::core::rad_to_deg(
        //         projection_chain
        //             .joint_rotation_angle("frame")
        //             .unwrap_or_default()
        //     ),
        //     projection_chain
        //         .joint_rotation_angle("boom")
        //         .unwrap_or_default(),
        //     glonax::core::rad_to_deg(
        //         projection_chain
        //             .joint_rotation_angle("boom")
        //             .unwrap_or_default()
        //     ),
        //     projection_chain
        //         .joint_rotation_angle("arm")
        //         .unwrap_or_default(),
        //     glonax::core::rad_to_deg(
        //         projection_chain
        //             .joint_rotation_angle("arm")
        //             .unwrap_or_default()
        //     ),
        // );

        let projection_point = projection_chain.world_transformation() * na::Point3::origin();
        if target.coords.norm() - projection_point.coords.norm() > kinematic_epsilon {
            log::error!(
                "Projection point:   [{:.2}, {:.2}, {:.2}]",
                projection_point.x,
                projection_point.y,
                projection_point.z
            );
            return Err(anyhow::anyhow!("IK error"));
        }

        log::info!("Press enter to continue");
        std::io::stdin().read_line(&mut String::new())?;

        // return Ok(());

        while let Ok((size, _)) = socket.recv_from(&mut buffer).await {
            if let Ok(frame) = glonax::transport::frame::Frame::try_from(&buffer[..size]) {
                if frame.message == glonax::transport::frame::FrameMessage::Signal {
                    let signal =
                        glonax::core::Signal::try_from(&buffer[frame.payload_range()]).unwrap();

                    if let glonax::core::Metric::EncoderAbsAngle((node, value)) = signal.metric {
                        match node {
                            node if frame_encoder.id() == node => {
                                perception_chain
                                    .set_joint_position("frame", Rotation3::from_yaw(value));
                            }
                            node if boom_encoder.id() == node => {
                                perception_chain
                                    .set_joint_position("boom", Rotation3::from_pitch(value));
                            }
                            node if arm_encoder.id() == node => {
                                perception_chain
                                    .set_joint_position("arm", Rotation3::from_pitch(value));
                            }
                            node if attachment_encoder.id() == node => {
                                perception_chain
                                    .set_joint_position("attachment", Rotation3::from_pitch(value));
                            }
                            _ => {}
                        }

                        if !perception_chain.is_ready() {
                            continue;
                        }

                        log::debug!("Perception chain: {:?}", perception_chain);

                        let distance = projection_chain.distance(&perception_chain);
                        log::debug!("Target distance:        {:.2}m", distance);

                        let error_chain = perception_chain.error(&projection_chain);

                        let mut done = true;
                        for (_, rot_angle) in error_chain
                            .iter()
                            .filter(|(_, e)| e.axis().is_some())
                            .map(|(j, e)| (j, e.angle()))
                        {
                            if rot_angle.abs() > angular_tolerance {
                                done = false;
                            }
                        }

                        if done {
                            client.send_motion(glonax::core::Motion::StopAll).await?;

                            log::info!("Press enter to continue");
                            std::io::stdin().read_line(&mut String::new())?;
                            break;
                        }

                        let mut motion_list = vec![];

                        for (joint_name, rot_error) in
                            error_chain.iter().filter(|(_, e)| e.axis().is_some())
                        {
                            if joint_name == &&"frame" {
                                let axis = rot_error.axis().unwrap();
                                let axis_rot_error_angle = (axis.x * rot_error.angle())
                                    + (axis.y * rot_error.angle())
                                    + (axis.z * rot_error.angle());

                                let axis_rot_error_angle_optimized =
                                    glonax::core::geometry::shortest_rotation(axis_rot_error_angle);
                                let axis_rot_error_power =
                                    frame_power.power(axis_rot_error_angle_optimized);

                                log::debug!(
                                    " ⇒ {:<15} Error: {:5.2}rad {:6.2}°   Power: {:6}   State: {}",
                                    "Frame",
                                    axis_rot_error_angle_optimized,
                                    glonax::core::rad_to_deg(axis_rot_error_angle_optimized),
                                    axis_rot_error_power,
                                    if rot_error.angle() > angular_tolerance {
                                        "Moving"
                                    } else {
                                        "Locked"
                                    }
                                );

                                if rot_error.angle() > angular_tolerance {
                                    motion_list.push(glonax::core::Motion::new(
                                        glonax::core::Actuator::Slew,
                                        axis_rot_error_power as i16,
                                    ));
                                } else {
                                    motion_list.push(glonax::core::Motion::new(
                                        glonax::core::Actuator::Slew,
                                        glonax::core::Motion::POWER_NEUTRAL,
                                    ));
                                }
                            } else if joint_name == &&"boom" {
                                let axis = rot_error.axis().unwrap();
                                let axis_rot_error_angle = (axis.x * rot_error.angle())
                                    + (axis.y * rot_error.angle())
                                    + (axis.z * rot_error.angle());
                                let axis_rot_error_power = boom_power.power(axis_rot_error_angle);

                                log::debug!(
                                    " ⇒ {:<15} Error: {:5.2}rad {:6.2}°   Power: {:6}   State: {}",
                                    "Boom",
                                    axis_rot_error_angle,
                                    glonax::core::rad_to_deg(axis_rot_error_angle),
                                    axis_rot_error_power,
                                    if rot_error.angle() > angular_tolerance {
                                        "Moving"
                                    } else {
                                        "Locked"
                                    }
                                );

                                if rot_error.angle() > angular_tolerance {
                                    motion_list.push(glonax::core::Motion::new(
                                        glonax::core::Actuator::Boom,
                                        axis_rot_error_power as i16,
                                    ));
                                } else {
                                    motion_list.push(glonax::core::Motion::new(
                                        glonax::core::Actuator::Boom,
                                        glonax::core::Motion::POWER_NEUTRAL,
                                    ));
                                }
                            } else if joint_name == &&"arm" {
                                let axis = rot_error.axis().unwrap();
                                let axis_rot_error_angle = (axis.x * rot_error.angle())
                                    + (axis.y * rot_error.angle())
                                    + (axis.z * rot_error.angle());
                                let axis_rot_error_power = arm_power.power(axis_rot_error_angle);

                                log::debug!(
                                    " ⇒ {:<15} Error: {:5.2}rad {:6.2}°   Power: {:6}   State: {}",
                                    "Arm",
                                    axis_rot_error_angle,
                                    glonax::core::rad_to_deg(axis_rot_error_angle),
                                    axis_rot_error_power,
                                    if rot_error.angle() > angular_tolerance {
                                        "Moving"
                                    } else {
                                        "Locked"
                                    }
                                );

                                if rot_error.angle() > angular_tolerance {
                                    motion_list.push(glonax::core::Motion::new(
                                        glonax::core::Actuator::Arm,
                                        axis_rot_error_power as i16,
                                    ));
                                } else {
                                    motion_list.push(glonax::core::Motion::new(
                                        glonax::core::Actuator::Arm,
                                        glonax::core::Motion::POWER_NEUTRAL,
                                    ));
                                }
                            } else if joint_name == &&"attachment" {
                                let axis = rot_error.axis().unwrap();
                                let axis_rot_error_angle = (axis.x * rot_error.angle())
                                    + (axis.y * rot_error.angle())
                                    + (axis.z * rot_error.angle());
                                let axis_rot_error_power =
                                    attachment_power.power(axis_rot_error_angle);

                                log::debug!(
                                    " ⇒ {:<15} Error: {:5.2}rad {:6.2}°   Power: {:6}   State: {}",
                                    "Attachment",
                                    axis_rot_error_angle,
                                    glonax::core::rad_to_deg(axis_rot_error_angle),
                                    axis_rot_error_power,
                                    if rot_error.angle() > angular_tolerance {
                                        "Moving"
                                    } else {
                                        "Locked"
                                    }
                                );

                                if rot_error.angle() > angular_tolerance {
                                    motion_list.push(glonax::core::Motion::new(
                                        glonax::core::Actuator::Attachment,
                                        axis_rot_error_power as i16,
                                    ));
                                } else {
                                    motion_list.push(glonax::core::Motion::new(
                                        glonax::core::Actuator::Attachment,
                                        glonax::core::Motion::POWER_NEUTRAL,
                                    ));
                                }
                            }
                        }

                        for motion in motion_list {
                            client.send_motion(motion).await?;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
