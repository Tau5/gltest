use std::ffi::{c_char, CString, IntoStringError};
use std::ptr;
use std::ptr::{null, null_mut};
use gl::types::{GLchar, GLenum, GLint, GLsizei, GLuint};
use log::info;

pub fn gen_buffers(n: GLsizei) -> GLuint {
    let mut buf_id: GLuint = 0;

    unsafe {
        gl::GenBuffers(n, ptr::from_mut(&mut buf_id));
    }

    return buf_id;
}

pub fn compile_shader(shader: &str, type_: GLenum) -> GLuint {
    let shader_data = CString::new(shader).unwrap();
    let shader_pointer = shader_data.as_ptr();
    let pointer_array: [*const c_char; 1] = [shader_pointer];

    let shader_id: GLuint = unsafe {
        gl::CreateShader(type_)
    };

    let mut success: GLint = 1;

    unsafe {
        gl::ShaderSource(shader_id, 1, pointer_array.as_ptr(), null());
        gl::CompileShader(shader_id);
        gl::GetShaderiv(shader_id, gl::COMPILE_STATUS, ptr::from_mut(&mut success));
    }

    if success < 1 {
        let mut info_log: [u8; 512] = [0; 512];
        unsafe {
            gl::GetShaderInfoLog(shader_id, 512, null_mut(), ptr::from_mut(&mut info_log) as *mut GLchar);
        }
        //
        let info_log = String::from_utf8_lossy(&info_log).replace("\n", "\n\t");
            //.unwrap_or_else(|e| format!("(Couldn't get info log from shader {})", e));

        panic!("Couldn't compile shader (Error code {})\n\t{}", success, info_log);
    }

    shader_id
}