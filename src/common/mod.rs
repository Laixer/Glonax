pub mod position;
pub mod index;
mod ring;

pub use ring::Ring;

#[derive(Debug)]
/// 3 axis vector.
pub struct Vector3<T> {
    x: T,
    y: T,
    z: T,
}
