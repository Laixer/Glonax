// Copyright (C) 2023 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use clap::Parser;

use na::{Isometry3, Point3, UnitQuaternion, Vector3};
use nalgebra as na;
use parry3d::shape::Cuboid;

mod config;
mod robot;

#[derive(Clone, Copy)]
struct Target {
    pub point: Point3<f32>,
    pub orientation: UnitQuaternion<f32>,
    pub interpolation: bool,
}

impl Target {
    fn new(point: Point3<f32>, orientation: UnitQuaternion<f32>) -> Self {
        Self {
            point,
            orientation,
            interpolation: false,
        }
    }

    fn from_point(x: f32, y: f32, z: f32) -> Self {
        Self {
            point: Point3::new(x, y, z),
            orientation: UnitQuaternion::identity(),
            interpolation: false,
        }
    }
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({:.2}, {:.2}, {:.2}) [{:.2}rad {:.2}°, {:.2}rad {:.2}°, {:.2}rad {:.2}°]",
            self.point.x,
            self.point.y,
            self.point.z,
            self.orientation
                .axis()
                .map_or(0.0, |axis| axis.x * self.orientation.angle()),
            self.orientation
                .axis()
                .map_or(0.0, |axis| axis.x * self.orientation.angle())
                .to_degrees(),
            self.orientation
                .axis()
                .map_or(0.0, |axis| axis.y * self.orientation.angle()),
            self.orientation
                .axis()
                .map_or(0.0, |axis| axis.y * self.orientation.angle())
                .to_degrees(),
            self.orientation
                .axis()
                .map_or(0.0, |axis| axis.z * self.orientation.angle()),
            self.orientation
                .axis()
                .map_or(0.0, |axis| axis.z * self.orientation.angle())
                .to_degrees(),
        )
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
        log::debug!(" IK Local Z:        {:.2}", local_z);

        let theta_1 = target.point.y.atan2(target.point.x);

        let offset = 0.16;
        let offset_x = offset * theta_1.cos();
        let offset_y = offset * theta_1.sin();

        log::debug!(" IK Vector offset:  ({:.2}, {:.2})", offset_x, offset_y);

        let local_x = target.point.x - offset_x;
        let local_y = target.point.y - offset_y;
        log::debug!(" IK Local X:        {:.2}", local_x);
        log::debug!(" IK Local Y:        {:.2}", local_y);

        // L4 is the leg between the origin and the target projected on the XY plane (ground).
        let l4 = (local_x.powi(2) + local_y.powi(2)).sqrt();
        log::debug!(" IK Vector length L4: {:.2}", l4);
        // L5 is the leg between the origin and the target (vector).
        let l5 = (l4.powi(2) + local_z.powi(2)).sqrt();
        log::debug!(" IK Vector length L5: {:.2}", l5);

        if l5 >= self.l1 + self.l2 {
            return Err(());
        }

        let theta_2p1 = local_z.atan2(l4);
        log::debug!(
            " IK theta_2p1:      {:5.2}rad {:5.2}°",
            theta_2p1,
            theta_2p1.to_degrees()
        );
        let theta_2p2 =
            ((self.l1.powi(2) + l5.powi(2) - self.l2.powi(2)) / (2.0 * self.l1 * l5)).acos();
        log::debug!(
            " IK theta_2p2:      {:5.2}rad {:5.2}°",
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

            let rel_attachment_error = attach_target - abs_pitch_attachment;
            log::debug!(
                "RelAttach error:   {:5.2}rad {:5.2}°",
                rel_attachment_error,
                rel_attachment_error.to_degrees()
            );

            if rel_attachment_error < -55.0_f32.to_radians() {
                log::warn!("Attachment pitch is below lower bound");
            } else if rel_attachment_error > 125.0_f32.to_radians() {
                log::warn!("Attachment pitch is above upper bound");
            }

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
    use std::collections::VecDeque;

    let kinematic_epsilon = 0.0001;
    let kinematic_control = true;
    let kinematic_interval = std::time::Duration::from_millis(25);

    let robot = robot::excavator(&config);

    log::debug!("Configured: {}", robot);

    let frame_encoder = robot.device_by_name("frame_encoder").unwrap().clone();
    let boom_encoder = robot.device_by_name("boom_encoder").unwrap().clone();
    let arm_encoder = robot.device_by_name("arm_encoder").unwrap().clone();
    let attachment_encoder = robot.device_by_name("attachment_encoder").unwrap().clone();

    let mut perception_chain = glonax::robot::Chain::new(robot);
    perception_chain
        .add_link("frame")
        .add_link("boom")
        .add_link("arm")
        .add_link("attachment");

    let mut projection_chain = perception_chain.clone();

    let perception_chain_shared = std::sync::Arc::new(tokio::sync::RwLock::new(perception_chain));
    let perception_chain_shared_read = perception_chain_shared.clone();

    let mut client = glonax::transport::ConnectionOptions::new()
        .read(false)
        .write(true)
        .connect(
            config.address.to_owned(),
            config.global.bin_name.to_string(),
        )
        .await?;

    log::info!("Connected to {}", config.address);

    client.send_motion(Motion::StopAll).await?;

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
                            node if frame_encoder.id() == node => {
                                perception_chain_shared
                                    .write()
                                    .await
                                    .set_joint_position("frame", UnitQuaternion::from_yaw(value));
                            }
                            node if boom_encoder.id() == node => {
                                perception_chain_shared
                                    .write()
                                    .await
                                    .set_joint_position("boom", UnitQuaternion::from_pitch(value));
                            }
                            node if arm_encoder.id() == node => {
                                perception_chain_shared
                                    .write()
                                    .await
                                    .set_joint_position("arm", UnitQuaternion::from_pitch(value));
                            }
                            node if attachment_encoder.id() == node => {
                                perception_chain_shared.write().await.set_joint_position(
                                    "attachment",
                                    UnitQuaternion::from_pitch(value),
                                );
                            }
                            _ => {}
                        }

                        log::trace!("Perception: {:?}", perception_chain_shared.read().await);
                    }
                }
            }
        }
    });

    let mut targets = VecDeque::from([
        Target::new(
            Point3::new(5.21 + 0.16, 0.0, 1.295 + 0.595),
            UnitQuaternion::from_euler_angles(0.0, 90_f32.to_radians() + 60_f32.to_radians(), 0.0),
        ),
        Target::new(
            Point3::new(5.21 + 0.16, 5.0, 1.295 + 0.595),
            UnitQuaternion::from_euler_angles(0.0, 90_f32.to_radians() + 60_f32.to_radians(), 0.0),
        ),
    ]);

    // let str = std::fs::read_to_string("contrib/share/programs/basic_training.json")?;
    // let targets: Vec<Target> = serde_json::from_str::<Vec<[f32; 6]>>(&str)?
    //     .iter()
    //     .map(|v| {
    //         Target::new(
    //             Point3::new(v[0], v[1], v[2]),
    //             UnitQuaternion::from_euler_angles(v[3], v[4], v[5]),
    //         )
    //     })
    //     .collect();

    for (idx, target) in targets.iter().enumerate() {
        log::debug!(" * Target {:2}    {}", idx, target);
    }

    let ground_plane = Cuboid::new(Vector3::new(10.0, 10.0, 1.0));
    let ground_transform = Isometry3::translation(0.0, 0.0, -1.0);

    let obst0_box = Cuboid::new(Vector3::new(2.5, 0.25, 0.5));
    let obst0_transform = Isometry3::translation(3.0, 2.0, 0.5);

    let bucket_geometry = Cuboid::new(Vector3::new(0.75, 1.04, 0.25));
    let bucket_transform = Isometry3::translation(0.75, 0.0, 0.375);

    loop {
        projection_chain.reset();

        if targets.is_empty() {
            log::info!("All targets reached");

            break;
        }

        let target = targets.pop_front().unwrap();

        {
            log::debug!("Current target      {}", target);
            log::debug!("Is interpolation:   {}", target.interpolation);

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

            projection_chain.set_joint_positions(vec![
                UnitQuaternion::from_yaw(p_frame_yaw),
                UnitQuaternion::from_pitch(-p_boom_pitch + 59.35_f32.to_radians()),
                UnitQuaternion::from_pitch(p_arm_pitch),
            ]);

            if let Some(angle) = p_attachment_pitch {
                projection_chain.set_joint_position(
                    "attachment",
                    UnitQuaternion::from_pitch(angle + 55_f32.to_radians()),
                );
            }

            let projection_point = projection_chain.world_transformation() * na::Point3::origin();

            log::debug!("Projection chain: {:?}", projection_chain);

            if (target.point.coords.norm() - projection_point.coords.norm()).abs()
                > kinematic_epsilon
            {
                log::error!("Target norm: {}", target.point.coords.norm());
                log::error!("Projection norm: {}", projection_point.coords.norm());
                log::error!(
                    "Diff: {}",
                    target.point.coords.norm() - projection_point.coords.norm()
                );
                return Err(anyhow::anyhow!("IK error"));
            }
        }

        log::info!("Press enter to continue");
        std::io::stdin().read_line(&mut String::new())?;

        client.send_motion(Motion::ResumeAll).await?;

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        loop {
            tokio::time::sleep(kinematic_interval).await;

            let perception_chain = perception_chain_shared_read.read().await;
            if perception_chain.last_update().elapsed() > std::time::Duration::from_millis(200) {
                log::warn!("No update received for 200ms, stopping");
                client.send_motion(Motion::StopAll).await?;
                break;
            }

            log::debug!("Perception: {:?}", perception_chain);

            if let Some(abs_pitch) = perception_chain.abs_pitch() {
                log::debug!("Absolute pitch:    {:.2}°", abs_pitch.to_degrees());
            }

            let distance = projection_chain.distance(&perception_chain);
            log::debug!("Target distance:   {:.2}m", distance);

            let mut contact_zone = false;
            let mut has_contact = false;
            let mut needs_reposition = false;

            let world_transformation = perception_chain.world_transformation() * bucket_transform;

            let colliders = [
                (&ground_transform, &ground_plane),
                (&obst0_transform, &obst0_box),
            ];

            for (collider_transform, collider_geom) in colliders {
                let res = parry3d::query::contact(
                    collider_transform,
                    collider_geom,
                    &world_transformation,
                    &bucket_geometry,
                    1.0,
                );

                if let Ok(contact) = res {
                    if let Some(contact) = contact {
                        contact_zone = true;

                        // log::warn!(
                        //     "                        Effector is in obstacle prediction zone"
                        // );

                        if contact.dist.abs() < 0.40 && !target.interpolation {
                            log::warn!("                        Effector needs repositioning");
                            needs_reposition = true;
                        }

                        if contact.dist.abs() < 0.15 {
                            log::warn!("                        Effector is too close to obstacle");
                            has_contact = true;
                        }

                        log::debug!("Collider           Distance: {:.2}m", contact.dist);
                        log::debug!("Collider           ({:+.2}, {:+.2}, {:+.2}) - ({:+.2}, {:+.2}, {:+.2})",
                            contact.point1.x,
                            contact.point1.y,
                            contact.point1.z,
                            contact.point2.x,
                            contact.point2.y,
                            contact.point2.z
                        );
                    }
                }
            }

            if !perception_chain.is_ready() || !kinematic_control {
                continue;
            }

            if needs_reposition && !target.interpolation {
                client.send_motion(Motion::StopAll).await?;

                let current_point = perception_chain.world_transformation() * Point3::origin();

                log::debug!(
                    "Try target:        ({:.2}, {:.2}, {:.2})",
                    current_point.x,
                    current_point.y,
                    current_point.z + 0.50
                );

                log::info!("Press enter to continue");
                std::io::stdin().read_line(&mut String::new())?;

                targets.push_front(target);

                let mut interpol_target =
                    Target::from_point(current_point.x, current_point.y, current_point.z + 0.50);
                interpol_target.interpolation = true;

                targets.push_front(interpol_target);

                break;
            }

            if has_contact {
                log::error!("                       Effector is in obstacle contact zone");
                client.send_motion(Motion::StopAll).await?;
                break;
            }

            let mut done = true;

            // TODO: Send all commands at once
            for mut joint_diff in perception_chain.error(&projection_chain) {
                joint_diff.power_limit = if contact_zone { Some(20_000) } else { None };

                log::debug!(" ⇒ {:?}", joint_diff);

                if let Some(motion) = joint_diff.actuator_motion() {
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
