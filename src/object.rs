use std::iter;
use gl::types::{GLfloat, GLint, GLuint};
use nalgebra_glm::{mat4_to_mat3, proj, translate, vec3, Mat3, Mat4, TVec3, Vec3};
use crate::textures::*;

use nalgebra_glm as glm;
use crate::aabb::AABB;
use crate::collidable::Collidable;
use crate::{geometry, prefabs};
use crate::geometry::Geometry;
use crate::mesh::Mesh;
use crate::model::Model;
use crate::shader::ShaderProgram;
use crate::vao::{TriangleArrayVAO, VAO};

pub struct Object {
    translate: Vec3,
    rotation: glm::Quat,
    scale: Vec3,
    model: Mat4,
    normal: Mat3,
    has_collision: bool,
    aabb: Vec<AABB>,
    aabb_vao: Option<TriangleArrayVAO>,
    geometry: Box<dyn Geometry>,
    base_aabb: Vec<AABB>
}

impl Object {
    pub(crate) fn render_scaled(&mut self, shader: &ShaderProgram, mat_store: &MaterialStore) {
        let scale = self.scale;

        let new_scale = self.scale * 1.02;
        self.scale = new_scale;

        self.update_model();
        self.render(shader, mat_store);

        self.scale = scale;
        self.update_model();
    }

    pub fn get_scale(&self) -> Vec3 {
        self.scale
    }

    pub fn get_size(&self) -> Vec3 {
        let mut max = glm::vec3(-f32::INFINITY, -f32::INFINITY, -f32::INFINITY);
        let mut min = glm::vec3(f32::INFINITY, f32::INFINITY, f32::INFINITY);

        for aabb in &self.aabb {
            if aabb.start.x < min.x { min.x = aabb.start.x };
            if aabb.start.x < min.x { min.x = aabb.start.x };
            if aabb.start.y < min.y { min.y = aabb.start.y };
            if aabb.end.z < min.z { min.z = aabb.end.z };
            if aabb.end.y < min.y { min.y = aabb.end.y };
            if aabb.end.z < min.z { min.z = aabb.end.z };

            if aabb.start.x > max.x { max.x = aabb.start.x };
            if aabb.start.x > max.x { max.x = aabb.start.x };
            if aabb.start.y > max.y { max.y = aabb.start.y };
            if aabb.end.z > max.z { max.z = aabb.end.z };
            if aabb.end.y > max.y { max.y = aabb.end.y };
            if aabb.end.z > max.z { max.z = aabb.end.z };
        }

        (max - min).abs()
    }
}

impl Object {
    pub fn new(translate: Vec3, scale: Vec3, geometry: Box<dyn Geometry>, has_collision: bool) -> Self {
        let base_aabb = geometry.base_aabb();
        let rotation = glm::Quat::identity();

        let mut out = Self {
            translate,
            rotation,
            scale,
            geometry,
            model: glm::identity::<f32, 4>(),
            normal: glm::identity::<f32, 3>(),
            has_collision,
            aabb: base_aabb.clone(),
            aabb_vao: None,
            base_aabb
        };

        out.update_model();
        out.update_aabb();
        out.aabb_vao = Some(prefabs::cube());

        out
    }

    pub fn set_translate(&mut self, translate: Vec3) {
        self.translate = translate;
        self.update_model();
    }

    pub fn set_rotation(&mut self, rotation: glm::Quat) {
        self.rotation = rotation;
        self.update_model();
    }

    pub fn set_scale(&mut self, scale: Vec3) {
        self.scale = scale;
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
        for (i, base_aabb) in self.base_aabb.iter().enumerate() {
            self.aabb[i].start.x = base_aabb.start.x * self.scale.x;
            self.aabb[i].start.y = base_aabb.start.y * self.scale.y;
            self.aabb[i].start.z = base_aabb.start.z * self.scale.z;
            self.aabb[i].end.x = self.scale.x * base_aabb.end.x;
            self.aabb[i].end.y = self.scale.y * base_aabb.end.y;
            self.aabb[i].end.z = self.scale.z * base_aabb.end.z;
            self.aabb[i] = self.aabb[i].translate(self.translate);
        }

    }

    fn update_model(&mut self) {
        self.model = glm::Mat4::identity();
        self.model = glm::translate(&self.model, &self.translate) ;
        self.model *= glm::quat_to_mat4(&self.rotation);
        self.model = glm::scale(&self.model, &self.scale);
        self.normal = glm::mat4_to_mat3(&glm::inverse_transpose(self.model));
        
        self.update_aabb();
        // TODO: Implement rotation! (Requires quaternions and math thingies)
    }

    pub fn render(&self, shader: &ShaderProgram, material_store: &MaterialStore) {
        self.geometry.render(shader, &self.model, &self.normal, material_store);
    }

    pub fn debug_render_aabb(&self, shader_program: &ShaderProgram) {
        //if self.aaab_vao.is_none() {
        //    self.aaab_vao = Some(self.aabb.get_vao());
        //}

        if let Some(vao) = &self.aabb_vao {
            for aabb in &self.aabb {
                aabb.render(vao, shader_program);
            }
        }
    }

    pub fn from_model(translate: Vec3, rotation: Vec3, scale: Vec3, has_collision: bool, model: Model, material_store: &MaterialStore) -> Self {
        Self::new(translate, scale, Box::from(model), has_collision)
    }

}

impl Collidable for Object {
    fn collides(&self, other: &AABB) -> bool {
        if !self.has_collision {
            false
        } else {
            self.aabb.iter().any(|f| f.collision(other))
        }
    }
}