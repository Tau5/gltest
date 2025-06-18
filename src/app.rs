use crate::camera::Camera;
use crate::lighting::{DirLight, LightManager, PointLight, SpotLight};
use crate::object::Object;
use crate::player::Player;
use crate::prefabs;
use crate::shader::ShaderProgram;
use crate::textures::{MaterialStore, load_image, load_texture, TextureSource};
use crate::vao::{TriangleArrayVAO, VAO};
use gl::types::{GLfloat, GLint};
use glfw::ffi::{
    GLFWwindow, glfwGetCursorPos, glfwGetKey, glfwGetTime, glfwPollEvents, glfwSwapBuffers,
};
use nalgebra_glm as glm;
use nalgebra_glm::{TMat4, TVec3};
use std::ffi::c_double;
use std::ptr;
use glfw::Key;
use crate::cube::Cube;
use crate::input::{InputManager, KeyStatus};
use crate::model::Model;

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
    mouse_change_counter: i32,
    player: Player,

    material_store: MaterialStore,
    input_manager: InputManager,
    light_manager: LightManager,
    pub lamp_vao: TriangleArrayVAO,
}

impl App {
    pub fn new(window: *mut GLFWwindow, width: usize, height: usize) -> App {
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

            //gl::ClearColor(0.2, 0.2, 0.2, 1.0);
            //gl::ClearColor(0.21, 0.64, 0.99, 1.0);
            // Qt.rgba(0.21, 0.38, 0.52, 1)
            gl::ClearColor(0.08, 0.25, 0.4, 1.0);
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

        let camera = Camera::new(3.14 / 2.0, 0.0, glm::vec3(8.0, 0.0, -07.0));
        let player = Player::new(camera);

        let mut material_store = MaterialStore::new("textures/fallback.png".into());
        let lightpoint_pos = glm::vec3(-5.0, -12.0, -4.0);
        let light_color: TVec3<GLfloat> = glm::vec3(0.94, 0.89, 0.81);
        //let light_color: TVec3<GLfloat> = glm::vec3(0.94, 0.49, 0.41);
        let diffuse_color: TVec3<GLfloat> = light_color.scale(0.7);
        let ambient_color: TVec3<GLfloat> = light_color.scale(0.2) as TVec3<f32>;

        let spotlight = SpotLight::new(
            player.get_position(),
            player.get_front(),
            0.21,
            0.27,
            glm::vec3(0.2, 0.2, 0.2),
            glm::vec3(0.5, 0.5, 0.5),
            glm::vec3(0.5, 0.5, 0.5),
        );

        let mut light_manager = LightManager::new(
            4,
            DirLight::new(
                lightpoint_pos,
                ambient_color,
                diffuse_color,
                diffuse_color
            ),
            Some(spotlight),
        );
        light_manager.add_lightpoint(PointLight::new(
            glm::vec3(-5.0, -13.0, -5.0),
            glm::vec3(1.0, 0.2, 0.2),
            glm::vec3(1.0, 0.6, 0.6),
            glm::vec3(1.0, 0.6, 0.6),
            1.0, 0.07, 0.017
        )).unwrap();

        let lamp_vao = prefabs::cube();

        material_store
            .load(
                "container",
                Some(TextureSource::ImagePath("textures/container2.png".into())),
                Some(TextureSource::ImagePath("textures/container2_specular.png".into())),
                Some(TextureSource::ImagePath("textures/container2_emission.png".into())),
                1.0,
            )
            .expect("Error loading container material");

        material_store
            .load(
                "sand",
                Some(TextureSource::ImagePath("textures/sand.png".into())),
                Some(TextureSource::ImagePath("textures/sand_spec.png".into())),
                None,
                4.0,
            )
            .unwrap();

        material_store
            .load("water", Some(TextureSource::ImagePath("textures/water.png".into())), None, None, 16.0)
            .unwrap();

        material_store
            .load("fogata", Some(TextureSource::ImagePath("textures/fogata.png".into())), None, None, 1.0)
            .unwrap();

        //let mut lightpoint_pos = glm::vec3(6.0, 0.0, 10.0);
        //let lightpoint_pos = glm::vec3(-0.0, -0.7, -0.7);

        let objects = Self::generate_objects(&mut material_store);

        let proj = glm::perspective(
            width as f32 / height as f32,
            std::f32::consts::TAU / 8.0,
            0.1,
            100.0,
        );

        let mut input_manager = InputManager::new();

        input_manager.add_key(Key::W);
        input_manager.add_key(Key::A);
        input_manager.add_key(Key::S);
        input_manager.add_key(Key::D);
        input_manager.add_key(Key::H);
        input_manager.add_key(Key::J);
        input_manager.add_key(Key::K);
        input_manager.add_key(Key::L);
        input_manager.add_key(Key::F);
        input_manager.add_key(Key::Z);
        input_manager.add_key(Key::X);
        input_manager.add_key(Key::C);
        input_manager.add_key(Key::V);
        input_manager.add_key(Key::B);
        input_manager.add_key(Key::N);
        input_manager.add_key(Key::M);
        input_manager.add_key(Key::Left);
        input_manager.add_key(Key::Right);
        input_manager.add_key(Key::Escape);


        unsafe { glfw::ffi::glfwSetInputMode(window, glfw::ffi::CURSOR, glfw::ffi::CURSOR_DISABLED); }
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
            proj,
            objects,
            lighting_shader,
            lightpoint_shader,
            aabb_shader,
            material_store,
            light_manager,
            lamp_vao,
            input_manager,
        }
        //      while unsafe { glfwWindowShouldClose(window) == 0 }
    }

    fn generate_objects(mut material_store: &mut MaterialStore) -> Vec<Object> {
        let testcube_pos = glm::vec3(8.0, 0.0, -2.0);
        let plane_pos = glm::vec3(8.0, -5.0, -2.0);
        let mut model_test = Model::new("models/example.glb".into(), &mut material_store);

        let mut objects = Vec::new();

        let container = Object::new(
            testcube_pos,
            glm::vec3(0.0, 0.0, 0.0),
            glm::vec3(2.0, 1.0, 2.0),
            Box::from(Cube::new(
                material_store.get("container"),
            )),
            true,
        );

        let plane = Object::new(
            plane_pos,
            glm::vec3(0.0, 0.0, 0.0),
            glm::vec3(10.0, 1.0, 10.0),
            Box::from(Cube::new( material_store.get("box"),
            )),
            true,
        );

        let beach_sand = Object::new(
            glm::vec3(0.0, -15.0, 0.0),
            glm::vec3(0.0, 0.0, 0.0),
            glm::vec3(40.0, 0.1, 20.0),
            Box::from(Cube::new( material_store.get("sand"),
            )),
            true,
        );

        let beach_waterbed = Object::new(
            glm::vec3(0.0, -20.0, 0.0),
            glm::vec3(0.0, 0.0, 0.0),
            glm::vec3(70.0, 0.1, 70.0),
            Box::from(Cube::new( material_store.get("sand"),
            )),
            true,
        );

        let ocean = Object::new(
            glm::vec3(0.0, -15.1, 0.0),
            glm::vec3(0.0, 0.0, 0.0),
            glm::vec3(100.0, 0.1, 100.0),
            Box::from(Cube::new( material_store.get("water"),
            )),
            true,
        );

        let fire = Object::from_model(
            glm::vec3(-5.0, -14.0, -5.0),
            glm::vec3(0.0, 0.0, 0.0),
            glm::vec3(1.0, 1.0, 1.0),
            true,
            model_test,
            material_store
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

            self.input_manager.poll(window);

            unsafe {
                if !self.paused {
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
                    if self.input_manager.get_status(Key::Left) == KeyStatus::JustPressed {
                        if self.selected_obj > 0 {
                            self.selected_obj -= 1;
                        }
                    }
                    if self.input_manager.get_status(Key::Right) == KeyStatus::JustPressed {
                        if self.selected_obj < self.objects.len() - 1 {
                            self.selected_obj += 1;
                        }
                    }

                    if self.input_manager.get_status(Key::F) == KeyStatus::JustPressed {
                        self.light_manager.toggle_spotlight();
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

                if self.input_manager.get_status(Key::Escape) == KeyStatus::JustPressed {
                    self.paused = !self.paused;
                    if self.paused {
                        glfw::ffi::glfwSetInputMode(window, glfw::ffi::CURSOR, glfw::ffi::CURSOR_NORMAL);
                        self.mouse_change_counter = 0;
                    } else {
                        glfw::ffi::glfwSetInputMode(window, glfw::ffi::CURSOR, glfw::ffi::CURSOR_DISABLED);
                    }
                }

                if self.input_manager.get_status(Key::Z) == KeyStatus::JustPressed {
                    self.light_manager.directional_light.direction = self.player.get_front();
                }
                if let Some(light) = self.light_manager.get_mut_pointlight(0) {

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
                obj.render(&self.lighting_shader, &self.material_store);
            }

            self.lightpoint_shader.load();
            self.lightpoint_shader.setMat4(c"view", &view);
            self.lightpoint_shader.setMat4(c"projection", &self.proj);

            for light in self.light_manager.iter() {
                let lightpoint_model = glm::translate(&model, &light.position);
                let lightpoint_model = glm::scale(&lightpoint_model, &glm::vec3(0.25, 0.25, 0.25));
                let testcube_normal = glm::mat4_to_mat3(&glm::inverse_transpose(lightpoint_model));

                self.lightpoint_shader.setMat4(c"model", &lightpoint_model);
                self.lightpoint_shader
                    .setMat3(c"normalMatrix", &testcube_normal);
                self.lightpoint_shader.setVec3(c"color", &light.diffuse);

                self.lamp_vao.render();
            }

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

            if (self.mouse_change_counter < 3)
                && ((self.last_x != current_x) || (self.last_y != current_y))
            {
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
        self.lighting_shader
            .setVec3(c"viewPos", &self.player.get_position());

        if let Some(spot_light) = &mut self.light_manager.spot_light {
            spot_light.position = self.player.get_position();
            spot_light.direction = self.player.get_front();
        }

        self.light_manager.load_lights(&self.lighting_shader);

        ////let mut light_vector = glm::vec3_to_vec4(&self.lightpoint_pos);
        //let mut light_vector = glm::vec3_to_vec4(&self.player.get_position());
        //let spotlight_direction = &self.player.get_front();
        //light_vector.w = 1.0;

        //self.lighting_shader.setVec4(c"light.vector", &light_vector);
        //self.lighting_shader.setVec3(c"light.ambient", &self.ambient_color);
        //self.lighting_shader.setVec3(c"light.diffuse", &self.diffuse_color); // darken diuse light a bit
        //self.lighting_shader.setVec3(c"light.specular", &self.diffuse_color);
        //self.lighting_shader.setFloat(c"light.constant", 1.0);
        //self.lighting_shader.setFloat(c"light.linear", 0.045);
        //self.lighting_shader.setFloat(c"light.quadratic", 0.0075);
        //self.lighting_shader.setVec3(c"light.spotlightDirection", &spotlight_direction);
        //self.lighting_shader.setFloat(c"light.spAngleInner", f32::cos(0.21));
        //self.lighting_shader.setFloat(c"light.spAngleOuter", f32::cos(0.27));
    }
}
