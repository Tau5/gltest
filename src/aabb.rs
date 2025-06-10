use std::{mem, ptr};
use std::ffi::c_void;
use gl::types::{GLfloat, GLsizei, GLsizeiptr, GLuint};
use nalgebra_glm as glm;
use glm::Vec3;
use nalgebra_glm::vec3;
use crate::objects;
use crate::shader::ShaderProgram;
use crate::utils::gen_buffers;
use crate::vao::{LineLoopVAO, TriangleArrayVAO, VAO};

#[derive(Debug)]
pub struct AABB {
    pub start: Vec3,
    pub end: Vec3,
    //pub position: Vec3,
    //pub size: Vec3,
}

impl AABB {
    pub fn new(start: Vec3, end: Vec3) -> Self {
        Self { start, end }
    }

    pub fn collision(&self, other: &AABB) -> bool {
        let aMinX = self.start.x;
        let aMaxX = self.end.x;
        let bMinX = other.start.x;
        let bMaxX = other.end.x;
        
        //let aMinX = self.position.x;
        //let aMaxX = self.position.x + self.size.x;
        //let bMinX = other.position.x;
        //let bMaxX = other.position.x + other.size.x;

        let res =
            aMinX <= bMaxX
                &&  aMaxX >= bMinX
                &&  self.start.y < other.end.y
                &&  self.end  .y >= other.start.y
                &&  self.start.z <= other.end.z
                &&  self.end  .z >= other.start.z;

        //let res =
        //    aMinX <= bMaxX
        //&&  aMaxX >= bMinX
        //&&  self.position.y < other.position.y + other.size.y
        //&&  self.position.y + self.size.y >= other.position.y
        //&&  self.position.z <= other.position.z + other.size.z
        //&&  self.position.z + self.size.z >= other.position.z;


        println!("Collides Me{:?} with Other{:?}? {}", &self, other, res);
        
        res
    }
    
    pub fn translate(&self, trans: Vec3) -> Self {
        let start = self.start + trans;
        let end = self.end + trans;
        
        AABB::new(start, end)
    }

    pub fn get_vao(&self) -> TriangleArrayVAO {
        return objects::cube();
    }


    pub fn render(&self, vao: &TriangleArrayVAO, shader: &ShaderProgram) {
        let mut model = glm::identity::<f32, 4>();
        let scale = (self.end - self.start)*1.01;
        let position = ((self.start + self.end) / 2.0);

        model = glm::translate(&model, &position);
        model = glm::scale(&model, &scale);

        shader.setMat4(c"model", &model);

        vao.render();
    }

}

