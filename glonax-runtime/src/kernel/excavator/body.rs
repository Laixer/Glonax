use crate::algorithm::ik::InverseKinematics;

// TODO: Move
pub struct MotionProfile {
    pub scale: f32,
    pub offset: i16,
    pub limit: i16,
    pub cutoff: f32,
}

impl MotionProfile {
    pub fn proportional_power(&self, value: f32) -> Option<i16> {
        let power = (value * self.scale) as i16;
        let power = if value.is_sign_positive() {
            power.min(self.limit) + self.offset
        } else {
            power.max(-self.limit) - self.offset
        };

        if value.abs() > self.cutoff {
            Some(power)
        } else {
            None
        }
    }

    pub fn proportional_power_inverse(&self, value: f32) -> Option<i16> {
        let power = (value * self.scale) as i16;
        let power = if value.is_sign_positive() {
            (-power).max(-self.limit) - self.offset
        } else {
            (-power).min(self.limit) + self.offset
        };

        if value.abs() > self.cutoff {
            Some(power)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy)]
pub struct RigidBody {
    pub length_boom: f32,
    pub length_arm: f32,
}

#[derive(Clone, Copy)]
pub struct Rig {
    pub angle_slew: Option<f32>,
    pub angle_boom: Option<f32>,
    pub angle_arm: Option<f32>,
    #[allow(dead_code)]
    pub angle_attachment: Option<f32>,
}

impl Rig {
    pub fn angle_boom(&self) -> Option<f32> {
        self.angle_boom
    }

    pub fn angle_arm(&self) -> Option<f32> {
        self.angle_arm
    }

    pub fn angle_slew(&self) -> Option<f32> {
        self.angle_slew
    }
}

// TODO: Rename to domain
pub struct Body {
    rigid: RigidBody,
    chain: Pose,
}

impl Body {
    pub const fn new(rigid: RigidBody) -> Self {
        Self {
            rigid,
            chain: Pose {
                rigid,
                rig: Rig {
                    angle_slew: None,
                    angle_boom: None,
                    angle_arm: None,
                    angle_attachment: None,
                },
            },
        }
    }

    // TODO: const from const module
    pub async fn signal_update(&mut self, reader: &mut crate::signal::SignalReader) {
        use crate::core;
        use crate::core::metric::MetricValue;
        use crate::kernel::excavator::consts::*;
        use crate::signal::Encoder;

        if let Ok(Ok((source, signal))) =
            tokio::time::timeout(std::time::Duration::from_millis(500), reader.recv()).await
        {
            match source {
                super::BODY_PART_BOOM => {
                    if let MetricValue::Angle(value) = signal.value {
                        let encoder = Encoder::new(BOOM_ENCODER_RANGE, BOOM_ANGLE_RANGE);

                        let angle = encoder.scale(value.x as f32);
                        let percentage = encoder.scale_to(100.0, value.x as f32);

                        let angle_offset = core::deg_to_rad(5.3);
                        let angle_at_datum = angle - angle_offset;

                        self.chain.rig.angle_boom = Some(angle_at_datum);

                        debug!(
                            "Boom Encoder: {:?}\tAngle rel.: {:>+5.2}rad {:>+5.2}° {:.1}%\tAngle datum: {:>+5.2}rad {:>+5.2}°",
                            value.x,
                            angle,
                            core::rad_to_deg(angle),
                            percentage,
                            angle_at_datum,
                            core::rad_to_deg(angle_at_datum)
                        );
                    }
                }
                super::BODY_PART_ARM => {
                    if let MetricValue::Angle(value) = signal.value {
                        let encoder = Encoder::new(ARM_ENCODER_RANGE, ARM_ANGLE_RANGE);

                        let angle = encoder.scale(value.x as f32);
                        let percentage = encoder.scale_to(100.0, value.x as f32);

                        let angle_offset = core::deg_to_rad(36.8);
                        let angle_at_datum = -angle_offset - (2.1 - angle);

                        self.chain.rig.angle_arm = Some(angle_at_datum);

                        debug!(
                            "Arm Encoder: {:?}\tAngle rel.: {:>+5.2}rad {:>+5.2}° {:.1}%\tAngle datum: {:>+5.2}rad {:>+5.2}°",
                            value.x,
                            angle,
                            core::rad_to_deg(angle),
                            percentage,
                            angle_at_datum,
                            core::rad_to_deg(angle_at_datum)
                        );
                    }
                }
                super::BODY_PART_FRAME => {
                    if let MetricValue::Angle(value) = signal.value {
                        let encoder = Encoder::new(SLEW_ENCODER_RANGE, SLEW_ANGLE_RANGE);

                        let angle = encoder.scale(value.x as f32);
                        let percentage = encoder.scale_to(100.0, value.x as f32);

                        let angle_at_datum = angle;

                        self.chain.rig.angle_slew = Some(angle_at_datum);

                        debug!(
                            "Turn Encoder: {:?}\tAngle rel.: {:>+5.2}rad {:>+5.2}° {:.1}%",
                            value.x,
                            angle,
                            core::rad_to_deg(angle),
                            percentage,
                        );
                    }
                }
                super::BODY_PART_BUCKET => {
                    if let MetricValue::Angle(value) = signal.value {
                        let encoder = Encoder::new(BUCKET_ENCODER_RANGE, BUCKET_ANGLE_RANGE);

                        let angle = encoder.scale(value.x as f32);
                        let percentage = encoder.scale_to(100.0, value.x as f32);

                        // TODO: Offset is negative.
                        // let angle_offset = core::deg_to_rad(36.8);

                        // TODO: REMOVE MOVE MOVE MOVE MOVE
                        // unsafe {
                        //     AGENT.update_boom_angle(core::deg_to_rad(60.0));
                        //     AGENT.update_arm_angle(core::deg_to_rad(-40.0));
                        //     AGENT.update_slew_angle(0.0);
                        // }
                        // if let Ok(mut model) = self.model.try_write() {
                        //     model.update_boom_angle(core::deg_to_rad(60.0));
                        //     model.update_arm_angle(core::deg_to_rad(-40.0));
                        //     model.update_slew_angle(0.0);
                        // };

                        debug!(
                            "Bucket Encoder: {:?}\tAngle rel.: {:>+5.2}rad {:>+5.2}° {:.1}%",
                            value.x,
                            angle,
                            core::rad_to_deg(angle),
                            percentage,
                        );
                    }
                }
                _ => {}
            }
        }
    }

