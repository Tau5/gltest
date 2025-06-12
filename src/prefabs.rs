use std::ffi::c_void;
use std::{mem, ptr};
use std::process::id;
use gl::types::{GLfloat, GLsizei, GLsizeiptr, GLuint};
use crate::start;
use crate::utils::gen_buffers;
use crate::vao::{TriangleArrayVAO, VAOId};

pub fn cube() -> TriangleArrayVAO {
    let vertex_data: [GLfloat; 36 * 8] = [
        -0.5, -0.5, -0.5,  0.0, 0.0,  0.0,  0.0, -1.0,
        0.5, -0.5, -0.5,  1.0, 0.0,   0.0,  0.0, -1.0,
        0.5,  0.5, -0.5,  1.0, 1.0,   0.0,  0.0, -1.0,
        0.5,  0.5, -0.5,  1.0, 1.0,   0.0,  0.0, -1.0,
        -0.5,  0.5, -0.5,  0.0, 1.0,  0.0,  0.0, -1.0,
        -0.5, -0.5, -0.5,  0.0, 0.0,  0.0,  0.0, -1.0,

        -0.5, -0.5,  0.5,  0.0, 0.0,  0.0,  0.0, 1.0,
        0.5, -0.5,  0.5,  1.0, 0.0,   0.0,  0.0, 1.0,
        0.5,  0.5,  0.5,  1.0, 1.0,   0.0,  0.0, 1.0,
        0.5,  0.5,  0.5,  1.0, 1.0,   0.0,  0.0, 1.0,
        -0.5,  0.5,  0.5,  0.0, 1.0,  0.0,  0.0, 1.0,
        -0.5, -0.5,  0.5,  0.0, 0.0,  0.0,  0.0, 1.0,

        -0.5,  0.5,  0.5,  1.0, 0.0,  -1.0,  0.0,  0.0,
        -0.5,  0.5, -0.5,  1.0, 1.0,  -1.0,  0.0,  0.0,
        -0.5, -0.5, -0.5,  0.0, 1.0,  -1.0,  0.0,  0.0,
        -0.5, -0.5, -0.5,  0.0, 1.0,  -1.0,  0.0,  0.0,
        -0.5, -0.5,  0.5,  0.0, 0.0,  -1.0,  0.0,  0.0,
        -0.5,  0.5,  0.5,  1.0, 0.0,  -1.0,  0.0,  0.0,

        0.5,  0.5,  0.5,  1.0, 0.0,   1.0,  0.0,  0.0,
        0.5,  0.5, -0.5,  1.0, 1.0,   1.0,  0.0,  0.0,
        0.5, -0.5, -0.5,  0.0, 1.0,   1.0,  0.0,  0.0,
        0.5, -0.5, -0.5,  0.0, 1.0,   1.0,  0.0,  0.0,
        0.5, -0.5,  0.5,  0.0, 0.0,   1.0,  0.0,  0.0,
        0.5,  0.5,  0.5,  1.0, 0.0,   1.0,  0.0,  0.0,

        -0.5, -0.5, -0.5,  0.0, 1.0,  0.0, -1.0,  0.0,
        0.5, -0.5, -0.5,  1.0, 1.0,   0.0, -1.0,  0.0,
        0.5, -0.5,  0.5,  1.0, 0.0,   0.0, -1.0,  0.0,
        0.5, -0.5,  0.5,  1.0, 0.0,   0.0, -1.0,  0.0,
        -0.5, -0.5,  0.5,  0.0, 0.0,  0.0, -1.0,  0.0,
        -0.5, -0.5, -0.5,  0.0, 1.0,  0.0, -1.0,  0.0,

        -0.5,  0.5, -0.5,  0.0, 1.0,  0.0,  1.0,  0.0,
        0.5,  0.5, -0.5,  1.0, 1.0,   0.0,  1.0,  0.0,
        0.5,  0.5,  0.5,  1.0, 0.0,   0.0,  1.0,  0.0,
        0.5,  0.5,  0.5,  1.0, 0.0,   0.0,  1.0,  0.0,
        -0.5,  0.5,  0.5,  0.0, 0.0,  0.0,  1.0,  0.0,
        -0.5,  0.5, -0.5,  0.0, 1.0,   0.0,  1.0,  0.0
    ];

    let mut vbo: GLuint = 1;
    let mut vao: GLuint = 1;
    //let mut ebo: GLuint = 1;

    unsafe {
        gl::GenVertexArrays(1, ptr::from_mut(&mut vao));
    }

    unsafe {
        gl::BindVertexArray(vao);

        vbo = gen_buffers(1);
        //ebo = gen_buffers(1);

        //gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        //gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, size_of_val(&indices) as GLsizeiptr, indices.as_ptr() as *const c_void, gl::STATIC_DRAW);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

        gl::BufferData(gl::ARRAY_BUFFER, size_of_val(&vertex_data) as GLsizeiptr, vertex_data.as_ptr() as *const c_void, gl::STATIC_DRAW);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, mem::size_of::<[GLfloat; 8]>() as GLsizei, 0 as *mut c_void);
        gl::EnableVertexAttribArray(0);

        gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, mem::size_of::<[GLfloat; 8]>() as GLsizei, mem::size_of::<[GLfloat; 3]>() as *mut c_void);
        gl::EnableVertexAttribArray(1);

        gl::VertexAttribPointer(2, 3, gl::FLOAT, gl::FALSE, mem::size_of::<[GLfloat; 8]>() as GLsizei, mem::size_of::<[GLfloat; 5]>() as *mut c_void);
        gl::EnableVertexAttribArray(2);
    }

    TriangleArrayVAO::new(vao, 36)
}
pub fn rect(x: GLfloat, y: GLfloat, w: GLfloat, h: GLfloat) -> GLuint {
    let vertex_data : [GLfloat; 4 * 5] = [
        0.5,  0.5, 0.0,  1.0, 1.0,   // top right
        0.5, -0.5, 0.0,  1.0, 0.0,   // bottom right
        -0.5, -0.5, 0.0, 0.0, 0.0,   // bottom let
        -0.5,  0.5, 0.0, 0.0, 1.0    // top let
    ];

    let indices: [GLuint; 6] = [
        0, 1, 3,
        1, 2, 3
    ];


    let mut vbo: GLuint = 1;
    let mut vao: GLuint = 1;
    let mut ebo: GLuint = 1;

    unsafe {
        gl::GenVertexArrays(1, ptr::from_mut(&mut vao));
    }

    unsafe {
        gl::BindVertexArray(vao);

        vbo = gen_buffers(1);
        ebo = gen_buffers(1);

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, size_of_val(&indices) as GLsizeiptr, indices.as_ptr() as *const c_void, gl::STATIC_DRAW);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

        gl::BufferData(gl::ARRAY_BUFFER, size_of_val(&vertex_data) as GLsizeiptr, vertex_data.as_ptr() as *const c_void, gl::STATIC_DRAW);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, mem::size_of::<[GLfloat; 5]>() as GLsizei, 0 as *mut c_void);
        gl::EnableVertexAttribArray(0);

        gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, mem::size_of::<[GLfloat; 5]>() as GLsizei, mem::size_of::<[GLfloat; 3]>() as *mut c_void);
        gl::EnableVertexAttribArray(1);
    }

    return vao;
}

