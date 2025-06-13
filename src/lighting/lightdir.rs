use nalgebra_glm::Vec3;
use crate::shader::ShaderProgram;

pub struct DirLight {
    pub direction: Vec3,
    pub ambient: Vec3,
    pub diffuse: Vec3,
    pub specular: Vec3,
}

impl DirLight {
    pub fn load(&self, shader: &ShaderProgram) {
        shader.setVec3(c"directionalLight.direction", &self.direction);
        shader.setVec3(c"directionalLight.ambient", &self.ambient);
        shader.setVec3(c"directionalLight.diffuse", &self.diffuse);
        shader.setVec3(c"directionalLight.specular", &self.specular);
    }

    pub fn new(direction: Vec3, ambient: Vec3, diffuse: Vec3, specular: Vec3) -> Self {
        Self { direction, ambient, diffuse, specular }
    }
}