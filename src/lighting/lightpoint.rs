use std::ffi::CString;
use nalgebra_glm::Vec3;
use gl::types::{GLfloat, GLint};
use crate::shader::ShaderProgram;

pub struct PointLight {
    pub position: Vec3,
    pub ambient: Vec3,
    pub diffuse: Vec3,
    pub specular: Vec3,

    pub constant: GLfloat,
    pub linear: GLfloat,
    pub quadratic: GLfloat,
}

impl PointLight {
    fn get_uniform_name(index: usize, property: &str) -> CString {
        CString::new(format!("pointLights[{}].{}", index, property)).unwrap()
    }
    pub fn load(&self, index: usize, shader: &ShaderProgram) {
        shader.setVec3(
            PointLight::get_uniform_name(index, "position").as_c_str(),
            &self.position,
        );
        shader.setVec3(
            PointLight::get_uniform_name(index, "ambient").as_c_str(),
            &self.ambient,
        );
        shader.setVec3(
            PointLight::get_uniform_name(index, "diffuse").as_c_str(),
            &self.diffuse,
        );
        shader.setVec3(
            PointLight::get_uniform_name(index, "specular").as_c_str(),
            &self.specular,
        );
        shader.setFloat(
            PointLight::get_uniform_name(index, "constant").as_c_str(),
            self.constant,
        );
        shader.setFloat(
            PointLight::get_uniform_name(index, "linear").as_c_str(),
            self.linear,
        );
        shader.setFloat(
            PointLight::get_uniform_name(index, "quadratic").as_c_str(),
            self.quadratic,
        );
    }

    pub fn new(position: Vec3, ambient: Vec3, diffuse: Vec3, specular: Vec3, constant: GLfloat, linear: GLfloat, quadratic: GLfloat) -> Self {
        Self { position, ambient, diffuse, specular, constant, linear, quadratic }
    }
}