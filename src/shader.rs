use std::ffi::CStr;
use gl::types::{GLfloat, GLint, GLsizei, GLuint};
use nalgebra_glm::TVec3;
use crate::utils::compile_shader;
use crate::textures::Material;

#[derive(Copy, Clone)]
pub struct ShaderProgram {
    id: GLuint,
}

impl ShaderProgram {
    pub fn new(vertex_shader: &str, fragment_shader: &str) -> Self {
        let program_id = unsafe { gl::CreateProgram() };

        let vertex = compile_shader(vertex_shader, gl::VERTEX_SHADER);
        let fragment = compile_shader(fragment_shader, gl::FRAGMENT_SHADER);

        unsafe {
            gl::AttachShader(program_id, vertex);
            gl::AttachShader(program_id, fragment);
            gl::LinkProgram(program_id);

            gl::DeleteShader(vertex);
            gl::DeleteShader(fragment);
        }

        ShaderProgram { id: program_id }
    }

    pub fn load(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    pub fn setFloat(&self, uniform_name: &CStr, value: GLfloat) {
        unsafe {
            let uniform_location = gl::GetUniformLocation(self.id, uniform_name.as_ptr());
            gl::Uniform1f(uniform_location, value);
        }
    }

    pub fn setInt(&self, uniform_name: &CStr, value: GLint) {
        unsafe {
            let uniform_location = gl::GetUniformLocation(self.id, uniform_name.as_ptr());
            gl::Uniform1i(uniform_location, value);
        }
    }

    pub fn setMat4(&self, uniform_name: &CStr, value: &nalgebra_glm::TMat4<GLfloat>) {
        unsafe {
            let uniform_location = gl::GetUniformLocation(self.id, uniform_name.as_ptr());
            gl::UniformMatrix4fv(uniform_location, 1, gl::FALSE, nalgebra_glm::value_ptr(value).as_ptr());
        }
    }

    pub fn setMat3(&self, uniform_name: &CStr, value: &nalgebra_glm::TMat3<GLfloat>) {
        unsafe {
            let uniform_location = gl::GetUniformLocation(self.id, uniform_name.as_ptr());
            gl::UniformMatrix3fv(uniform_location, 1, gl::FALSE, nalgebra_glm::value_ptr(value).as_ptr());
        }
    }

    pub fn setVec3(&self, uniform_name: &CStr, value: &nalgebra_glm::Vec3) {
        unsafe {
            let uniform_location = gl::GetUniformLocation(self.id, uniform_name.as_ptr());
            gl::Uniform3fv(uniform_location, 1, nalgebra_glm::value_ptr(value).as_ptr());
        }
    }

    pub fn setVec4(&self, uniform_name: &CStr, value: &nalgebra_glm::Vec4) {
        unsafe {
            let uniform_location = gl::GetUniformLocation(self.id, uniform_name.as_ptr());
            gl::Uniform4fv(uniform_location, 1, nalgebra_glm::value_ptr(value).as_ptr());
        }
    }

}
