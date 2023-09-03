// Copyright (C) 2023 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use clap::Parser;

use na::{Isometry3, Rotation3};
use nalgebra as na;
use parry3d::query::PointQuery;

mod config;

struct Target {
    point: na::Point3<f32>,
    orientation: na::Rotation3<f32>,
}

impl Target {
    fn new(point: na::Point3<f32>, orientation: na::Rotation3<f32>) -> Self {
        Self { point, orientation }
    }
}

// TODO: move to core::algorithm
struct InverseKinematics {
    l1: f32,
    l2: f32,
}

impl InverseKinematics {
    fn new(l1: f32, l2: f32) -> Self {
        Self { l1, l2 }
    }

    fn solve(&self, target: &Target) -> std::result::Result<(f32, f32, f32, Option<f32>), ()> {
        use glonax::core::geometry::law_of_cosines;

        let local_z = target.point.z - 0.595 - 1.295;
        log::debug!("Local Z:            {:.2}", local_z);

        let theta_1 = target.point.y.atan2(target.point.x);

        let offset = 0.16;
        let offset_x = offset * theta_1.cos();
        // let offset_x = 0.0;
        // let offset_y = 0.0;
        let offset_y = offset * theta_1.sin();

        log::debug!("Vector offset:      ({:.2}, {:.2})", offset_x, offset_y);

        let local_x = target.point.x - offset_x;
        let local_y = target.point.y - offset_y;
        log::debug!("Local X:            {:.2}", local_x);
        log::debug!("Local Y:            {:.2}", local_y);

        // L4 is the leg between the origin and the target projected on the XY plane (ground).
        let l4 = (local_x.powi(2) + local_y.powi(2)).sqrt();
        log::debug!("Vector length L4:   {:.2}", l4);
        // L5 is the leg between the origin and the target (vector).
        let l5 = (l4.powi(2) + local_z.powi(2)).sqrt();
        log::debug!("Vector length L5:   {:.2}", l5);

        if l5 >= self.l1 + self.l2 {
            return Err(());
        }

        let theta_2p1 = local_z.atan2(l4);
        log::debug!(
            "theta_2p1:         {:5.2}rad {:5.2}°",
            theta_2p1,
            theta_2p1.to_degrees()
        );
        let theta_2p2 =
            ((self.l1.powi(2) + l5.powi(2) - self.l2.powi(2)) / (2.0 * self.l1 * l5)).acos();
        log::debug!(
            "theta_2p2:         {:5.2}rad {:5.2}°",
            theta_2p2,
            theta_2p2.to_degrees()
        );

        let theta_2 = local_z.atan2(l4) + law_of_cosines(self.l1, l5, self.l2);
        let theta_3 = std::f32::consts::PI - law_of_cosines(self.l1, self.l2, l5);

        let theta_4 = if target.orientation.axis().is_some() {
            let attach_target = target.orientation.angle();
            log::debug!(
                "Attachment target: {:5.2}rad {:5.2}°",
                attach_target,
                attach_target.to_degrees()
            );

            let abs_pitch_attachment = (-59.35_f32.to_radians() + theta_2) + theta_3;
            log::debug!(
                "Projected pitch:   {:5.2}rad {:5.2}°",
                abs_pitch_attachment,
                abs_pitch_attachment.to_degrees()
            );

            let rel_attachment_error = attach_target - abs_pitch_attachment; // - 55.0_f32.to_radians();
            log::debug!(
                "RelAttach error:   {:5.2}rad {:5.2}°",
                rel_attachment_error,
                rel_attachment_error.to_degrees()
            );

            Some(rel_attachment_error)
        } else {
            None
        };

        Ok((theta_1, theta_2, theta_3, theta_4))
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

    let (instance, address) = glonax::channel::recv_instance().await?;

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

async fn daemonize(config: &config::DumpConfig) -> anyhow::Result<()> {
    use glonax::core::geometry::EulerAngles;
    use glonax::core::Motion;
    use glonax::robot::{
        Device, DeviceType, Joint, JointType, MotionProfile, RobotBuilder, RobotType,
    };

    let kinematic_epsilon = 0.0001;
    let kinematic_control = true;

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
        .add_joint(
            Joint::with_actuator(
                "frame",
                JointType::Continuous,
                glonax::core::Actuator::Slew,
                MotionProfile::new(7_000.0, 12_000.0, 0.01, false),
            )
            .set_height(1.295),
        )
        .add_joint(
            Joint::with_actuator(
                "boom",
                JointType::Revolute,
                glonax::core::Actuator::Boom,
                MotionProfile::new(15_000.0, 12_000.0, 0.01, false),
            )
            .set_origin_translation(0.16, 0.0, 0.595)
            .set_pitch(-59.35_f32.to_radians())
            .set_bounds(-59.35_f32.to_radians(), 45.0_f32.to_radians()),
        )
        .add_joint(
            Joint::with_actuator(
                "arm",
                JointType::Revolute,
                glonax::core::Actuator::Arm,
                MotionProfile::new(15_000.0, 12_000.0, 0.01, true),
            )
            .set_length(6.0)
            .set_bounds(38.96_f32.to_radians(), 158.14_f32.to_radians()),
        )
        .add_joint(
            Joint::with_actuator(
                "attachment",
                JointType::Revolute,
                glonax::core::Actuator::Attachment,
                MotionProfile::new(15_000.0, 12_000.0, 0.05, false),
            )
            .set_length(2.97)
            .set_pitch(-55_f32.to_radians())
            .set_bounds(-55_f32.to_radians(), 125_f32.to_radians())
            .set_tolerance(0.05),
        )
        .add_joint(Joint::new("effector", JointType::Fixed).set_length(1.5))
        .build();

    log::debug!("Configured: {}", robot);

    // let frame_encoder = robot.device_by_name("frame_encoder").unwrap();
    // let boom_encoder = robot.device_by_name("boom_encoder").unwrap();
    // let arm_encoder = robot.device_by_name("arm_encoder").unwrap();
    // let attachment_encoder = robot.device_by_name("attachment_encoder").unwrap();

    let mut perception_chain = glonax::robot::Chain::new(robot);
    perception_chain
        .add_link("frame")
        .add_link("boom")
        .add_link("arm")
        .add_link("attachment");

    let mut projection_chain = perception_chain.clone();

    let perception_chain_shared = std::sync::Arc::new(tokio::sync::RwLock::new(perception_chain));
    let perception_chain_shared_read = perception_chain_shared.clone();

    let last_update = std::sync::Arc::new(tokio::sync::RwLock::new(std::time::Instant::now()));
    let last_update_read = last_update.clone();

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

    tokio::spawn(async move {
        use glonax::core::{Metric, Signal};
        use glonax::transport::frame::{Frame, FrameMessage};

        let socket = glonax::channel::broadcast_bind()
            .await
            .expect("Failed to bind to socket");

        let mut buffer = [0u8; 1024];

        log::debug!("Listening for signals");

        loop {
            let (size, _) = socket.recv_from(&mut buffer).await.unwrap();

            if let Ok(frame) = Frame::try_from(&buffer[..size]) {
                if frame.message == FrameMessage::Signal {
                    let signal = Signal::try_from(&buffer[frame.payload_range()]).unwrap();

                    if let Metric::EncoderAbsAngle((node, value)) = signal.metric {
                        match node {
                            node if 0x6A == node => {
                                perception_chain_shared
                                    .write()
                                    .await
                                    .set_joint_position("frame", Rotation3::from_yaw(value));
                                *last_update.write().await = std::time::Instant::now();
                            }
                            node if 0x6B == node => {
                                perception_chain_shared
                                    .write()
                                    .await
                                    .set_joint_position("boom", Rotation3::from_pitch(value));
                                *last_update.write().await = std::time::Instant::now();
                            }
                            node if 0x6C == node => {
                                perception_chain_shared
                                    .write()
                                    .await
                                    .set_joint_position("arm", Rotation3::from_pitch(value));
                                *last_update.write().await = std::time::Instant::now();
                            }
                            node if 0x6D == node => {
                                perception_chain_shared
                                    .write()
                                    .await
                                    .set_joint_position("attachment", Rotation3::from_pitch(value));
                                *last_update.write().await = std::time::Instant::now();
                            }
                            _ => {}
                        }

                        // log::debug!("Perception: {:?}", perception_chain_shared.read().await);
                    }
                }
            }
        }
    });

    ///////////////////////////////////////////

    // let targets = [Target::new(
    //     na::Point3::new(5.21 + 0.16, 2.50, 1.295 + 0.595),
    //     na::Rotation3::identity(),
    // )];

    let str = std::fs::read_to_string("contrib/share/programs/basic_training.json")?;
    let targets: Vec<Target> = serde_json::from_str::<Vec<[f32; 6]>>(&str)?
        .iter()
        .map(|v| {
            Target::new(
                na::Point3::new(v[0], v[1], v[2]),
                na::Rotation3::from_euler_angles(v[3], v[4], v[5]),
            )
        })
        .collect();

    for (idx, target) in targets.iter().enumerate() {
        log::debug!(
            "Target {:2}           ({:.2}, {:.2}, {:.2}) [{:.2}rad {:.2}°, {:.2}rad {:.2}°, {:.2}rad {:.2}°]",
            idx,
            target.point.x,
            target.point.y,
            target.point.z,
            target.orientation.axis().map_or(0.0, |axis| axis.x * target.orientation.angle()),
            target.orientation.axis().map_or(0.0, |axis| axis.x * target.orientation.angle()).to_degrees(),
            target.orientation.axis().map_or(0.0, |axis| axis.y * target.orientation.angle()),
            target.orientation.axis().map_or(0.0, |axis| axis.y * target.orientation.angle()).to_degrees(),
            target.orientation.axis().map_or(0.0, |axis| axis.z * target.orientation.angle()),
            target.orientation.axis().map_or(0.0, |axis| axis.z * target.orientation.angle()).to_degrees(),
        );
    }

    for target in targets {
        projection_chain.reset();

        client.send_motion(Motion::ResumeAll).await?;

        log::debug!(
            "Target              ({:.2}, {:.2}, {:.2}) [{:.2}rad {:.2}°, {:.2}rad {:.2}°, {:.2}rad {:.2}°]",
            target.point.x,
            target.point.y,
            target.point.z,
            target.orientation.axis().map_or(0.0, |axis| axis.x * target.orientation.angle()),
            target.orientation.axis().map_or(0.0, |axis| axis.x * target.orientation.angle()).to_degrees(),
            target.orientation.axis().map_or(0.0, |axis| axis.y * target.orientation.angle()),
            target.orientation.axis().map_or(0.0, |axis| axis.y * target.orientation.angle()).to_degrees(),
            target.orientation.axis().map_or(0.0, |axis| axis.z * target.orientation.angle()),
            target.orientation.axis().map_or(0.0, |axis| axis.z * target.orientation.angle()).to_degrees(),
        );

        ///////////////////////////////////
        // log::debug!("IK");

        // let rot = rot * Rotation3::from_pitch(10_f32.to_radians());

        // let rot = Rotation3::from_yaw(90_f32.to_radians() );
        // let rot = rot * Rotation3::from_pitch(40_f32.to_radians());

        // let yy = na::UnitQuaternion::from_euler_angles(0.0, 40.0_f32.to_radians(), 0.0);
        // let yy = yy * na::UnitQuaternion::from_euler_angles(0.0, 10_f32.to_radians(), 0.0);

        // let yy = yy * na::UnitQuaternion::from_euler_angles(0.0, 0.0, 45_f32.to_radians());

        // log::debug!("YY:           {}", yy);
        // log::debug!("axis_angle:   {:?}", yy.axis_angle());
        // log::debug!(
        //     "YY theta:     {}    {:.2}",
        //     yy.angle(),
        //     yy.angle().to_degrees()
        // );
        // log::debug!("axis_angle:   {:?}", yy.euler_angles());

        // log::debug!("{}", rot);

        // log::debug!(
        //     "Angle        {:?}       {:.2}rad {:.2}°",
        //     rot.axis().unwrap(),
        //     rot.axis().unwrap().z * rot.axis().unwrap().y * rot.angle(),
        //     rot.axis().unwrap().z * rot.axis().unwrap().y * rot.angle().to_degrees()
        // );

        // let rot = na::Rotation3::rotation_between(&na::Vector3::new(1.0, 0.0, 0.0), &target.point.coords);

        // log::debug!("Angle              {:#?}", rot);

        // let offset = rot * na::Point2::new(0.16, 0.0);
        // log::debug!("Vector point:       [{:.3}, {:.3}]", offset.x, offset.y);
        // let lv = na::distance(&na::Point2::new(0.0, 0.0), &offset);
        // log::debug!("Vector L:           {:.2}", lv);

        // let l1 = na::distance(&na::Point2::new(0.0, 0.0), &target.point.xy());
        // log::debug!("L1 distance:        {:.2}", l1);
        // let l4 = na::distance(&offset, &target.point.xy());
        // log::debug!("L4 distance:        {:.2}", l4);

        // let offset = na::Point3::new(offset.x, offset.y, 0.595 + 1.295);
        // let l5 = na::distance(&offset, &target.point);
        // log::debug!("L5 distance:        {:.2}", l5);

        // let q = target - offset;
        // log::debug!("Vector point:       [{:.3}, {:.3}, {:.3}]", q.x, q.y, q.z);

        // let rot = na::Rotation3::rotation_between(&na::Vector3::new(1.0,0.0, 0.0), &na::Vector3::new(5.226, 2.491, 0.010) ).unwrap();
        // log::debug!(
        //     "Angle θ2p1          {:?} {:?} {:?}",
        //     rot.axis(),
        //     rot.angle(),
        //     rad_to_deg(rot.angle() as f32)
        // );

        // let q = target - offset;

        // log::debug!("Vector point:       [{:.3}, {:.3}, {:.3}]", q.x, q.y, q.z);

        // let rot = na::Rotation2::rotation_between(&na::Vector2::new(1.0,0.01), &na::Vector2::new(5.226,0.2) );
        // log::debug!(
        //     "Angle θ2p1          {:?} {:?}",
        //     rot.angle(),
        //     rad_to_deg(rot.angle() as f32)
        // );

        // let l5 = na::distance(&na::Point2::new(0.16, 0.0), &target.xz());
        // log::debug!("L5 distance: {:.2}", l5);

        // let theta_2p2 = ((6.0_f32.powi(2) + (l5 as f32).powi(2) - 2.97_f32.powi(2))
        //     / (2.0 * 6.0_f32 * (l5 as f32)))
        //     .acos();

        // log::debug!(
        //     "Angle (θp2): {:5.2}rad {:5.2}°",
        //     theta_2p2,
        //     rad_to_deg(theta_2p2)
        // );

        // let theta_3 = std::f32::consts::PI
        //     - ((6.0_f32.powi(2) + 2.97_f32.powi(2) - (l5 as f32).powi(2)) / (2.0 * 6.0_f32 * 2.97_f32))
        //         .acos();

        // log::debug!(
        //     "Angle (θ3): {:5.2}rad {:5.2}°",
        //     theta_3,
        //     rad_to_deg(theta_3)
        // );

        // return Ok(());

        ///////////////////////////////////
        log::debug!("IK2");

        let solver = InverseKinematics::new(6.0, 2.97);

        let (p_frame_yaw, p_boom_pitch, p_arm_pitch, p_attachment_pitch) =
            solver.solve(&target).expect("IK failed");
        log::debug!(
            "IK angles:         {:5.2}rad {:5.2}° {:5.2}rad {:5.2}°  {:5.2}rad {:5.2}°",
            p_frame_yaw,
            p_frame_yaw.to_degrees(),
            p_boom_pitch,
            p_boom_pitch.to_degrees(),
            p_arm_pitch,
            p_arm_pitch.to_degrees(),
        );

        if let Some(angle) = p_attachment_pitch {
            log::debug!(
                "IK angles:         {:5.2}rad {:5.2}°",
                angle,
                angle.to_degrees()
            );
        }

        ///////////////////////////////////

        projection_chain.set_joint_positions(vec![
            Rotation3::from_yaw(p_frame_yaw),
            Rotation3::from_pitch(-p_boom_pitch + 59.35_f32.to_radians()),
            Rotation3::from_pitch(p_arm_pitch),
        ]);

        if let Some(angle) = p_attachment_pitch {
            projection_chain.set_joint_position("attachment", Rotation3::from_pitch(angle));
        }

        let projection_point = projection_chain.world_transformation() * na::Point3::origin();

        log::debug!("Projection chain: {:?}", projection_chain);

        if (target.point.coords.norm() - projection_point.coords.norm()).abs() > kinematic_epsilon {
            log::error!("Target norm: {}", target.point.coords.norm());
            log::error!("Projection norm: {}", projection_point.coords.norm());
            log::error!(
                "Diff: {}",
                target.point.coords.norm() - projection_point.coords.norm()
            );
            return Err(anyhow::anyhow!("IK error"));
        }

        log::info!("Press enter to continue");
        std::io::stdin().read_line(&mut String::new())?;

        let ground_plane = parry3d::shape::Cuboid::new(na::Vector3::new(10.0, 10.0, 1.0));

        loop {
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;

            let time_since_last_update = last_update_read.read().await.elapsed();

            if time_since_last_update > std::time::Duration::from_millis(200) {
                log::warn!("No update received for 200ms, stopping");
                client.send_motion(Motion::StopAll).await?;
                break;
            }

            let perception_chain = perception_chain_shared_read.read().await;

            log::debug!("Perception: {:?}", perception_chain);

            let distance = projection_chain.distance(&perception_chain);
            log::debug!("Target distance:    {:.2}m", distance);

            let effector_point = perception_chain.world_transformation() * na::Point3::origin();

            // let point_projection = ground_plane.project_local_point(&effector_point, true);

            let transform = Isometry3::from_parts(
                na::Translation3::new(0.0, 0.0, -1.0),
                na::UnitQuaternion::identity(),
            );

            let point_projection = ground_plane.project_point(&transform, &effector_point, true);
            let distance = ground_plane.distance_to_point(&transform, &effector_point, true);

            log::debug!(
                "Ground              Contact: {}  Distance: {:.2}",
                point_projection.is_inside,
                distance
            );

            if !perception_chain.is_ready() || !kinematic_control {
                continue;
            }

            let mut done = true;

            // TODO: Send all commands at once
            for joint_diff in perception_chain.error(&projection_chain) {
                log::debug!(" ⇒ {:?}", joint_diff);

                if let Some(motion) = joint_diff.actuator_motion() {
                    // motion_list.push(motion);
                    client.send_motion(motion).await?;
                }

                done = joint_diff.is_below_tolerance() && done;
            }

            if done {
                client.send_motion(Motion::StopAll).await?;
                break;
            }
        }
    }

    Ok(())
}
