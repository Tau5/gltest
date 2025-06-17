use std::ffi::c_void;
use std::{mem, ptr};
use std::cmp::Ordering;
use std::fmt::format;
use fastrand::usize;
use gl::types::{GLfloat, GLint, GLsizei, GLsizeiptr, GLuint};
use image::EncodableLayout;
use nalgebra_glm::{Vec2, Vec3};
use russimp::material::TextureType;
use crate::shader::ShaderProgram;
use crate::textures::{BindableTexture, GLTexture, Material, MaterialStore};
use crate::utils::gen_buffers;
use crate::vao::VAO;

#[derive(PartialEq, PartialOrd)]
#[repr(C)]
pub struct Vertex {
    position: Vec3,
    tex_coords: Vec2,
    normal: Vec3,
}

impl Vertex {
    pub fn as_bytes(&self) -> [GLfloat; 8] {
        [
            self.position.x,    self.position.y,    self.position.z,
            self.tex_coords.x,  self.tex_coords.y,
            self.normal.x,      self.normal.y,      self.normal.z,
        ]
    }
}

pub struct Texture {
    id: usize,
    r#type: String
}

pub struct Mesh {
    vertices: Vec<Vertex>,
    indices: Vec<GLuint>,
    pub(crate) material_id: String,
    material: Material,
    pub min_position: Vec3,
    pub max_position: Vec3,
    vao: GLuint,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<GLuint>, material_id: String, material: Material) -> Self {
        let mut min = vertices[0].position;
        let mut max = vertices[0].position;

        for vtx in &vertices {
            if vtx.position > min {
                min = vtx.position;
            }
        }

        for vtx in &vertices {
            if vtx.position > max {
                max = vtx.position;
            }
        }

        let mut obj = Self { vertices, indices, material_id , material, min_position: min, max_position: max, vao: 0 };
        obj.setup_mesh();

        obj
    }

    pub fn setup_mesh(&mut self) {
        let mut vbo: GLuint = 1;
        let mut vao: GLuint = 1;
        let mut ebo: GLuint = 1;

        let mut buffer: Vec<GLfloat> = Vec::new();

        for v in &self.vertices {
            for b in v.as_bytes() {
                buffer.push(b);
            }
        }

        unsafe {
            gl::GenVertexArrays(1, ptr::from_mut(&mut vao));
            vbo = gen_buffers(1);
            ebo = gen_buffers(1);
            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(gl::ARRAY_BUFFER, (size_of::<[GLfloat; 8]>() * self.vertices.len()) as GLsizeiptr, buffer.as_bytes().as_ptr() as *const c_void, gl::STATIC_DRAW);

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);

            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, (size_of::<GLuint>() * self.indices.len()) as GLsizeiptr, self.indices.as_slice().as_ptr() as *const c_void, gl::STATIC_DRAW);

            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, size_of::<[GLfloat; 8]>() as GLsizei, 0 as *mut c_void);

            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, size_of::<[GLfloat; 8]>() as GLsizei, size_of::<[GLfloat; 3]>() as *mut c_void);

            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(2, 3, gl::FLOAT, gl::FALSE, size_of::<[GLfloat; 8]>() as GLsizei, size_of::<[GLfloat; 5]>() as *mut c_void);

            self.vao = vao;
        }
    }

    pub(crate) fn render(&self, shader: &ShaderProgram) {
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

        unsafe {
            gl::BindVertexArray(self.vao);
            gl::DrawElements(gl::TRIANGLES, self.indices.len() as GLsizei, gl::UNSIGNED_INT, 0 as *const c_void);
            gl::BindVertexArray(0);
        }

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
}



impl Mesh {
    pub fn from(mesh: &russimp::mesh::Mesh, scene: &russimp::scene::Scene, material_store: &mut MaterialStore, model_name: String) -> Self {
        let mut vertices = Vec::new();

        for (i, vertex) in mesh.vertices.iter().enumerate() {
            let vtx = Vec3::new(vertex.x, vertex.y, vertex.z);
            let normal = mesh.normals[i];
            let normal = Vec3::new(normal.x, normal.y, normal.z);

            if let Some(t) = &mesh.texture_coords[0] {
                if let Some(tex) = t.get(i) {
                    vertices.push(
                        Vertex {
                            position: vtx,
                            tex_coords: Vec2::new(tex.x, tex.y),
                            normal,
                        }
                    )
                }
            }

        }

        let indices = mesh
            .faces
            .iter()
            .flat_map(|face| face.0.iter().copied())
            .collect();

        let material = scene.materials.get(mesh.material_index as usize).unwrap();

        let diffuse = material.textures.get(&TextureType::Diffuse)
            .and_then(|f|
                          Some(format!("textures/{}.{}", f.borrow().filename.clone(), f.borrow().ach_format_hint))
            );
        let specular = material.textures.get(&TextureType::Specular)
            .and_then(|f|
                          Some(format!("textures/{}.{}", f.borrow().filename.clone(), f.borrow().ach_format_hint))
            );
        let emission = material.textures.get(&TextureType::EmissionColor)
            .and_then(|f|
                Some(format!("textures/{}.{}", f.borrow().filename.clone(), f.borrow().ach_format_hint))
            );

        let material_id = material_store.load(format!("{}mat{}", model_name, mesh.material_index).as_str(),
            diffuse.as_deref(), specular.as_deref(), emission.as_deref(), 1.0
        ).unwrap();

        //let material_id= String::from("fallback");

        let material = material_store.get(&material_id);

        Mesh::new(vertices, indices, material_id, material)
    }
}