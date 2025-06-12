use gl::types::GLfloat;
use crate::textures::GLTexture;

#[derive(Copy, Clone)]
pub struct Material {
    pub diffuse_map: Option<GLTexture>,
    pub specular_map: Option<GLTexture>,
    pub emission_map: Option<GLTexture>,
    pub factor: GLfloat
}