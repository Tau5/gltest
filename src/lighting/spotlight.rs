use nalgebra_glm::Vec3;
use gl::types::GLfloat;
use crate::shader::ShaderProgram;

pub struct SpotLight {
    pub position: Vec3,
    pub direction: Vec3,

    // NOTE: Variable says angle, but has to be stored as cos(angle)
    // to not incur a penalty on the gpu by doing a reverse cosine

    pub angle_inner: GLfloat,
    pub angle_outer: GLfloat,

    pub ambient: Vec3,
    pub diffuse: Vec3,
    pub specular: Vec3,
}

impl SpotLight {
    pub fn load(&self, shader: &ShaderProgram) {
        shader.setInt(c"spotLightEnable", 1);
        shader.setVec3(c"spotLight.position", &self.position);
        shader.setVec3(c"spotLight.direction", &self.direction);
        shader.setVec3(c"spotLight.ambient", &self.ambient);
        shader.setVec3(c"spotLight.diffuse", &self.diffuse);
        shader.setVec3(c"spotLight.specular", &self.specular);
        shader.setFloat(c"spotLight.angleInner", self.angle_inner);
        shader.setFloat(c"spotLight.angleOuter", self.angle_outer);
    }

    pub fn new(position: Vec3, direction: Vec3, angle_inner: GLfloat, angle_outer: GLfloat, ambient: Vec3, diffuse: Vec3, specular: Vec3) -> Self {
        let angle_inner = GLfloat::cos(angle_inner);
        let angle_outer = GLfloat::cos(angle_outer);

        Self { position, direction, angle_inner, angle_outer, ambient, diffuse, specular }
    }
}