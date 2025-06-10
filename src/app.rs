use crate::camera::Camera;
use crate::image::{load_image, load_texture};
use crate::object::Object;
use crate::objects;
use crate::player::Player;
use crate::shader::ShaderProgram;
use crate::vao::{TriangleArrayVAO, VAO};
use gl::types::GLfloat;
use glfw::ffi::{
    glfwGetCursorPos, glfwGetKey, glfwGetTime, glfwPollEvents, glfwSwapBuffers, GLFWwindow
    ,
};
use nalgebra_glm as glm;
use nalgebra_glm::{TMat4, TVec3};
use std::ffi::c_double;
use std::ptr;

pub struct App {
    objects: Vec<Object>,
    lighting_shader: ShaderProgram,
    lightpoint_shader: ShaderProgram,
    aabb_shader: ShaderProgram,

    camera_speed: GLfloat,
    mouse_sensitivity: c_double,
    last_y: c_double,
    last_x: c_double,
    window: *mut GLFWwindow,

    paused: bool,
    selected_obj: usize,
    proj: TMat4<f32>,
    ambient_color: TVec3<GLfloat>,
    diffuse_color: TVec3<GLfloat>,
    light_color: TVec3<GLfloat>,
    mouse_change_counter: i32,
    player: Player,
    lightpoint_pos: TVec3<GLfloat>,
    light_point: TriangleArrayVAO,
}

impl App {
    pub fn new(window: *mut GLFWwindow, width: usize, height: usize) -> App {
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }

        let lighting_shader = ShaderProgram::new(
            include_str!("vertex_shader.glsl"),
            include_str!("lighting.glsl"),
        );

        let lightpoint_shader = ShaderProgram::new(
            include_str!("vertex_shader.glsl"),
            include_str!("lightpoint.glsl"),
        );

        let aabb_shader = ShaderProgram::new(
            include_str!("vertex_shader.glsl"),
            include_str!("aaab_renderer.glsl"),
        );

        let light_point = objects::cube();

        let container_specular = load_texture(load_image("textures/container2_specular.png"));
        let container_emission = load_texture(load_image("textures/container2_emission.png"));
        let texture = load_texture(load_image("textures/container2.png"));

        let mut lightpoint_pos = glm::vec3(6.0, 0.0, -4.0);
        let testcube_pos = glm::vec3(8.0, 0.0, -2.0);
        let plane_pos = glm::vec3(8.0, -5.0, -2.0);

        let mut objects = Vec::new();

        let mut container = Object::new(
            testcube_pos,
            glm::vec3(0.0, 0.0, 0.0),
            glm::vec3(2.0, 1.0, 2.0),
            Box::from(objects::cube()),
            texture,
            Some(container_specular),
            Some(container_emission),
            true,
        );

        let plane = Object::new(
            plane_pos,
            glm::vec3(0.0, 0.0, 0.0),
            glm::vec3(10.0, 1.0, 10.0),
            Box::from(objects::cube()),
            texture,
            Some(container_specular),
            None,
            true,
        );

        objects.push(container);
        objects.push(plane);

        let camera = Camera::new(3.14 / 2.0, 0.0, glm::vec3(8.0, 0.0, -07.0));

        let player = Player::new(camera);

        // Counts how many updates the mouse has made up to two (See last_x/y update at the bottom)
        let light_color: TVec3<GLfloat> = glm::vec3(1.0, 1.0, 1.0);
        let diffuse_color: TVec3<GLfloat> = light_color.scale(0.5);
        let ambient_color: TVec3<GLfloat> = light_color.scale(0.2) as TVec3<f32>;

        let proj = glm::perspective(
            width as f32 / height as f32,
            std::f32::consts::TAU / 8.0,
            0.1,
            100.0,
        );