    pub fn _update_slew_angle(&mut self, angle: f32) {
        self.chain.rig.angle_slew = Some(angle)
    }

    pub fn _update_boom_angle(&mut self, angle: f32) {
        self.chain.rig.angle_boom = Some(angle)
    }

    pub fn _update_arm_angle(&mut self, angle: f32) {
        self.chain.rig.angle_arm = Some(angle)
    }

    #[allow(dead_code)]
    pub fn effector_point(&self) -> Option<nalgebra::Point3<f32>> {
        self.chain.effector_point()
    }

    pub fn effector_point_abs(&self) -> Option<nalgebra::Point3<f32>> {
        if let Some(effector_point) = self.chain.effector_point() {
            Some(nalgebra::Point3::new(
                effector_point.x,
                effector_point.y + super::consts::FRAME_HEIGHT,
                effector_point.z,
            ))
        } else {
            None
        }
    }
}

pub struct Objective {
    body: std::sync::Arc<tokio::sync::RwLock<super::body::Body>>,
    chain: Pose,
}

impl Objective {
    pub fn new(model: std::sync::Arc<tokio::sync::RwLock<super::body::Body>>, rig: Rig) -> Self {
        let rigid = model.try_read().unwrap().rigid;
        Self {
            body: model,
            chain: Pose { rigid, rig },
        }
    }

    pub fn from_point(
        model: std::sync::Arc<tokio::sync::RwLock<super::body::Body>>,
        target: nalgebra::Point3<f32>,
    ) -> Self {
        let target = Pose::from_effector_point(model.try_read().unwrap().rigid, target);
        Self {
            body: model,
            chain: target,
        }
    }

