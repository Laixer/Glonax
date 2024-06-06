use nalgebra::Vector3;

use crate::{
    core::{MachineType, Object},
    runtime::{CommandSender, NullConfig, Service, ServiceContext, SignalReceiver},
    world::{Actor, ActorBuilder, ActorSegment},
};

const ROBOT_ACTOR_NAME: &str = "volvo_ec240cl";

const ENCODER_FRAME: u8 = 0x6A;
const ENCODER_BOOM: u8 = 0x6B;
const ENCODER_ARM: u8 = 0x6C;
const ENCODER_ATTACHMENT: u8 = 0x6D;

pub struct Pilot {
    actor: Actor,
}

impl Service<NullConfig> for Pilot {
    fn new(_: NullConfig) -> Self
    where
        Self: Sized,
    {
        // TODO: Build the actor from configuration and machine instance
        let actor = ActorBuilder::new(ROBOT_ACTOR_NAME, MachineType::Excavator)
            .attach_segment(
                "undercarriage",
                ActorSegment::new(Vector3::new(0.0, 0.0, 0.0)),
            )
            .attach_segment("frame", ActorSegment::new(Vector3::new(-4.0, 5.0, 107.0)))
            .attach_segment("boom", ActorSegment::new(Vector3::new(4.0, 20.0, 33.0)))
            .attach_segment("arm", ActorSegment::new(Vector3::new(510.0, 20.0, 5.0)))
            .attach_segment(
                "attachment",
                ActorSegment::new(Vector3::new(310.0, -35.0, 45.0)),
            )
            .build();

        Self { actor }
    }

    fn ctx(&self) -> ServiceContext {
        ServiceContext::new("pilot")
    }

    async fn wait_io_sub(&mut self, _command_tx: CommandSender, mut signal_rx: SignalReceiver) {
        while let Ok(Object::Rotator(rotator)) = signal_rx.recv().await {
            if rotator.source == ENCODER_FRAME {
                self.actor.set_relative_rotation("frame", rotator.rotator);
            }

            if rotator.source == ENCODER_BOOM {
                self.actor.set_relative_rotation("boom", rotator.rotator);
            }

            if rotator.source == ENCODER_ARM {
                self.actor.set_relative_rotation("arm", rotator.rotator);
            }

            if rotator.source == ENCODER_ATTACHMENT {
                self.actor
                    .set_relative_rotation("attachment", rotator.rotator);
            }

            let body_world_location = self.actor.world_location("frame");
            trace!(
                "Frame: X={:.2} Y={:.2} Z={:.2}",
                body_world_location.x,
                body_world_location.y,
                body_world_location.z
            );

            let boom_world_location = self.actor.world_location("boom");
            trace!(
                "Boom: X={:.2} Y={:.2} Z={:.2}",
                boom_world_location.x,
                boom_world_location.y,
                boom_world_location.z
            );

            let arm_world_location = self.actor.world_location("arm");
            trace!(
                "Arm: X={:.2} Y={:.2} Z={:.2}",
                arm_world_location.x,
                arm_world_location.y,
                arm_world_location.z
            );

            let bucket_world_location = self.actor.world_location("attachment");
            trace!(
                "Attachment: X={:.2} Y={:.2} Z={:.2}",
                bucket_world_location.x,
                bucket_world_location.y,
                bucket_world_location.z
            );
        }
    }
}
