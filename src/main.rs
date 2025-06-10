mod utils;
mod objects;
mod shader;
mod image;
mod camera;
mod object;
mod vao;
mod player;
mod aabb;
mod collidable;
mod app;

use glfw;
use glfw::ffi::{GLFWwindow, OPENGL_COMPAT_PROFILE, glfwCreateWindow, glfwGetKey, glfwGetProcAddress, glfwGetWindowSize, glfwInit, glfwMakeContextCurrent, glfwPollEvents, glfwSetFramebufferSizeCallback, glfwSetWindowShouldClose, glfwSwapBuffers, glfwTerminate, glfwWindowHint, glfwWindowShouldClose, glfwGetTime, glfwGetCursorPos, glfwSwapInterval, OPENGL_CORE_PROFILE, glfwGetWindowFrameSize};
use std::ffi::{CString, c_int, c_void, c_double};
use std::ops::Mul;
use std::ptr;
use std::ptr::null_mut;
use std::time::SystemTime;
use fastrand::f64;
use gl::types::GLfloat;
use nalgebra_glm as glm;
use nalgebra_glm::{proj, TVec3};
use crate::camera::Camera;
use crate::collidable::Collidable;
use crate::image::{load_image, load_texture};
use crate::object::Object;
use crate::player::Player;
use crate::shader::ShaderProgram;
use crate::vao::VAO;

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

