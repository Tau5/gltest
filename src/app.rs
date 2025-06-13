use crate::camera::Camera;
use crate::textures::{load_image, load_texture, MaterialStore};
use crate::object::Object;
use crate::prefabs;
use crate::player::Player;
use crate::shader::ShaderProgram;
use crate::vao::{TriangleArrayVAO, VAO};
use gl::types::{GLfloat, GLint};
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

    material_store: MaterialStore
}

impl App {
    pub fn new(window: *mut GLFWwindow, width: usize, height: usize) -> App {
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

            //gl::ClearColor(0.2, 0.2, 0.2, 1.0);
            gl::ClearColor(0.21, 0.64, 0.99, 1.0);
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

        let mut material_store = MaterialStore::new("textures/fallback.png".into());

        let light_point = prefabs::cube();

        material_store.load("container",
                            Some("textures/container2.png"),
                            Some("textures/container2_specular.png"),
                            Some("textures/container2_emission.png"),
                            1.0
        ).expect("Error loading container material");

        material_store.load("sand",
            Some("textures/sand.png"), Some("textures/sand_spec.png"), None, 1.0).unwrap();

        material_store.load("water",
            Some("textures/water.png"), None, None, 16.0).unwrap();

        material_store.load("fogata",
                            None, None, Some("textures/fogata.png"), 1.0).unwrap();

        //let mut lightpoint_pos = glm::vec3(6.0, 0.0, 10.0);
        let lightpoint_pos = glm::vec3(-5.0, -12.0, -4.0);
        let light_color: TVec3<GLfloat> = glm::vec3(0.94, 0.89, 0.81);
        //let light_color: TVec3<GLfloat> = glm::vec3(0.94, 0.49, 0.41);
        let diffuse_color: TVec3<GLfloat> = light_color.scale(0.5);
        let ambient_color: TVec3<GLfloat> = light_color.scale(0.2) as TVec3<f32>;
        
        //let lightpoint_pos = glm::vec3(-0.0, -0.7, -0.7);
        let testcube_pos = glm::vec3(8.0, 0.0, -2.0);
        let plane_pos = glm::vec3(8.0, -5.0, -2.0);

        let mut objects = Vec::new();

        let container = Object::new(
            testcube_pos,
            glm::vec3(0.0, 0.0, 0.0),
            glm::vec3(2.0, 1.0, 2.0),
            Box::from(prefabs::cube()),
            material_store.get("container"),
            true,
        );

        let plane = Object::new(
            plane_pos,
            glm::vec3(0.0, 0.0, 0.0),
            glm::vec3(10.0, 1.0, 10.0),
            Box::from(prefabs::cube()),
            material_store.get("box"),
            true,
        );

        let objects = Self::generate_objects(&mut material_store);

        let camera = Camera::new(3.14 / 2.0, 0.0, glm::vec3(8.0, 0.0, -07.0));

        let player = Player::new(camera);

        // Counts how many updates the mouse has made up to two (See last_x/y update at the bottom)
        // Qt.rgba(0.94, 0.89, 0.51, 1)

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
            light_point,
            material_store
        }
        //      while unsafe { glfwWindowShouldClose(window) == 0 }
    }

    fn generate_objects(mut material_store: &mut MaterialStore) -> Vec<Object> {
        let testcube_pos = glm::vec3(8.0, 0.0, -2.0);
        let plane_pos = glm::vec3(8.0, -5.0, -2.0);

        let mut objects = Vec::new();

        let container = Object::new(
            testcube_pos,
            glm::vec3(0.0, 0.0, 0.0),
            glm::vec3(2.0, 1.0, 2.0),
            Box::from(prefabs::cube()),
            material_store.get("container"),
            true,
        );

        let plane = Object::new(
            plane_pos,
            glm::vec3(0.0, 0.0, 0.0),
            glm::vec3(10.0, 1.0, 10.0),
            Box::from(prefabs::cube()),
            material_store.get("box"),
            true,
        );

        let beach_sand = Object::new(
            glm::vec3(0.0, -15.0, 0.0),
            glm::vec3(0.0, 0.0, 0.0),
            glm::vec3(40.0, 0.1, 20.0),
            Box::from(prefabs::cube()),
            material_store.get("sand"),
            true,
        );

        let beach_waterbed = Object::new(
            glm::vec3(0.0, -20.0, 0.0),
            glm::vec3(0.0, 0.0, 0.0),
            glm::vec3(70.0, 0.1, 70.0),
            Box::from(prefabs::cube()),
            material_store.get("sand"),
            true,
        );

        let ocean = Object::new(
            glm::vec3(0.0, -15.1, 0.0),
            glm::vec3(0.0, 0.0, 0.0),
            glm::vec3(100.0, 0.1, 100.0),
            Box::from(prefabs::cube()),
            material_store.get("water"),
            true,
        );

        let fire = Object::new(
            glm::vec3(-5.0, -13.0, -5.0),
            glm::vec3(0.0, 0.0, 0.0),
            glm::vec3(2.0, 2.0, 0.0),
            Box::from(prefabs::cube()),
            material_store.get("fogata"),
            true
        );

        objects.push(container);
        objects.push(plane);
        objects.push(beach_sand);
        objects.push(beach_waterbed);
        objects.push(ocean);
        objects.push(fire);
        objects
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
                    if self.paused {
                        self.mouse_change_counter = 0;
                    }
                }


                if (glfwGetKey(window, glfw::ffi::KEY_Z)) == glfw::ffi::PRESS {
                    self.lightpoint_pos.x -= 0.01;
                }
                if (glfwGetKey(window, glfw::ffi::KEY_X)) == glfw::ffi::PRESS {
                    self.lightpoint_pos.x += 0.01;
                }
                if (glfwGetKey(window, glfw::ffi::KEY_C)) == glfw::ffi::PRESS {
                    self.lightpoint_pos.y -= 0.01;
                }
                if (glfwGetKey(window, glfw::ffi::KEY_V)) == glfw::ffi::PRESS {
                    self.lightpoint_pos.y += 0.01;
                }
                if (glfwGetKey(window, glfw::ffi::KEY_B)) == glfw::ffi::PRESS {
                    self.lightpoint_pos.z -= 0.01;
                }
                if (glfwGetKey(window, glfw::ffi::KEY_N)) == glfw::ffi::PRESS {
                    self.lightpoint_pos.z += 0.01;
                }
            }

            self.player.translate(move_x, move_z, &self.objects);
            self.player.apply_gravity(-0.27, &self.objects);
            self.player.rotate_camera(
                (mouse_x * self.mouse_sensitivity) as f32,
                -(mouse_y * self.mouse_sensitivity) as f32,
            );
            let view = self.player.get_view();

            //self.lightpoint_pos.x = GLfloat::sin(time_value as f32) * 2.0 + 8.0;

            unsafe {
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            }

            // mat3(transpose(inverse(model)))

            self.load_lighting_shader(&view);

            for obj in &self.objects {
                obj.render(&self.lighting_shader);
            }

            let lightpoint_model = glm::translate(&model, &self.lightpoint_pos);
            let lightpoint_model = glm::scale(&lightpoint_model, &glm::vec3(0.25, 0.25, 0.25));
            let testcube_normal = glm::mat4_to_mat3(&glm::inverse_transpose(lightpoint_model));

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

    fn load_lighting_shader(&mut self, view: &TMat4<GLfloat>) {
        self.lighting_shader.load();
        self.lighting_shader.setMat4(c"view", &view);
        self.lighting_shader.setMat4(c"projection", &self.proj);

        //let mut light_vector = glm::vec3_to_vec4(&self.lightpoint_pos);
        let mut light_vector = glm::vec3_to_vec4(&self.player.get_position());
        let spotlight_direction = &self.player.get_front();
        light_vector.w = 1.0;

        self.lighting_shader.setVec4(c"light.vector", &light_vector);
        self.lighting_shader.setVec3(c"light.ambient", &self.ambient_color);
        self.lighting_shader.setVec3(c"light.diffuse", &self.diffuse_color); // darken diuse light a bit
        self.lighting_shader.setVec3(c"light.specular", &self.diffuse_color);
        self.lighting_shader.setFloat(c"light.constant", 1.0);
        self.lighting_shader.setFloat(c"light.linear", 0.045);
        self.lighting_shader.setFloat(c"light.quadratic", 0.0075);
        self.lighting_shader.setVec3(c"light.spotlightDirection", &spotlight_direction);
        self.lighting_shader.setFloat(c"light.spAngleInner", f32::cos(0.21));
        self.lighting_shader.setFloat(c"light.spAngleOuter", f32::cos(0.27));

        self.lighting_shader.setVec3(c"viewPos", &self.player.get_position());
    }
}
