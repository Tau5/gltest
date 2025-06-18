use nalgebra_glm::{vec3, Mat3, Mat4};
use crate::aabb::AABB;
use crate::geometry::Geometry;
use crate::prefabs;
use crate::shader::ShaderProgram;
use crate::textures::{BindableTexture, Material, MaterialStore};
use crate::vao::VAO;

pub struct Cube {
    vao: Box<dyn VAO>,
    material: Material
}

impl Cube {
    pub fn new(material: Material) -> Self {
        Self { vao: Box::new(prefabs::cube()), material }
    }
}

impl Geometry for Cube {
    fn render(&self, shader: &ShaderProgram, model: &Mat4, normal: &Mat3, material_store: &MaterialStore) {
        shader.setMat4(c"model", model);
        shader.setMat3(c"normalMatrix", normal);
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

    fn base_aabb(&self) -> AABB {
        AABB::new(
            vec3(-0.5, -0.5, -0.5),
            vec3(0.5, 0.5, 0.5)
        )
    }
}