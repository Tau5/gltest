use nalgebra_glm::{Mat3, Mat4};
use russimp::node::Node;
use russimp::scene;
use crate::mesh::Mesh;
use crate::shader::ShaderProgram;
use crate::vao::VAO;
use russimp::scene::{PostProcess, Scene};
use crate::aabb::AABB;
use crate::geometry::Geometry;
use crate::textures::{Material, MaterialStore};

pub struct Model {
    pub meshes: Vec<Mesh>,
    path: String,
}

impl Model {

    fn process_node(&mut self, node: &Node, scene: &Scene, material_store: &mut MaterialStore, id: String) {
        for mesh_idx in &node.meshes {
            if let Some(mesh) = scene.meshes.get(*mesh_idx as usize) {
                self.meshes.push(Mesh::from(mesh, scene, material_store, id.clone(), node.transformation));
            }
        }

        for child in node.children.borrow().iter() {
            self.process_node(child, scene, material_store, id.clone());
        }
    }

    pub fn new(path: String, id: String, material_store: &mut MaterialStore) -> Self {
        let scene = Scene::from_file(&path, vec![
            PostProcess::GenerateNormals,
            PostProcess::Triangulate
        ]).unwrap();

        let mut out = Self {
            meshes: Vec::new(),
            path
        };

        let root = &scene.root.clone().unwrap();
        out.process_node(root.as_ref(), &scene, material_store, id);



        out
    }

}

impl Geometry for Model {
    fn render(&self, shader: &ShaderProgram, model: &Mat4, normal: &Mat3, material_store: &MaterialStore) {
        shader.setMat4(c"model", model);
        shader.setMat3(c"normalMatrix", &normal);
        for mesh in &self.meshes {
            mesh.render(shader);
        }
    }

    fn base_aabb(&self) -> Vec<AABB> {
        self.meshes
            .iter()
            .map(|f| f.aabb)
            .collect::<Vec<AABB>>()
    }
}