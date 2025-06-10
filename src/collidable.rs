use crate::aabb::AABB;

pub trait Collidable {
    fn collides(&self, other: &AABB) -> bool;
}