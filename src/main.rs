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
mod world;
mod openxr_handler;
mod renderer;

use std::default::Default;
use crate::app::App;
use std::ffi::{c_int, CString};
use std::ptr::null_mut;
use glutin::config::{Config, ConfigTemplate, ConfigTemplateBuilder};
use glutin::context::{ContextApi, ContextAttributesBuilder, NotCurrentContext};
use glutin::display::GetGlDisplay;
use crate::utils::gl_message_callback;
use glutin::prelude::*;
use glutin_winit::{DisplayBuilder, GlWindow};
use nalgebra_glm::e;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::raw_window_handle::{HasWindowHandle};
use winit::window::{Window, WindowAttributes};

//extern "C" fn framebuffer_size_callback(win: *mut GLFWwindow, width: c_int, height: c_int) {
//    unsafe {
//        gl::Viewport(0, 0, width, height);
//    }
//}

//fn process_input(window: &mut GLFWwindow) {
//    unsafe {
//        if glfwGetKey(window, glfw::ffi::KEY_ESCAPE) == glfw::ffi::PRESS {
//            glfwSetWindowShouldClose(window, 1);
//        }
//    }
//}

pub enum GlDisplayCreationState {
    /// The display was not build yet.
    Builder(DisplayBuilder),
    /// The display was already created for the application.
    Init,
}

const RADIUS: f64 = 20.0;

fn start() {

}

const DEFAULT_WINDOW_WIDTH: i32 = 1280;
const DEFAULT_WINDOW_HEIGHT: i32 = 800;

pub fn config_picker(configs: Box<dyn Iterator<Item = Config> + '_>) -> Config {
    configs
        .reduce(|accum, config| {
            let transparency_check = config.supports_transparency().unwrap_or(false)
                & !accum.supports_transparency().unwrap_or(false);

            if !transparency_check || config.num_samples() > accum.num_samples() {
                config
            } else {
                accum
            }
        })
        .unwrap()
}

pub fn window_attributes() -> WindowAttributes {
    Window::default_attributes()
        .with_title("Hola")
}

fn main() {
    println!("Hello, world!");
    
    
    let event_loop = EventLoop::new().unwrap();

    let template =
        ConfigTemplateBuilder::new().with_alpha_size(8).with_transparency(cfg!(cgl_backend));

    let display_builder = DisplayBuilder::new().with_window_attributes(Some(Window::default_attributes()
        .with_title("Hola")));

    let mut app = App::new(template, display_builder, &event_loop, 800, 600);
    
    event_loop.run_app(&mut app).unwrap();

    //let gl_context = create_gl_context(&window, &config);

    //let attrs = window
    //    .build_surface_attributes(Default::default()).unwrap();

    //let gl_surface = unsafe {
    //    config.display().create_window_surface(&config, &attrs).unwrap()
    //};

    //let gl_context = gl_context.make_current(&gl_surface).unwrap();

    //app.gl_context = Some(gl_context);
    //app.gl_display = GlDisplayCreationState::Builder(gl_surface.display());

    unsafe {

        //gl::load_with(|s| {
        //        let symbol = CString::new(s).unwrap();
        //        config.display().get_proc_address(symbol.as_c_str()).cast()
        //    }
        //);
        //
        //window.set_title("Hola");

        /*
        //glfwInit();
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

        gl::Enable(gl::DEBUG_OUTPUT);
        gl::DebugMessageCallback(Some(gl_message_callback), std::ptr::null());

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
         */
    }
}

fn create_gl_context(window: &Window, config: &Config) -> NotCurrentContext {
    let raw_window_handle = window.window_handle().ok().map(|f| f.as_raw());

    let context_attributes = ContextAttributesBuilder::new().build(raw_window_handle);

    let gl_display = config.display();

    unsafe {
        gl_display.create_context(&config, &context_attributes).unwrap()
    }

}