fn start(window: *mut GLFWwindow, width: usize, height: usize) {
    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    }

    let lighting_shader = ShaderProgram::new(
        include_str!("vertex_shader.glsl"),
        include_str!("lighting.glsl")
    );

    let lightpoint_shader = ShaderProgram::new(
        include_str!("vertex_shader.glsl"),
        include_str!("lightpoint.glsl")
    );

    let aaab_shader = ShaderProgram::new(
        include_str!("vertex_shader.glsl"),
        include_str!("aaab_renderer.glsl")
    );

    let light_point = objects::cube();

    let image = load_image("textures/container2.png");
    let container_specular = load_texture(load_image("textures/container2_specular.png"));
    let container_emission = load_texture(load_image("textures/container2_emission.png"));
    let texture = load_texture(image);
    let texture2 = load_texture(load_image("textures/awesomeface.png"));

    let mut lightpoint_pos = glm::vec3(6.0, 0.0, -4.0);
    let testcube_pos = glm::vec3(8.0, 0.0, -2.0);
    let plane_pos = glm::vec3(8.0, -5.0, -2.0);

    let mut container = Object::new(
        testcube_pos,
        glm::vec3(0.0, 0.0, 0.0),
        glm::vec3(2.0, 1.0, 2.0),
        Box::from(objects::cube()),
        texture,
        Some(container_specular),
        Some(container_emission),
        true
    );

    let plane = Object::new(
        plane_pos,
        glm::vec3(0.0, 0.0, 0.0),
        glm::vec3(10.0, 1.0, 10.0),
        Box::from(objects::cube()),
        texture,
        Some(container_specular),
        None,
        true
    );

    let camera_speed = 0.2;
    let mouse_sensitivity = 0.01;

    let mut last_x = 0.0;
    let mut last_y = 0.0;

    let mut camera = Camera::new(
        3.14/2.0, 0.0, glm::vec3(8.0, 0.0, -07.0)
    );

    let mut player = Player::new(camera);

    // Counts how many updates the mouse has made up to two (See last_x/y update at the bottom)
    let mut mouse_change_counter = 0;

    let light_color: TVec3<GLfloat> = glm::vec3(
        1.0, 1.0, 1.0,
    );
    let diffuse_color: TVec3<GLfloat> = light_color.scale(0.5);
    let ambient_color: TVec3<GLfloat> = light_color.scale(0.2) as TVec3<f32>;

    let proj = glm::perspective(width as f32 / height as f32, std::f32::consts::TAU / 8.0, 0.1, 100.0);

    let mut objects: Vec<Object> = vec![
        container,
        plane
    ];

    let mut paused = false;
    let mut selected_obj = 0;

    while unsafe { glfwWindowShouldClose(window) == 0 } {
        if let Some(window) = unsafe { window.as_mut() } {
            //process_input(window);
            let time_value = unsafe { glfwGetTime() };
            

            let mut model = glm::identity::<f32, 4>();

            let mut current_x: c_double = 0.0;
            let mut current_y: c_double = 0.0;

            if (!paused) {
                unsafe {
                    glfwGetCursorPos(window, ptr::from_mut(&mut current_x), ptr::from_mut(&mut current_y));
                }
            }

            let mouse_x =
                if (mouse_change_counter > 2) { current_x - last_x } else { 0.0 };
            let mouse_y =
                if (mouse_change_counter > 2) { current_y - last_y } else { 0.0 };

            let mut move_x = 0.0;
            let mut move_z = 0.0;


            unsafe {
                if (!paused) {
                    if glfwGetKey(window, glfw::ffi::KEY_D) == glfw::ffi::PRESS {
                        move_x += camera_speed
                    }
                    if glfwGetKey(window, glfw::ffi::KEY_A) == glfw::ffi::PRESS {
                        move_x -= camera_speed;
                    }
                    if glfwGetKey(window, glfw::ffi::KEY_W) == glfw::ffi::PRESS {
                        move_z += camera_speed;
                    }
                    if glfwGetKey(window, glfw::ffi::KEY_S) == glfw::ffi::PRESS {
                        move_z -= camera_speed;
                    }
                    if glfwGetKey(window, glfw::ffi::KEY_LEFT) == glfw::ffi::PRESS {
                        if selected_obj > 0 {
                            selected_obj -= 1;
                        }
                    }
                    if glfwGetKey(window, glfw::ffi::KEY_RIGHT) == glfw::ffi::PRESS {
                        if selected_obj < objects.len() - 1 {
                            selected_obj += 1;
                        }
                    }

                    if let Some(obj) = objects.get_mut(selected_obj) {
                        if glfwGetKey(window, glfw::ffi::KEY_I) == glfw::ffi::PRESS {
                            obj.scale(0.1, 0.0, 0.0);
                        }
                        if glfwGetKey(window, glfw::ffi::KEY_L) == glfw::ffi::PRESS {
                            obj.scale(0.0, 0.0, 0.1);
                        }
                    }
                }

                if (glfwGetKey(window, glfw::ffi::KEY_ESCAPE)) == glfw::ffi::PRESS {
                    paused = !paused;
                    if (paused) {
                        mouse_change_counter = 0;
                    }
                }

            }

            player.translate(move_x, move_z, &objects);
            player.apply_gravity(-0.01, &objects);
            player.rotate_camera((mouse_x * mouse_sensitivity) as f32, -(mouse_y * mouse_sensitivity) as f32);
            let view = player.get_view();

            lightpoint_pos.x = GLfloat::sin(time_value as f32) * 2.0 + 8.0;

            unsafe {
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            }

            let lightpoint_model = glm::translate(&model, &lightpoint_pos);
            let lightpoint_model = glm::scale(&lightpoint_model, &glm::vec3(0.25, 0.25, 0.25));

            let testcube_normal = glm::mat4_to_mat3(&glm::inverse_transpose(lightpoint_model));
            // mat3(transpose(inverse(model)))

            lighting_shader.load();
            lighting_shader.setMat4(c"view", &view);
            lighting_shader.setMat4(c"projection", &proj);

            lighting_shader.setVec3(c"light.position", &lightpoint_pos);
            lighting_shader.setVec3(c"light.ambient",  &ambient_color);
            lighting_shader.setVec3(c"light.diffuse",  &diffuse_color); // darken diuse light a bit
            lighting_shader.setVec3(c"light.specular", &glm::vec3(1.0, 1.0, 1.0));
            lighting_shader.setVec3(c"viewPos", &player.get_position());

            for obj in &objects {
                obj.render(&lighting_shader);
            }

            lightpoint_shader.load();
            lightpoint_shader.setMat4(c"model", &lightpoint_model);
            lightpoint_shader.setMat4(c"view", &view);
            lightpoint_shader.setMat4(c"projection", &proj);
            lightpoint_shader.setMat3(c"normalMatrix", &testcube_normal);
            lightpoint_shader.setVec3(c"color", &light_color);

            light_point.render();

            aaab_shader.load();
            aaab_shader.setMat4(c"view", &view);
            aaab_shader.setMat4(c"projection", &proj);

            if let Some(obj) = objects.get(selected_obj) {
                obj.debug_render_aabb(&aaab_shader);
            }

            unsafe {
                glfwSwapBuffers(window);
                glfwPollEvents();
            }

            if (mouse_change_counter < 3) && ((last_x != current_x) || (last_y != current_y)) {
                mouse_change_counter += 1;
            }

            last_x = current_x;
            last_y = current_y;
        }
    }
}

const DEFAULT_WINDOW_WIDTH: i32 = 800;
const DEFAULT_WINDOW_HEIGHT: i32 = 600;

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

        glfw::ffi::glfwSetInputMode(window, glfw::ffi::CURSOR, glfw::ffi::CURSOR_DISABLED);

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
        gl::ClearColor(0.2, 0.2, 0.2, 1.0);

        start(window, DEFAULT_WINDOW_WIDTH as usize, DEFAULT_WINDOW_HEIGHT as usize);

        glfwTerminate()
    }
}

