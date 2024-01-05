use nalgebra::{Matrix4, Point3, Rotation3, Translation3, Vector3};

use crate::core::MachineType;

#[derive(Default)]
pub struct World {
    actors: Vec<Actor>,
}

impl World {
    /// Add actor to world and return index.
    #[inline]
    pub fn add_actor(&mut self, actor: Actor) -> usize {
        self.actors.push(actor);
        self.actors.len() - 1
    }

    /// Retrieve actor by index.
    #[inline]
    pub fn get_actor(&self, index: usize) -> Option<&Actor> {
        self.actors.get(index)
    }

    /// Retrieve actor by index mutably.
    #[inline]
    pub fn get_actor_mut(&mut self, index: usize) -> Option<&mut Actor> {
        self.actors.get_mut(index)
    }

    /// Retrieve actor by name.
    pub fn get_actor_by_name(&self, name: impl ToString) -> Option<&Actor> {
        self.actors
            .iter()
            .find(|actor| actor.name() == name.to_string())
    }

    /// Retrieve actor by name mutably.
    pub fn get_actor_by_name_mut(&mut self, name: impl ToString) -> Option<&mut Actor> {
        self.actors
            .iter_mut()
            .find(|actor| actor.name() == name.to_string())
    }
}

pub struct ActorBuilder {
    /// Actor name.
    name: String,
    /// Actor type.
    ty: MachineType,
    /// Actor segments.
    segments: Vec<(String, ActorSegment)>,
}

impl ActorBuilder {
    pub fn new(name: impl ToString, ty: MachineType) -> Self {
        Self {
            name: name.to_string(),
            ty,
            segments: Vec::new(),
        }
    }

    pub fn attach_segment(mut self, name: impl ToString, segment: ActorSegment) -> Self {
        self.segments.push((name.to_string(), segment));
        self
    }

    pub fn build(self) -> Actor {
        let root = ActorSegment::new(Vector3::new(0.0, 0.0, 0.0));

        Actor {
            name: self.name,
            ty: self.ty,
            segments: if self.segments.is_empty() {
                vec![("root".to_string(), root)]
            } else {
                self.segments
            },
        }
    }
}

// TODO: Convert to and from bytes
#[derive(Clone)]
pub struct Actor {
    /// Actor name.
    name: String,
    /// Actor type.
    ty: MachineType,
    /// Actor segments.
    segments: Vec<(String, ActorSegment)>,
}

impl Actor {
    /// Actor name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Actor type.
    pub fn ty(&self) -> MachineType {
        self.ty
    }

    /// Actor root location.
    pub fn location(&self) -> Point3<f32> {
        self.segments[0].1.location()
    }

    /// Actor root rotation.
    pub fn rotation(&self) -> Rotation3<f32> {
        self.segments[0].1.rotation()
    }

    /// Set actor root location.
    pub fn set_location(&mut self, location: Vector3<f32>) {
        self.segments[0].1.set_location(location);
    }

    /// Set actor root rotation.
    pub fn set_rotation(&mut self, rotation: Rotation3<f32>) {
        self.segments[0].1.set_rotation(rotation);
    }

    pub fn set_relative_rotation(&mut self, name: impl ToString, rotation: Rotation3<f32>) {
        for (sname, segment) in self.segments.iter_mut() {
            if sname == &name.to_string() {
                segment.set_rotation(rotation);
                break;
            }
        }
    }

    pub fn add_relative_rotation(&mut self, name: impl ToString, rotation: Rotation3<f32>) {
        for (sname, segment) in self.segments.iter_mut() {
            if sname == &name.to_string() {
                segment.add_rotation(rotation);
                break;
            }
        }
    }

    pub fn relative_location(&self, name: impl ToString) -> Option<Point3<f32>> {
        for (sname, segment) in &self.segments {
            if sname == &name.to_string() {
                return Some(segment.location());
            }
        }

        None
    }

    pub fn world_location(&self, name: impl ToString) -> Point3<f32> {
        let mut transform = Matrix4::identity();

        for (sname, segment) in self.segments.iter() {
            transform *= segment.transformation();

            if sname == &name.to_string() {
                break;
            }
        }

        transform.transform_point(&Point3::new(0.0, 0.0, 0.0))
    }
}

impl Actor {
    pub fn to_bytes(&self) -> Vec<u8> {
        use bytes::BufMut;

        let mut buf = bytes::BytesMut::with_capacity(64);

        buf.put_u8(self.ty as u8);

        let name_bytes = self.name.as_bytes();
        buf.put_u16(name_bytes.len() as u16);
        buf.put(name_bytes);

        buf.put_u8(self.segments.len() as u8);

        for (name, segment) in &self.segments {
            let name_bytes = name.as_bytes();
            buf.put_u16(name_bytes.len() as u16);
            buf.put(name_bytes);

            buf.put(&segment.to_bytes()[..]);
        }

        buf.to_vec()
    }
}

impl TryFrom<Vec<u8>> for Actor {
    type Error = ();

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        use bytes::Buf;

        let mut buf = bytes::Bytes::copy_from_slice(&value);

