use nalgebra::{Matrix4, Point3, Rotation3, Translation3, Vector3};

#[derive(Default)]
pub struct World {
    actors: Vec<Actor>,
}

impl World {
    pub fn add_actor(&mut self, actor: Actor) {
        self.actors.push(actor);
    }

    pub fn get_actor(&self, index: usize) -> Option<&Actor> {
        self.actors.get(index)
    }

    pub fn get_actor_mut(&mut self, index: usize) -> Option<&mut Actor> {
        self.actors.get_mut(index)
    }

    pub fn clear(&mut self) {
        self.actors.clear();
    }
}

#[derive(Default)]
pub struct ActorBuilder {
    segments: Vec<(String, ActorSegment)>,
}

impl ActorBuilder {
    pub fn new(segments: Vec<(String, ActorSegment)>) -> Self {
        Self { segments }
    }

    pub fn attach_segment(mut self, name: impl ToString, segment: ActorSegment) -> Self {
        self.segments.push((name.to_string(), segment));
        self
    }

    pub fn build(self) -> Actor {
        let root = ActorSegment::new(Vector3::new(0.0, 0.0, 0.0));

        Actor {
            segments: if self.segments.is_empty() {
                vec![("root".to_string(), root)]
            } else {
                self.segments
            },
        }
    }
}

// TODO: Convert to and from bytes
// TODO: Add name, type, and id to actor
#[derive(Clone)]
pub struct Actor {
    segments: Vec<(String, ActorSegment)>,
}

impl Actor {
    pub fn location(&self) -> Point3<f32> {
        self.segments[0].1.location()
    }

    pub fn rotation(&self) -> Rotation3<f32> {
        self.segments[0].1.rotation()
    }

    pub fn set_location(&mut self, location: Vector3<f32>) {
        self.segments[0].1.set_location(location);
    }

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

    #[inline]
    pub fn location(&self) -> Point3<f32> {
        self.isometry.translation.vector.into()
    }

    #[inline]
    pub fn rotation(&self) -> Rotation3<f32> {
        self.isometry.rotation
    }

    #[inline]
    pub fn transformation(&self) -> Matrix4<f32> {
        self.isometry.to_homogeneous()
    }

    #[inline]
    pub fn set_location(&mut self, location: Vector3<f32>) {
        self.isometry.translation = Translation3::from(location);
    }

    #[inline]
    pub fn set_rotation(&mut self, rotation: Rotation3<f32>) {
        self.isometry.rotation = rotation;
    }

    #[inline]
    pub fn add_location(&mut self, location: Vector3<f32>) {
        self.isometry.translation.vector += location;
    }

    #[inline]
    pub fn add_rotation(&mut self, rotation: Rotation3<f32>) {
        self.isometry.rotation *= rotation;
    }
}