        Self {
            window,
            last_x: 0.0,
            last_y: 0.0,
            camera_speed: 0.2,
            mouse_sensitivity: 0.01,
            mouse_change_counter: 0,
            paused: false,
            selected_obj: 0,
            player,
            light_color,
            diffuse_color,
            ambient_color,
            proj,
            objects,
            lighting_shader,
            lightpoint_shader,
            aabb_shader,
            lightpoint_pos,
            light_point
        }
        //      while unsafe { glfwWindowShouldClose(window) == 0 }
    }

    pub fn game_loop(&mut self) {
        if let Some(window) = unsafe { self.window.as_mut() } {
            //process_input(window);
            let time_value = unsafe { glfwGetTime() };
            let model = glm::identity::<f32, 4>();
            let mut current_x: c_double = 0.0;
            let mut current_y: c_double = 0.0;

            if !self.paused {
                unsafe {
                    glfwGetCursorPos(
                        window,
                        ptr::from_mut(&mut current_x),
                        ptr::from_mut(&mut current_y),
                    );
                }
            }

            let mouse_x = if self.mouse_change_counter > 2 {
                current_x - self.last_x
            } else {
                0.0
            };

            let mouse_y = if self.mouse_change_counter > 2 {
                current_y - self.last_y
            } else {
                0.0
            };

            let mut move_x: f32 = 0.0;
            let mut move_z: f32 = 0.0;

            unsafe {
                if (!self.paused) {
                    if glfwGetKey(window, glfw::ffi::KEY_D) == glfw::ffi::PRESS {
                        move_x += self.camera_speed
                    }
                    if glfwGetKey(window, glfw::ffi::KEY_A) == glfw::ffi::PRESS {
                        move_x -= self.camera_speed;
                    }
                    if glfwGetKey(window, glfw::ffi::KEY_W) == glfw::ffi::PRESS {
                        move_z += self.camera_speed;
                    }
                    if glfwGetKey(window, glfw::ffi::KEY_S) == glfw::ffi::PRESS {
                        move_z -= self.camera_speed;
                    }
                    if glfwGetKey(window, glfw::ffi::KEY_LEFT) == glfw::ffi::PRESS {
                        if self.selected_obj > 0 {
                            self.selected_obj -= 1;
                        }
                    }
                    if glfwGetKey(window, glfw::ffi::KEY_RIGHT) == glfw::ffi::PRESS {
                        if self.selected_obj < self.objects.len() - 1 {
                            self.selected_obj += 1;
                        }
                    }

                    if let Some(obj) = self.objects.get_mut(self.selected_obj) {
                        if glfwGetKey(window, glfw::ffi::KEY_I) == glfw::ffi::PRESS {
                            obj.scale(0.1, 0.0, 0.0);
                        }
                        if glfwGetKey(window, glfw::ffi::KEY_L) == glfw::ffi::PRESS {
                            obj.scale(0.0, 0.0, 0.1);
                        }
                    }
                }

                if (glfwGetKey(window, glfw::ffi::KEY_ESCAPE)) == glfw::ffi::PRESS {
                    self.paused = !self.paused;
                    if (self.paused) {
                        self.mouse_change_counter = 0;
                    }
                }
            }

            self.player.translate(move_x, move_z, &self.objects);
            self.player.apply_gravity(-0.01, &self.objects);
            self.player.rotate_camera(
                (mouse_x * self.mouse_sensitivity) as f32,
                -(mouse_y * self.mouse_sensitivity) as f32,
            );
            let view = self.player.get_view();

            self.lightpoint_pos.x = GLfloat::sin(time_value as f32) * 2.0 + 8.0;

            unsafe {
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            }

            let lightpoint_model = glm::translate(&model, &self.lightpoint_pos);
            let lightpoint_model = glm::scale(&lightpoint_model, &glm::vec3(0.25, 0.25, 0.25));

            let testcube_normal = glm::mat4_to_mat3(&glm::inverse_transpose(lightpoint_model));
            // mat3(transpose(inverse(model)))

            self.lighting_shader.load();
            self.lighting_shader.setMat4(c"view", &view);
            self.lighting_shader.setMat4(c"projection", &self.proj);

            self.lighting_shader.setVec3(c"light.position", &self.lightpoint_pos);
            self.lighting_shader.setVec3(c"light.ambient", &self.ambient_color);
            self.lighting_shader.setVec3(c"light.diffuse", &self.diffuse_color); // darken diuse light a bit
            self.lighting_shader.setVec3(c"light.specular", &glm::vec3(1.0, 1.0, 1.0));
            self.lighting_shader.setVec3(c"viewPos", &self.player.get_position());

            for obj in &self.objects {
                obj.render(&self.lighting_shader);
            }

            self.lightpoint_shader.load();
            self.lightpoint_shader.setMat4(c"model", &lightpoint_model);
            self.lightpoint_shader.setMat4(c"view", &view);
            self.lightpoint_shader.setMat4(c"projection", &self.proj);
            self.lightpoint_shader.setMat3(c"normalMatrix", &testcube_normal);
            self.lightpoint_shader.setVec3(c"color", &self.light_color);

            self.light_point.render();

            self.aabb_shader.load();
            self.aabb_shader.setMat4(c"view", &view);
            self.aabb_shader.setMat4(c"projection", &self.proj);

            if let Some(obj) = self.objects.get(self.selected_obj) {
                obj.debug_render_aabb(&self.aabb_shader);
            }

            unsafe {
                glfwSwapBuffers(window);
                glfwPollEvents();
            }

            if (self.mouse_change_counter < 3) && ((self.last_x != current_x) || (self.last_y != current_y)) {
                self.mouse_change_counter += 1;
            }

            self.last_x = current_x;
            self.last_y = current_y;
        }
    }
}
