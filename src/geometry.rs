use nalgebra_glm::{Mat3, Mat4};
use crate::aabb::AABB;
use crate::shader::ShaderProgram;
use crate::textures::MaterialStore;

pub trait Geometry {
    fn render(&self, shader: &ShaderProgram, model: &Mat4, normal: &Mat3, material_store: &MaterialStore);
    fn base_aabb(&self) -> AABB;
}