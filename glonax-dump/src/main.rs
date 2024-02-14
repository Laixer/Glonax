// Copyright (C) 2024 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use clap::Parser;

use glonax::geometry::{shortest_rotation, EulerAngles};
use na::{Isometry3, UnitQuaternion, Vector3};
use nalgebra as na;
use parry3d::shape::Cuboid;

mod config;
mod program;
mod robot;

#[derive(Parser)]
#[command(author = "Copyright (C) 2024 Laixer Equipment B.V.")]
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

    // let (instance, address) = glonax::channel::recv_instance().await?;

    let mut config = config::DumpConfig {
        // address,
        instance: glonax::core::Instance {
            id: String::new(),
            model: String::new(),
            name: String::new(),
        },
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
    use glonax::core::Motion;
    use std::time::Duration;
    use tokio::time;

    // let kinematic_detect_obstacles = false;
    let kinematic_control = true;
    let kinematic_interval = std::time::Duration::from_millis(25);

    let robot = robot::excavator(&config);

    // log::debug!("Configured: {}", robot);

    let perception_chain = robot.kinematic_chain();
    let mut objective_chain = robot.kinematic_chain();

    let perception_chain_shared = std::sync::Arc::new(tokio::sync::RwLock::new(perception_chain));
    let perception_chain_shared_read = perception_chain_shared.clone();

    let mut client = glonax::transport::ConnectionOptions::new()
        .read(false)
        .write(true)
        .connect("localhost:30051", config.global.bin_name.to_string())
        .await?;

    log::info!("Connected to {}", "localhost:30051");

    client.send_packet(&Motion::StopAll).await?;

    let frame_encoder = robot.frame_device_id();
    let boom_encoder = robot.boom_device_id();
    let arm_encoder = robot.arm_device_id();
    let attachment_encoder = robot.attachment_device_id();

    // tokio::spawn(async move {
    // use glonax::core::{Metric, Signal};
    // use glonax::transport::frame::{Frame, FrameMessage};

    // let socket = glonax::channel::broadcast_bind().expect("Failed to bind to socket");

    // let mut buffer = [0u8; 1024];

    // log::debug!("Listening for signals");

    // loop {
    //     let (size, _) = socket.recv_from(&mut buffer).await.unwrap();

    // if let Ok(frame) = Frame::try_from(&buffer[..size]) {
    //     if frame.message == FrameMessage::Signal {
    //         let signal = Signal::try_from(&buffer[frame.payload_range()]).unwrap();
    //         if let Metric::EncoderAbsAngle((node, value)) = signal.metric {
    //             match node {
    //                 node if frame_encoder == node => {
    //                     perception_chain_shared
    //                         .write()
    //                         .await
    //                         .set_joint_position("frame", UnitQuaternion::from_yaw(value));
    //                 }
    //                 node if boom_encoder == node => {
    //                     let pitch = value - 59.35_f32.to_radians();
    //                     perception_chain_shared
    //                         .write()
    //                         .await
    //                         .set_joint_position("boom", UnitQuaternion::from_pitch(pitch));
    //                 }
    //                 node if arm_encoder == node => {
    //                     perception_chain_shared
    //                         .write()
    //                         .await
    //                         .set_joint_position("arm", UnitQuaternion::from_pitch(value));
    //                 }
    //                 node if attachment_encoder == node => {
    //                     let pitch = value - 55_f32.to_radians();
    //                     perception_chain_shared.write().await.set_joint_position(
    //                         "attachment",
    //                         UnitQuaternion::from_pitch(pitch),
    //                     );
    //                 }
    //                 _ => {}
    //             }

    //             log::trace!("Perception: {:?}", perception_chain_shared.read().await);
    //         }
    //     }
    // }
    //     }
    // });

    let mut program = program::from_file("contrib/share/programs/basic_training.json")?; // axis_align_test

    for (idx, target) in program.iter().enumerate() {
        log::debug!("> Target {:2}    {}", idx, target);
    }

    let ground_plane = Cuboid::new(Vector3::new(10.0, 10.0, 1.0));
    let ground_transform = Isometry3::translation(0.0, 0.0, -1.0);

    // let obst0_box = Cuboid::new(Vector3::new(2.5, 0.205, 0.725));
    // let obst0_box_buffer = Cuboid::new(Vector3::new(
    //     obst0_box.half_extents.x + 0.15,
    //     obst0_box.half_extents.y + 0.15,
    //     obst0_box.half_extents.z + 0.15,
    // ));
    // let obst0_transform = Isometry3::translation(3.0, 2.5, 0.725);

    let bucket_geometry = Cuboid::new(Vector3::new(0.75, 1.04, 0.25));
    let bucket_transform = Isometry3::translation(0.75, 0.0, 0.375);

    loop {
        if program.is_empty() {
            log::info!("All targets reached");
            break;
        }

        let target = program.pop_front().unwrap();
        log::info!("Objective target: {}", target);

        // set_chain_from_target(&target, &mut objective_chain)?;
        objective_chain.set_target(&target);

        log::debug!("Objective chain: {:?}", objective_chain);

        if kinematic_control {
            log::info!("Press enter to continue");
            std::io::stdin().read_line(&mut String::new())?;
        }

        client.send_packet(&Motion::ResumeAll).await?;

        if kinematic_control {
            time::sleep(Duration::from_millis(1000)).await;
        }

        loop {
            time::sleep(kinematic_interval).await;

            let perception_chain = perception_chain_shared_read.read().await;
            if perception_chain.last_update().elapsed() > Duration::from_millis(200) {
                client.send_packet(&Motion::StopAll).await?;
                log::warn!("No update received for 200ms, stopping");
                return Err(anyhow::anyhow!("No update received for 200ms"));
            }

            log::debug!("Perception: {:?}", perception_chain);

            let error_vector = perception_chain.translation_error(&objective_chain);

            log::debug!(
                "{:<35} ({:.2}, {:.2}, {:.2})",
                "Error vector (Attachment)",
                error_vector.x.abs(),
                error_vector.y.abs(),
                error_vector.z.abs()
            );

            if error_vector.x.abs() <= 0.06
                && error_vector.y.abs() <= 0.06
                && error_vector.z.abs() <= 0.06
            {
                client.send_packet(&Motion::StopAll).await?;
                log::info!("Target reached");
                break;
            }

            // let mut contact_zone = false;
            // let mut clearance_height = 0.0_f32;

            // let colliders = [(&obst0_transform, &obst0_box_buffer)];

            // let perception_transformation = perception_chain.transformation() * bucket_transform;

            // for (collider_transform, collider_geom) in colliders {
            //     if !kinematic_detect_obstacles {
            //         break;
            //     }

            //     let res = parry3d::query::contact(
            //         collider_transform,
            //         collider_geom,
            //         &perception_transformation,
            //         &bucket_geometry,
            //         1.0,
            //     );

            //     if let Ok(contact) = res {
            //         if let Some(contact) = contact {
            //             contact_zone = true;

            //             log::debug!("- Collider dist            {:.2}m", contact.dist);
            //             log::debug!("- Collider points          ({:+.2}, {:+.2}, {:+.2}) - ({:+.2}, {:+.2}, {:+.2})",
            //                 contact.point1.x,
            //                 contact.point1.y,
            //                 contact.point1.z,
            //                 contact.point2.x,
            //                 contact.point2.y,
            //                 contact.point2.z
            //             );
            //             log::debug!("- Collider normals         ({:+.2}, {:+.2}, {:+.2}) - ({:+.2}, {:+.2}, {:+.2})",
            //                 contact.normal1.x,
            //                 contact.normal1.y,
            //                 contact.normal1.z,
            //                 contact.normal2.x,
            //                 contact.normal2.y,
            //                 contact.normal2.z
            //             );

            //             // TODO: This is just informational, remove later
            //             let is_intersecting = parry3d::query::intersection_test(
            //                 collider_transform,
            //                 collider_geom,
            //                 &perception_transformation,
            //                 &bucket_geometry,
            //             )
            //             .unwrap();

            //             log::debug!("- Collider intersects      {}", is_intersecting);

            //             if contact.dist.abs() < 0.05 {
            //                 // TODO: Maybe not here?
            //                 if perception_chain.is_ready() && kinematic_control {
            //                     client.send_motion(Motion::StopAll).await?;
            //                     log::error!("Effector is in obstacle contact zone");
            //                     return Err(anyhow::anyhow!(
            //                         "Effector is in obstacle contact zone"
            //                     ));
            //                 } else {
            //                     log::error!("Effector is in obstacle contact zone");
            //                 }
            //             }

            //             let collider = collider_geom.aabb(&collider_transform);

            //             log::debug!("- Collider maxs            {:?}", collider.maxs);

            //             if contact.dist.abs() < 0.40 {
            //                 clearance_height = clearance_height.max(collider.maxs.z + 0.20);
            //                 log::debug!(
            //                     "- Collider clearance       {:.2}m [{:.2}m +0.20m]",
            //                     clearance_height,
            //                     collider.maxs.z
            //                 );
            //             }
            //         }
            //     }
            // }

            // if contact_zone {
            //     // let current_point = perception_chain.world_transformation() * Point3::origin();
            //     let perception_transformation = perception_chain.transformation();

            //     let world_transformation = perception_chain.transformation() * bucket_transform;

            //     let groud_points = parry3d::query::closest_points(
            //         &ground_transform,
            //         &ground_plane,
            //         &world_transformation,
            //         &bucket_geometry,
            //         25.0,
            //     )
            //     .unwrap();

            //     let height = match groud_points {
            //         parry3d::query::ClosestPoints::WithinMargin(p1, p2) => p2.z - p1.z,
            //         _ => 0.0,
            //     };

            //     let necessary_clearance = clearance_height - height;
            //     if necessary_clearance > 0.0 {
            //         log::debug!("Necessary clearance: {:.2}m", necessary_clearance);

            //         let new_z =
            //             perception_transformation.translation.vector.z + necessary_clearance;

            //         let clearance_target = Target::from_point(
            //             perception_transformation.translation.vector.x,
            //             perception_transformation.translation.vector.y,
            //             new_z + 0.08,
            //         );
            //         log::debug!("New clearance target: {}", clearance_target);

            //         // set_chain_from_target(&clearance_target, &mut objective_chain)?;
            //         objective_chain.set_target(&clearance_target);

            //         client.send_motion(Motion::StopAll).await?;
            //         tokio::time::sleep(Duration::from_millis(150)).await;
            //         client.send_motion(Motion::ResumeAll).await?;
            //     } else {
            //         log::debug!("No necessary clearance, continue with current height");

            //         let clearance_target = Target::from_point(
            //             target.point.x,
            //             target.point.y,
            //             perception_transformation.translation.vector.z,
            //         );
            //         log::debug!("New clearance target: {}", clearance_target);

            //         // set_chain_from_target(&clearance_target, &mut objective_chain)?;
            //         objective_chain.set_target(&clearance_target);
            //     }
            // } else {
            //     // set_chain_from_target(&target, &mut objective_chain)?;
            //     objective_chain.set_target(&target);
            // }

            let pitch = perception_chain.effector_pitch_angle();
            log::debug!(
                "{:<35} {:.2}°",
                "Abs pitch (Attachment)",
                pitch.to_degrees()
            );

            let dist2 = perception_chain.translation_norm(&objective_chain);

            log::debug!("{:<35} {:5.2}m", "Target distance (Attachment)", dist2);

            let world_transformation = perception_chain.transformation() * bucket_transform;

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
                    log::debug!("{:<35} intersecting", "Ground clearance (Attachment)");
                }
                parry3d::query::ClosestPoints::WithinMargin(p1, p2) => {
                    log::debug!(
                        "{:<35} {:5.2}m",
                        "Ground clearance (Attachment)",
                        p2.z - p1.z
                    );
                }
                parry3d::query::ClosestPoints::Disjoint => {
                    log::debug!("{:<35} disjoint", "Ground clearance (Attachment)");
                }
            }

            for joint_diff in perception_chain.rotation_error(&objective_chain) {
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

                // let error_angle = {
                //     if joint.ty() == &glonax::robot::JointType::Continuous {
                //         shortest_rotation(error_angle)
                //     } else {
                //         error_angle
                //     }
                // };

                let power = profile.power(error_angle);
                // let power = if contact_zone {
                //     power.max(-15_000).min(15_000)
                // } else {
                //     power
                // };

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
                    client.send_packet(&Motion::new(actuator, power)).await?;
                }
            }
        }
    }

    Ok(())
}
