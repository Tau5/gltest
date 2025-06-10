use gl::types::{GLint, GLuint};

pub type VAOId = GLuint;

pub trait VAO {
    fn render(&self);
}

pub struct TriangleArrayVAO {
    triangles: GLint,
    id: VAOId
}

impl TriangleArrayVAO {
    pub fn new(id: VAOId, triangles: GLint) -> Self {
        Self { triangles, id }
    }
}

impl VAO for TriangleArrayVAO {
    fn render(&self) {
        unsafe {
            gl::BindVertexArray(self.id);
            gl::DrawArrays(gl::TRIANGLES, 0, self.triangles);
            gl::BindVertexArray(0);
        }
    }
}

pub struct LineLoopVAO {
    lines: GLint,
    id: VAOId
}

impl LineLoopVAO {
    pub fn new(id: VAOId, lines: GLint) -> Self {
        Self { lines, id }
    }
}

impl VAO for LineLoopVAO {
    fn render(&self) {
        unsafe {
            gl::BindVertexArray(self.id);
            gl::DrawArrays(gl::TRIANGLES, 0, self.lines);
            gl::BindVertexArray(0);
        }
    }
}