    pub fn erorr_diff(&self) -> Rig {
        let physical_pose = &self.body.try_read().unwrap().chain;

        let rig_error = self.chain.erorr_diff(physical_pose);

        if let Some(angle_boom_error) = rig_error.angle_boom {
            debug!(
                "Physical Boom:  {:>+5.2}rad {:>+5.2}°  Target Boom:  {:>+5.2}rad {:>+5.2}°  Error: {:>+5.2}rad {:>+5.2}°",
                physical_pose.rig.angle_boom.unwrap(),
                crate::core::rad_to_deg(physical_pose.rig.angle_boom.unwrap()),
                self.chain.rig.angle_boom.unwrap(),
                crate::core::rad_to_deg(self.chain.rig.angle_boom.unwrap()),
                angle_boom_error,
                crate::core::rad_to_deg(angle_boom_error)
            );
        }

        if let Some(angle_arm_error) = rig_error.angle_arm {
            debug!(
                "Physical Arm:   {:>+5.2}rad {:>+5.2}°  Target Arm:   {:>+5.2}rad {:>+5.2}°  Error: {:>+5.2}rad {:>+5.2}°",
                physical_pose.rig.angle_arm.unwrap(),
                crate::core::rad_to_deg(physical_pose.rig.angle_arm.unwrap()),
                self.chain.rig.angle_arm.unwrap(),
                crate::core::rad_to_deg(self.chain.rig.angle_arm.unwrap()),
                angle_arm_error,
                crate::core::rad_to_deg(angle_arm_error)
            );
        }

        if let Some(angle_slew_error) = rig_error.angle_slew {
            debug!(
                "Physical Slew:  {:>+5.2}rad {:>+5.2}°  Target Slew:  {:>+5.2}rad {:>+5.2}°  Error: {:>+5.2}rad {:>+5.2}°",
                physical_pose.rig.angle_slew.unwrap(),
                crate::core::rad_to_deg(physical_pose.rig.angle_slew.unwrap()),
                self.chain.rig.angle_slew.unwrap(),
                crate::core::rad_to_deg(self.chain.rig.angle_slew.unwrap()),
                angle_slew_error,
                crate::core::rad_to_deg(angle_slew_error)
            );
        }

        rig_error
    }
}

struct Pose {
    rigid: RigidBody,
    rig: Rig,
}

impl Pose {
    fn from_effector_point(rigid: RigidBody, point: nalgebra::Point3<f32>) -> Self {
        let ik = InverseKinematics::new(rigid.length_boom, rigid.length_arm);

        let (angle_slew, angle_boom, angle_arm) = ik.solve(point).unwrap();

        Self {
            rigid,
            rig: Rig {
                angle_slew: Some(angle_slew),
                angle_boom: Some(angle_boom),
                angle_arm: Some(angle_arm),
                angle_attachment: None,
            },
        }
    }

    pub fn boom_point(&self) -> Option<nalgebra::Point2<f32>> {
        if let Some(angle_boom) = self.rig.angle_boom {
            let x = self.rigid.length_boom * angle_boom.cos();
            let y = self.rigid.length_boom * angle_boom.sin();

            Some(nalgebra::Point2::new(x, y))
        } else {
            None
        }
    }

    pub fn effector_point_flat(&self) -> Option<nalgebra::Point2<f32>> {
        if let (Some(boom_point), Some(angle_arm)) = (self.boom_point(), self.rig.angle_arm) {
            let x = boom_point.x
                + (self.rigid.length_arm * (angle_arm + self.rig.angle_boom.unwrap()).cos());
            let y = boom_point.y
                + (self.rigid.length_arm * (angle_arm + self.rig.angle_boom.unwrap()).sin());

            Some(nalgebra::Point2::new(x, y))
        } else {
            None
        }
    }

    pub fn effector_point(&self) -> Option<nalgebra::Point3<f32>> {
        if let (Some(effector_point), Some(angle_slew)) =
            (self.effector_point_flat(), self.rig.angle_slew)
        {
            let x = effector_point.x * angle_slew.cos();
            let y = effector_point.y;
            let z = effector_point.x * angle_slew.sin();

            Some(nalgebra::Point3::new(x, y, z))
        } else {
            None
        }
    }

    pub fn erorr_diff(&self, rhs: &Self) -> Rig {
        let mut rig_error = Rig {
            angle_slew: None,
            angle_boom: None,
            angle_arm: None,
            angle_attachment: None,
        };

        if let (Some(lhs_angle_slew), Some(rhs_angle_slew)) =
            (self.rig.angle_slew, rhs.rig.angle_slew)
        {
            rig_error.angle_slew = Some(lhs_angle_slew - rhs_angle_slew);
        }

        if let (Some(lhs_angle_boom), Some(rhs_angle_boom)) =
            (self.rig.angle_boom, rhs.rig.angle_boom)
        {
            rig_error.angle_boom = Some(lhs_angle_boom - rhs_angle_boom);
        }

        if let (Some(lhs_angle_arm), Some(rhs_angle_arm)) = (self.rig.angle_arm, rhs.rig.angle_arm)
        {
            rig_error.angle_arm = Some(lhs_angle_arm - rhs_angle_arm);
        }

        rig_error
    }
}
