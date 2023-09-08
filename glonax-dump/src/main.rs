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
mod ik;
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

fn set_chain_from_target(target: &Target, chain: &mut glonax::robot::Chain) -> anyhow::Result<()> {
    let kinematic_epsilon = 0.0001;

    let solver = ik::ExcavatorIK::new(6.0, 2.97);

    let kinrot = solver
        .solve(target)
        .map_err(|_| anyhow::anyhow!("IK error"))?;

    let mut positions = vec![kinrot.frame, kinrot.boom, kinrot.arm];

    if let Some(attachment) = kinrot.attachment {
        positions.push(attachment);
    }

    chain.reset();
    chain.set_joint_positions(positions);

    let vector = chain.world_transformation().translation.vector;

    if (target.point.coords.norm() - vector.norm()).abs() > kinematic_epsilon {
        log::error!("Target norm: {}", target.point.coords.norm());
        log::error!("Chain norm: {}", vector.norm());
        log::error!("Diff: {}", target.point.coords.norm() - vector.norm());
        Err(anyhow::anyhow!("IK error"))
    } else {
        Ok(())
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
    use std::time::Duration;
    use tokio::time;

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
            UnitQuaternion::from_euler_angles(
                0.0,
                180_f32.to_radians() - 55.3_f32.to_radians(),
                0.0,
            ),
        ),
        Target::new(
            Point3::new(5.21 + 0.16, 5.0, 1.295 + 0.595),
            UnitQuaternion::from_euler_angles(
                0.0,
                180_f32.to_radians() - 55.3_f32.to_radians(),
                0.0,
            ),
        ),
    ]);

    // let str = std::fs::read_to_string("contrib/share/programs/basic_training.json")?;
    // let mut targets: VecDeque<Target> = serde_json::from_str::<Vec<[f32; 6]>>(&str)?
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

    let obst0_box = Cuboid::new(Vector3::new(2.5, 0.205, 0.725));
    let obst0_box_buffer = Cuboid::new(Vector3::new(
        obst0_box.half_extents.x + 0.15,
        obst0_box.half_extents.y + 0.15,
        obst0_box.half_extents.z + 0.15,
    ));
    let obst0_transform = Isometry3::translation(3.0, 2.5, 0.725);

    let bucket_geometry = Cuboid::new(Vector3::new(0.75, 1.04, 0.25));
    let bucket_transform = Isometry3::translation(0.75, 0.0, 0.375);

    loop {
        if targets.is_empty() {
            log::info!("All targets reached");

            break;
        }

        let target = targets.pop_front().unwrap();

        log::info!("Current target: {}", target);
        log::debug!("Is interpolation:   {}", target.interpolation);

        set_chain_from_target(&target, &mut projection_chain)?;

        log::debug!("Projection chain: {:?}", projection_chain);

        if kinematic_control {
            log::info!("Press enter to continue");
            std::io::stdin().read_line(&mut String::new())?;
        }

        client.send_motion(Motion::ResumeAll).await?;

        if kinematic_control {
            time::sleep(Duration::from_millis(1000)).await;
        }

        loop {
            time::sleep(kinematic_interval).await;

            let perception_chain = perception_chain_shared_read.read().await;
            if perception_chain.last_update().elapsed() > Duration::from_millis(200) {
                client.send_motion(Motion::StopAll).await?;
                log::warn!("No update received for 200ms, stopping");
                return Err(anyhow::anyhow!("No update received for 200ms"));
            }

            log::debug!("Perception: {:?}", perception_chain);

            if let Some(abs_pitch) = perception_chain.abs_pitch() {
                log::debug!(
                    "{:<35} {:.2}°",
                    "Abs pitch (Effector)",
                    abs_pitch.to_degrees()
                );
            }
            if let Some(abs_pitch) = perception_chain.abs_pitch_with_attachment() {
                log::debug!(
                    "{:<35} {:.2}°",
                    "Abs pitch (Attachment)",
                    abs_pitch.to_degrees()
                );
            }

            let distance = projection_chain.distance(&perception_chain);
            log::debug!("{:<35} {:5.2}m", "Target distance (Effector)", distance);

            let mut contact_zone = false;
            let mut has_contact = false;
            let mut clearance_height = 0.0_f32;

            let world_transformation = perception_chain.world_transformation() * bucket_transform;

            let groud_points = parry3d::query::closest_points(
                &ground_transform,
                &ground_plane,
                &world_transformation,
                &bucket_geometry,
                25.0,
            )
            .unwrap();

            match groud_points {
                parry3d::query::ClosestPoints::Intersecting => {
                    log::debug!("{:<35} intersecting", "Ground distance (Attachment)");
                }
                parry3d::query::ClosestPoints::WithinMargin(p1, p2) => {
                    log::debug!(
                        "{:<35} {:5.2}m",
                        "Ground distance (Attachment)",
                        p2.z - p1.z
                    );
                }
                parry3d::query::ClosestPoints::Disjoint => {
                    log::debug!("{:<35} disjoint", "Ground distance (Attachment)");
                }
            }

            let colliders = [(&obst0_transform, &obst0_box_buffer)];

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

                        if contact.dist.abs() < 0.05 {
                            log::warn!("                        Effector is too close to obstacle");
                            has_contact = true;
                        }

                        let collider = collider_geom.aabb(&collider_transform);

                        log::debug!("Collider max             {:?}", collider.maxs);

                        if contact.dist < 0.50 {
                            clearance_height = clearance_height.max(collider.maxs.z + 0.20);
                            log::debug!("Collider clearance       {}", clearance_height);
                        }

                        let is_intersecting = parry3d::query::intersection_test(
                            collider_transform,
                            collider_geom,
                            &world_transformation,
                            &bucket_geometry,
                        )
                        .unwrap();

                        log::debug!("Collider dist            {:.2}m", contact.dist);
                        log::debug!("Collider points          ({:+.2}, {:+.2}, {:+.2}) - ({:+.2}, {:+.2}, {:+.2})",
                            contact.point1.x,
                            contact.point1.y,
                            contact.point1.z,
                            contact.point2.x,
                            contact.point2.y,
                            contact.point2.z
                        );
                        log::debug!("Collider normals         ({:+.2}, {:+.2}, {:+.2}) - ({:+.2}, {:+.2}, {:+.2})",
                            contact.normal1.x,
                            contact.normal1.y,
                            contact.normal1.z,
                            contact.normal2.x,
                            contact.normal2.y,
                            contact.normal2.z
                        );
                        log::debug!("Collider intersects      {}", is_intersecting);
                    }
                }
            }

            if clearance_height > 0.0 && !target.interpolation {
                client.send_motion(Motion::StopAll).await?;

                let current_point = perception_chain.world_transformation() * Point3::origin();

                targets.push_front(target);

                log::debug!(
                    "Clearance target Z:        ({:.2}, {:.2}, {:.2})",
                    target.point.x,
                    target.point.y,
                    current_point.z + clearance_height,
                );

                let mut interpol_target = Target::from_point(
                    target.point.x,
                    target.point.y,
                    current_point.z + clearance_height,
                );
                interpol_target.interpolation = true;
                targets.push_front(interpol_target);

                log::debug!(
                    "Clearance target:        ({:.2}, {:.2}, {:.2})",
                    current_point.x,
                    current_point.y,
                    current_point.z + clearance_height,
                );

                let mut interpol_target = Target::from_point(
                    current_point.x,
                    current_point.y,
                    current_point.z + clearance_height,
                );
                interpol_target.interpolation = true;

                targets.push_front(interpol_target);

                break;
            }

            if has_contact {
                log::error!("                       Effector is in obstacle contact zone");
                client.send_motion(Motion::StopAll).await?;
                return Err(anyhow::anyhow!("Effector is in obstacle contact zone"));
            }

            let mut done = true;

            // TODO: Send all commands at once
            for joint_diff in perception_chain.error(&projection_chain) {
                let joint = joint_diff.joint;

                if joint.actuator().is_none()
                    || joint.profile().is_none()
                    || joint_diff.rotation.axis().is_none()
                {
                    continue;
                }

                let axis = joint_diff.rotation.axis().unwrap();
                let error_angle = axis.x * joint_diff.rotation.angle()
                    + axis.y * joint_diff.rotation.angle()
                    + axis.z * joint_diff.rotation.angle();

                let actuator = joint.actuator().unwrap();
                let profile = joint.profile().unwrap();

                let error_angle = {
                    if joint.ty() == &glonax::robot::JointType::Continuous {
                        glonax::core::geometry::shortest_rotation(error_angle)
                    } else {
                        error_angle
                    }
                };

                let power = profile.power(error_angle);
                let power = if contact_zone {
                    power.max(-15_000).min(15_000)
                } else {
                    power
                };

                log::debug!(
                    " ⇒ {:<15} Error: {:5.2}rad {:7.2}°  Power: {:6} {:7.1}%",
                    joint.name(),
                    error_angle,
                    error_angle.to_degrees(),
                    power,
                    if power == 0 {
                        0.0
                    } else {
                        ((power.abs() as f32 - profile.offset)
                            / (Motion::POWER_MAX as f32 - profile.offset))
                            * 100.0
                    },
                );

                if perception_chain.is_ready() && kinematic_control {
                    client.send_motion(Motion::new(actuator, power)).await?;
                }

                done = power == 0 && done;
            }

            if done {
                client.send_motion(Motion::StopAll).await?;
                log::info!("Target reached");
                break;
            }
        }
    }

    Ok(())
}
