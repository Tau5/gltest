use nalgebra_glm::{vec3, Vec3};
use russimp::Vector3D;

pub fn vec3ai_to_glm(vec: Vector3D) -> Vec3 {
    vec3(vec.x, vec.y, vec.z)
}