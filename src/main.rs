mod utils;
mod prefabs;
mod shader;
mod textures;
mod camera;
mod object;
mod vao;
mod player;
mod aabb;
mod collidable;
mod app;
mod lighting;
mod input;
mod mesh;
mod model;
mod geometry;
mod cube;
mod util;

use crate::app::App;
use glfw;
use glfw::ffi::{glfwCreateWindow, glfwGetProcAddress, glfwGetWindowSize, glfwInit, glfwMakeContextCurrent, glfwSetFramebufferSizeCallback, glfwTerminate, glfwWindowHint, glfwWindowShouldClose, GLFWwindow, OPENGL_CORE_PROFILE};
use std::ffi::{c_int, CString};
use std::ptr::null_mut;

extern "C" fn framebuffer_size_callback(win: *mut GLFWwindow, width: c_int, height: c_int) {
    unsafe {
        gl::Viewport(0, 0, width, height);
    }
}

//fn process_input(window: &mut GLFWwindow) {
//    unsafe {
//        if glfwGetKey(window, glfw::ffi::KEY_ESCAPE) == glfw::ffi::PRESS {
//            glfwSetWindowShouldClose(window, 1);
//        }
//    }
//}

const RADIUS: f64 = 20.0;

fn start() {

}

const DEFAULT_WINDOW_WIDTH: i32 = 1280;
const DEFAULT_WINDOW_HEIGHT: i32 = 800;

fn main() {
    println!("Hello, world!");

    unsafe {
        glfwInit();
        glfwWindowHint(glfw::ffi::CONTEXT_VERSION_MAJOR, 4);
        glfwWindowHint(glfw::ffi::CONTEXT_VERSION_MINOR, 6);
        glfwWindowHint(glfw::ffi::OPENGL_PROFILE, OPENGL_CORE_PROFILE);
        glfwWindowHint(glfw::ffi::DOUBLEBUFFER, glfw::ffi::TRUE);

        let title = CString::new("Hola").unwrap();
        let window: *mut GLFWwindow =
            glfwCreateWindow(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT, title.as_ptr(), null_mut(), null_mut());


        if window.is_null() {
            glfwTerminate();
            panic!("No window!");
        }

        glfwMakeContextCurrent(window);

        //glfwSetWindowSizeCallback(window, Some(resize));
        glfwSetFramebufferSizeCallback(window, Some(framebuffer_size_callback));

        gl::load_with(|s| {
            //glfwGetProcAddress(s.as_ptr() as *const std::os::raw::c_char) as _
            let symbol = CString::new(s).unwrap();
            glfwGetProcAddress(symbol.as_c_str().as_ptr()).cast()
        });

        //gl::load_with(|s| window.get_proc_address(s) as *const _);

        //let gl33ctx = gl33::GlFns::load_from(&|s| {
        //    glfwGetProcAddress(s as *const std::os::raw::c_char) as _
        //}).unwrap();

        //gl::Viewport(0, 0, 800, 600);

        if !gl::Viewport::is_loaded() {
            panic!("aaaa");
        }


        let width: *mut c_int = null_mut();
        let height: *mut c_int = null_mut();

        glfwGetWindowSize(window, width, height);

        //if (!width.is_null() && !height.is_null()) {
        //    resize(window, *width, *height);
        //} else {
        //    resize(window, DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT);
        //}

        gl::Viewport(0, 0, DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT);

        //glfwSwapInterval(0);

        let mut app = App::new(window, DEFAULT_WINDOW_WIDTH as usize, DEFAULT_WINDOW_HEIGHT as usize);

        while glfwWindowShouldClose(window) == 0 {
           app.game_loop();
        }

        glfwTerminate()
    }
}