        let ty = buf.get_u8();
        let name_len = buf.get_u16() as usize;
        let name = String::from_utf8_lossy(&buf[..name_len]).to_string();
        buf.advance(name_len);

        let segment_count = buf.get_u8() as usize;

        let mut segments = Vec::with_capacity(segment_count);

        for _ in 0..segment_count {
            let name_len = buf.get_u16() as usize;
            let name = String::from_utf8_lossy(&buf[..name_len]).to_string();
            buf.advance(name_len);

            let segment = ActorSegment::try_from(&buf[..]).unwrap();
            buf.advance(24); // TODO: Remove magic number

            segments.push((name, segment));
        }

        // log::debug!("Actor: {} {}", name, ty);
        // log::debug!("segment_count: {}", segment_count);

        // segments.iter().for_each(|(name, segment)| {
        //     let (roll, pitch, yaw) = segment.rotation().euler_angles();
        //     log::debug!("segment: {} R={} P={} Y={}", name, roll, pitch, yaw);
        // });

        Ok(Self {
            name,
            ty: MachineType::try_from(ty)?,
            segments,
        })
    }
}

impl crate::protocol::Packetize for Actor {
    const MESSAGE_TYPE: u8 = 0x69; // 0x40

    fn to_bytes(&self) -> Vec<u8> {
        self.to_bytes()
    }
}

#[derive(Clone)]
pub struct ActorSegment {
    isometry: nalgebra::IsometryMatrix3<f32>,
}

impl ActorSegment {
    pub fn new(location: Vector3<f32>) -> Self {
        Self {
            isometry: nalgebra::IsometryMatrix3::from_parts(
                nalgebra::Translation3::from(location),
                nalgebra::Rotation3::identity(),
            ),
        }
    }

    /// Segment location.
    #[inline]
    pub fn location(&self) -> Point3<f32> {
        self.isometry.translation.vector.into()
    }

    /// Segment rotation.
    #[inline]
    pub fn rotation(&self) -> Rotation3<f32> {
        self.isometry.rotation
    }

    /// Segment transformation.
    #[inline]
    pub fn transformation(&self) -> Matrix4<f32> {
        self.isometry.to_homogeneous()
    }

    /// Set segment absolute location.
    #[inline]
    pub fn set_location(&mut self, location: Vector3<f32>) {
        self.isometry.translation = Translation3::from(location);
    }

    /// Set segment absolute rotation.
    #[inline]
    pub fn set_rotation(&mut self, rotation: Rotation3<f32>) {
        self.isometry.rotation = rotation;
    }

    /// Add segment relative location.
    #[inline]
    pub fn add_location(&mut self, location: Vector3<f32>) {
        self.isometry.translation.vector += location;
    }

    /// Add segment relative rotation.
    #[inline]
    pub fn add_rotation(&mut self, rotation: Rotation3<f32>) {
        self.isometry.rotation *= rotation;
    }
}

impl ActorSegment {
    pub fn to_bytes(&self) -> Vec<u8> {
        use bytes::BufMut;

        let mut buf = bytes::BytesMut::with_capacity(64);

        buf.put_f32(self.isometry.translation.vector.x);
        buf.put_f32(self.isometry.translation.vector.y);
        buf.put_f32(self.isometry.translation.vector.z);

        let (roll, pitch, yaw) = self.isometry.rotation.euler_angles();
        buf.put_f32(roll);
        buf.put_f32(pitch);
        buf.put_f32(yaw);

        buf.to_vec()
    }
}

impl TryFrom<&[u8]> for ActorSegment {
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        use bytes::Buf;

        let mut buf = bytes::Bytes::copy_from_slice(value);

        let translation = Vector3::new(buf.get_f32(), buf.get_f32(), buf.get_f32());
        let rotation = Rotation3::from_euler_angles(buf.get_f32(), buf.get_f32(), buf.get_f32());

        Ok(Self {
            isometry: nalgebra::IsometryMatrix3::from_parts(
                nalgebra::Translation3::from(translation),
                rotation,
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_actor_segment() {
        let segment = ActorSegment::new(Vector3::new(1.0, 2.0, 3.0));

        assert_eq!(segment.location(), Point3::new(1.0, 2.0, 3.0));
        assert_eq!(segment.rotation(), Rotation3::identity());

        let mut segment = ActorSegment::new(Vector3::new(1.0, 2.0, 3.0));

        segment.set_location(Vector3::new(4.0, 5.0, 6.0));
        segment.set_rotation(Rotation3::from_euler_angles(0.0, 0.0, std::f32::consts::PI));

        assert_eq!(segment.location(), Point3::new(4.0, 5.0, 6.0));
        assert_eq!(
            segment.rotation(),
            Rotation3::from_euler_angles(0.0, 0.0, std::f32::consts::PI)
        );

        segment.add_location(Vector3::new(1.0, 2.0, 3.0));
        segment.add_rotation(Rotation3::from_euler_angles(0.0, 0.0, std::f32::consts::PI));

        assert_eq!(segment.location(), Point3::new(5.0, 7.0, 9.0));
        assert_eq!(
            segment.rotation(),
            Rotation3::from_euler_angles(0.0, 0.0, 2.0 * std::f32::consts::PI)
        );
    }
}