pub fn triangle(length: GLfloat, height: GLfloat, x: GLfloat, y: GLfloat) -> GLuint {
    let vertex_data: [GLfloat; 8 * 3] = [
        x, y, 0.0,                          1.0, 0.0, 0.0,  1.0, 1.0,
        x + length / 2.0, y + height, 0.0,  0.0, 1.0, 0.0,  1.0, 0.0,
        x + length, y, 0.0,                 0.0, 0.0, 1.0,  0.0, 1.0
    ];

    let mut vbo: GLuint = 1;
    let mut vao: GLuint = 1;

    unsafe {
        gl::GenVertexArrays(1, ptr::from_mut(&mut vao));
        
        gl::BindVertexArray(vao);

        vbo = gen_buffers(1);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(gl::ARRAY_BUFFER, size_of_val(&vertex_data) as GLsizeiptr, vertex_data.as_ptr() as *const c_void, gl::STATIC_DRAW);

        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, mem::size_of::<[GLfloat; 8]>() as GLsizei, 0 as *mut c_void);
        gl::EnableVertexAttribArray(0);

        gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, mem::size_of::<[GLfloat; 8]>() as GLsizei, mem::size_of::<[GLfloat; 3]>() as *mut c_void);
        gl::EnableVertexAttribArray(1);

        gl::VertexAttribPointer(2, 2, gl::FLOAT, gl::FALSE, mem::size_of::<[GLfloat; 8]>() as GLsizei, mem::size_of::<[GLfloat; 6]>() as *mut c_void);
        gl::EnableVertexAttribArray(2);
    }

    return vao;
}

pub fn render_square(vao: GLuint) {
    unsafe {
        gl::BindVertexArray(vao);
        //gl::BindBuffer(gl::VERTEX_ARRAY, vao);
        gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0 as *const c_void);
        gl::BindVertexArray(0);
    }
}