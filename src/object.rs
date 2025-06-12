use gl::types::{GLfloat, GLint, GLuint};
use nalgebra_glm::{proj, Mat3, Mat4, TVec3, Vec3};
use crate::textures::*;

use nalgebra_glm as glm;
use crate::aabb::AABB;
use crate::collidable::Collidable;
use crate::shader::ShaderProgram;
use crate::vao::{TriangleArrayVAO, VAO};

pub struct Object {
    translate: Vec3,
    rotation: Vec3,
    scale: Vec3,
    vao: Box<dyn VAO>,
    material: Material,
    model: Mat4,
    normal: Mat3,
    has_collision: bool,
    aabb: AABB,
    aaab_vao: Option<TriangleArrayVAO>
}

impl Object {
    pub fn new(translate: Vec3, rotation: Vec3, scale: Vec3, vao: Box<dyn VAO>, material: Material, has_collision: bool) -> Self {
        let mut out = Self {
            translate,
            rotation,
            scale,
            vao,
            material,
            model: glm::identity::<f32, 4>(),
            normal: glm::identity::<f32, 3>(),
            has_collision,
            aabb: AABB::new(
                -(scale / 2.0),
                scale / 2.0).translate(translate),
            aaab_vao: None,
        };
        out.update_model();
        out.update_aabb();
        out.aaab_vao = Some(out.aabb.get_vao());

        out
    }

    pub fn set_translate(&mut self, translate: Vec3) {
        self.translate = translate;
        self.update_model();
    }

    pub fn set_rotation(&mut self, rotation: Vec3) {
        self.translate = rotation;
    }

    pub fn set_scale(&mut self, scale: Vec3) {
        self.translate = scale;
        self.update_model();
    }

    pub fn translate(&mut self, x: GLfloat, y: GLfloat, z: GLfloat) {
        self.translate.x += x;
        self.translate.y += y;
        self.translate.z += z;
        self.update_model();
    }

    pub fn rotate(&mut self, x: GLfloat, y: GLfloat, z: GLfloat) {
        self.translate.x += x;
        self.translate.y += y;
        self.translate.z += z;
        self.update_model();
    }

    pub fn scale(&mut self, x: GLfloat, y: GLfloat, z: GLfloat) {
        self.scale.x += x;
        self.scale.y += y;
        self.scale.z += z;
        self.update_model();
    }

    fn update_aabb(&mut self) {
        self.aabb.start = -(self.scale / 2.0);
        self.aabb.end = self.scale / 2.0;
        self.aabb = self.aabb.translate(self.translate);
    }

    fn update_model(&mut self) {
        self.model = glm::identity::<f32, 4>();
        self.model = glm::translate(&self.model, &self.translate);
        self.model = glm::scale(&self.model, &self.scale);
        self.normal = glm::mat4_to_mat3(&glm::inverse_transpose(self.model));
        
        self.update_aabb();
        // TODO: Implement rotation! (Requires quaternions and math thingies)
    }

    pub fn render(&self, shader: &ShaderProgram) {
        shader.setMat4(c"model", &self.model);
        shader.setMat3(c"normalMatrix", &self.normal);
        shader.setFloat(c"factor", self.material.factor);

        if let Some(diffuse_map) = &self.material.diffuse_map {
            diffuse_map.bind(gl::TEXTURE0);
            shader.setInt(c"material.diffuse", 0);
        }

        if let Some(specular_map) = &self.material.specular_map {
            specular_map.bind(gl::TEXTURE1);
            shader.setInt(c"material.specular", 1);
        }

        if let Some(emission_map) = &self.material.emission_map {
            emission_map.bind(gl::TEXTURE2);
            shader.setInt(c"material.emission", 2);
        }
        
        shader.setFloat(c"material.shininess", 32.0);

        self.vao.render();

        if let Some(diffuse_map) = &self.material.diffuse_map {
            diffuse_map.unbind(gl::TEXTURE0);
        }
        if let Some(specular_map) = &self.material.diffuse_map {
            specular_map.unbind(gl::TEXTURE1);
        }
        if let Some(emission_map) = &self.material.diffuse_map {
            emission_map.unbind(gl::TEXTURE2);
        }
    }

    pub fn debug_render_aabb(&self, shader_program: &ShaderProgram) {
        //if self.aaab_vao.is_none() {
        //    self.aaab_vao = Some(self.aabb.get_vao());
        //}

        if let Some(vao) = &self.aaab_vao {
            self.aabb.render(vao, shader_program);
        }
    }

}

impl Collidable for Object {
    fn collides(&self, other: &AABB) -> bool {
        if !self.has_collision {
            false
        } else { 
            self.aabb.collision(other)
        }
    }
}