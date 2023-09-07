use glonax::core::geometry::EulerAngles;
use nalgebra::UnitQuaternion;

use crate::Target;

pub(super) struct KinematicRotation {
    pub(super) frame: UnitQuaternion<f32>,
    pub(super) boom: UnitQuaternion<f32>,
    pub(super) arm: UnitQuaternion<f32>,
    pub(super) attachment: Option<UnitQuaternion<f32>>,
}

pub(super) struct ExcavatorIK {
    l1: f32,
    l2: f32,
}

impl ExcavatorIK {
    pub(super) fn new(l1: f32, l2: f32) -> Self {
        Self { l1, l2 }
    }

    pub(super) fn solve(&self, target: &Target) -> std::result::Result<KinematicRotation, ()> {
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

            let abs_pitch_attachment = -theta_2 + theta_3;
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

        Ok(KinematicRotation {
            frame: UnitQuaternion::from_yaw(theta_1),
            boom: UnitQuaternion::from_pitch(-theta_2 + 59.35_f32.to_radians()),
            arm: UnitQuaternion::from_pitch(theta_3),
            attachment: theta_4
                .map(|theta_4| UnitQuaternion::from_pitch(theta_4 + 55_f32.to_radians())),
        })
    }
}
