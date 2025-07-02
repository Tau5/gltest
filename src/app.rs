use std::error::Error;
use crate::camera::Camera;
use crate::cube::Cube;
use crate::input::{InputManager, KeyStatus};
use crate::lighting::{DirLight, LightManager, PointLight, SpotLight};
use crate::model::Model;
use crate::object::Object;
use crate::player::Player;
use crate::{config_picker, create_gl_context, prefabs};
use crate::shader::ShaderProgram;
use crate::textures::{MaterialStore, TextureSource};
use crate::vao::{TriangleArrayVAO, VAO};
use gl::types::GLfloat;
use nalgebra_glm as glm;
use nalgebra_glm::{TMat, TMat4, TVec3};
use std::ffi::{c_double, CString};
use std::num::{NonZero, NonZeroU32};
use glutin::config::{Config, ConfigTemplateBuilder, GetGlConfig, GlConfig};
use glutin::display::GetGlDisplay;
use glutin::prelude::{GlDisplay, GlSurface, NotCurrentGlContext, PossiblyCurrentGlContext};
use glutin::surface::{Surface, SwapInterval, WindowSurface};
use glutin_winit::{DisplayBuilder, GlWindow};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

struct AppState {
    gl_surface: Surface<WindowSurface>,
    // NOTE: Window should be dropped after all resources created using its
    // raw-window-handle.
    window: Window,
}

struct GlThings {
    surface: Surface<WindowSurface>,
    context: glutin::context::PossiblyCurrentContext,
    display: glutin::display::Display,
    config: Config,
    window: Window
}

pub struct App {
    objects: Vec<Object>,
    lighting_shader: ShaderProgram,
    lightpoint_shader: ShaderProgram,
    aabb_shader: ShaderProgram,
    border_shader: ShaderProgram,

    camera_speed: GLfloat,
    mouse_sensitivity: c_double,
    last_y: c_double,
    last_x: c_double,
    current_x: c_double,
    current_y: c_double,

    paused: bool,
    selected_obj: usize,
    proj: TMat4<f32>,
    mouse_change_counter: i32,
    player: Player,

    material_store: MaterialStore,
    input_manager: InputManager,
    light_manager: LightManager,
    pub lamp_vao: TriangleArrayVAO,

    gl_things: GlThings,

    template: ConfigTemplateBuilder,
    exit_state: Result<(), Box<dyn Error>>,
}

impl App {
    fn load_gl(template: ConfigTemplateBuilder, display_builder: DisplayBuilder, event_loop: &EventLoop<()>) -> GlThings {
        // We just created the event loop, so initialize the display, pick the config, and
        // create the context.
        let (window, gl_config) = match display_builder.clone().build(
            event_loop,
            template.clone(),
            config_picker,
        ) {
            Ok((window, gl_config)) => (window.unwrap(), gl_config),
            Err(err) => {
                panic!()
                },
        };

        println!("w{}h{}", window.inner_size().width, window.inner_size().height);

        println!("Picked a config with {} samples", gl_config.num_samples());

        // Mark the display as initialized to not recreate it on resume, since the
        // display is valid until we explicitly destroy it.
        //let gl_display = GlDisplayCreationState::Init;

        // Create gl context.
        let gl_context = create_gl_context(&window, &gl_config).treat_as_possibly_current();

        let gl_display = gl_context.display();

        let attrs = window
            .build_surface_attributes(Default::default())
            .expect("Failed to build surface attributes");

        let gl_surface =
            unsafe { gl_config.display().create_window_surface(&gl_config, &attrs).unwrap() };

        // The context needs to be current for the Renderer to set up shaders and
        // buffers. It also performs function loading, which needs a current context on
        // WGL.
        gl_context.make_current(&gl_surface).unwrap();

        //self.renderer.get_or_insert_with(|| Renderer::new(&gl_config.display()));

        // Try setting vsync.
        if let Err(res) = gl_surface
            .set_swap_interval(&gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))
        {
            eprintln!("Error setting vsync: {res:?}");
        }

        GlThings {
            surface: gl_surface,
            context: gl_context,
            display: gl_display,
            config: gl_config,
            window
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.game_loop()
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CursorMoved {
                position, ..
            } => {
                self.current_x = position.x;
                self.current_y = position.y;
            },

            WindowEvent::KeyboardInput {
                event, ..
            } => {
                self.input_manager.winit_key_event(event)
            }
            WindowEvent::RedrawRequested => {
                self.game_loop();
            },
            _ => {}
        }


    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.gl_things.window.request_redraw();
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        // This event is only raised on Android, where the backing NativeWindow for a GL
        // Surface can appear and disappear at any moment.
        println!("Android window removed");

        //// Destroy the GL Surface and un-current the GL Context before ndk-glue releases
        //// the window back to the system.
        //self.state = None;

        //// Make context not current.
        //self.gl_context = Some(
        //    self.gl_context.take().unwrap().make_not_current().unwrap().treat_as_possibly_current(),
        //);
    }
}

impl App {
    pub fn new(template: ConfigTemplateBuilder, display_b: DisplayBuilder, event_loop: &EventLoop<()>, width: usize, height: usize) -> App {
        let gl_things = Self::load_gl(template.clone(), display_b, event_loop);
        event_loop.set_control_flow(ControlFlow::Wait);
        gl_things.surface.set_swap_interval(
            &gl_things.context, SwapInterval::Wait(NonZero::new(60).unwrap())
        ).unwrap();

        gl::load_with(|s| {
                let symbol = CString::new(s).unwrap();
                gl_things.config.display().get_proc_address(symbol.as_c_str()).cast()
            }
        );

        gl_things.window.set_cursor_grab(winit::window::CursorGrabMode::Locked).unwrap();

        unsafe { gl::Viewport(0, 0, 800, 600); }

        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::STENCIL_TEST);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

            //gl::ClearColor(0.2, 0.2, 0.2, 1.0);
            //gl::ClearColor(0.21, 0.64, 0.99, 1.0);
            // Qt.rgba(0.21, 0.38, 0.52, 1)
            gl::ClearColor(0.08, 0.25, 0.4, 1.0);
        }

        let vertex_shader_text = include_str!("vertex_shader.glsl");

        let lighting_shader = ShaderProgram::new(vertex_shader_text, include_str!("lighting.glsl"));

        let lightpoint_shader =
            ShaderProgram::new(vertex_shader_text, include_str!("lightpoint.glsl"));

        let aabb_shader = ShaderProgram::new(
            include_str!("vertex_shader.glsl"),
            include_str!("aaab_renderer.glsl"),
        );

        let border_shader = ShaderProgram::new(
            include_str!("vertex_shader.glsl"),
            include_str!("border.glsl"),
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
            DirLight::new(lightpoint_pos, ambient_color, diffuse_color, diffuse_color),
            Some(spotlight),
        );
        light_manager
            .add_lightpoint(PointLight::new(
                glm::vec3(-5.0, -13.0, -5.0),
                glm::vec3(1.0, 0.2, 0.2),
                glm::vec3(1.0, 0.6, 0.6),
                glm::vec3(1.0, 0.6, 0.6),
                1.0,
                0.07,
                0.017,
            ))
            .unwrap();

        let lamp_vao = prefabs::cube();

        material_store
            .load(
                "container",
                Some(TextureSource::ImagePath("textures/container2.png".into())),
                Some(TextureSource::ImagePath(
                    "textures/container2_specular.png".into(),
                )),
                Some(TextureSource::ImagePath(
                    "textures/container2_emission.png".into(),
                )),
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
            .load(
                "water",
                Some(TextureSource::ImagePath("textures/water.png".into())),
                None,
                None,
                16.0,
            )
            .unwrap();

        material_store
            .load(
                "fogata",
                Some(TextureSource::ImagePath("textures/fogata.png".into())),
                None,
                None,
                1.0,
            )
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

        //input_manager.add_key(Key::Character("w".into()));
        //input_manager.add_key(Key::Character("a".into()));
        //input_manager.add_key(Key::Character("s".into()));
        //input_manager.add_key(Key::Character("d".into()));
        //input_manager.add_key(Key::Character("h".into()));
        //input_manager.add_key(Key::Character("j".into()));
        //input_manager.add_key(Key::Character("k".into()));
        //input_manager.add_key(Key::Character("l".into()));
        //input_manager.add_key(Key::Character("f".into()));
        //input_manager.add_key(Key::Character("z".into()));
        //input_manager.add_key(Key::Character("x".into()));
        //input_manager.add_key(Key::Character("c".into()));
        //input_manager.add_key(Key::Character("v".into()));
        //input_manager.add_key(Key::Character("b".into()));
        //input_manager.add_key(Key::Character("n".into()));
        //input_manager.add_key(Key::Character("m".into()));
        //input_manager.add_key(Key::Named(NamedKey::ArrowLeft));
        //input_manager.add_key(Key::Named(NamedKey::ArrowRight));
        //input_manager.add_key(Key::Named(NamedKey::Escape));

        //unsafe { glfw::ffi::glfwSetInputMode(window, glfw::ffi::CURSOR, glfw::ffi::CURSOR_DISABLED); }
        Self {
            last_x: 0.0,
            last_y: 0.0,
            current_x: 0.0,
            current_y: 0.0,
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
            border_shader,
            material_store,
            light_manager,
            lamp_vao,
            input_manager,
            exit_state: Ok(()),
            gl_things,
            template,
        }
        //      while unsafe { glfwWindowShouldClose(window) == 0 }
    }

    fn generate_objects(mut material_store: &mut MaterialStore) -> Vec<Object> {
        let testcube_pos = glm::vec3(8.0, 0.0, -2.0);
        let plane_pos = glm::vec3(8.0, -5.0, -2.0);
        let mut model_test = Model::new(
            "models/example.glb".into(),
            "kit".into(),
            &mut material_store,
        );

        let mut objects = Vec::new();

        let container = Object::new(
            testcube_pos,
            glm::vec3(0.0, 0.0, 0.0),
            glm::vec3(2.0, 1.0, 2.0),
            Box::from(Cube::new(material_store.get("container"))),
            true,
        );

        let plane = Object::new(
            plane_pos,
            glm::vec3(0.0, 0.0, 0.0),
            glm::vec3(10.0, 1.0, 10.0),
            Box::from(Cube::new(material_store.get("box"))),
            true,
        );

        let beach_sand = Object::new(
            glm::vec3(0.0, -15.0, 0.0),
            glm::vec3(0.0, 0.0, 0.0),
            glm::vec3(40.0, 0.1, 20.0),
            Box::from(Cube::new(material_store.get("sand"))),
            true,
        );

        let beach_waterbed = Object::new(
            glm::vec3(0.0, -20.0, 0.0),
            glm::vec3(0.0, 0.0, 0.0),
            glm::vec3(70.0, 0.1, 70.0),
            Box::from(Cube::new(material_store.get("sand"))),
            true,
        );

        let ocean = Object::new(
            glm::vec3(0.0, -15.1, 0.0),
            glm::vec3(0.0, 0.0, 0.0),
            glm::vec3(100.0, 0.1, 100.0),
            Box::from(Cube::new(material_store.get("water"))),
            true,
        );

        let fire = Object::from_model(
            glm::vec3(-5.0, -14.0, -5.0),
            glm::vec3(0.0, 0.0, 0.0),
            glm::vec3(1.0, 1.0, 1.0),
            true,
            model_test,
            material_store,
        );

        let crank = Object::from_model(
            glm::vec3(-2.0, -14.0, -5.0),
            glm::vec3(0.0, 0.0, 0.0),
            glm::vec3(0.1, 0.1, 0.1),
            true,
            Model::new("models/crank.glb".into(), "crank".into(), material_store),
            material_store,
        );

        let kleiner = Object::from_model(
            glm::vec3(-2.0, -14.0, 0.0),
            glm::vec3(0.0, 0.0, 0.0),
            glm::vec3(0.1, 0.1, 0.1),
            true,
            Model::new(
                "models/kleiner.glb".into(),
                "kleiner".into(),
                material_store,
            ),
            material_store,
        );

        let grass = Object::from_model(
            glm::vec3(-2.0, -14.0, 3.0),
            glm::vec3(0.0, 0.0, 0.0),
            glm::vec3(1.0, 1.0, 1.0),
            true,
            Model::new("models/grass.glb".into(), "grass".into(), material_store),
            material_store,
        );

        objects.push(container);
        objects.push(plane);
        objects.push(beach_sand);
        objects.push(beach_waterbed);
        objects.push(ocean);
        objects.push(fire);
        objects.push(crank);
        objects.push(kleiner);
        objects.push(grass);

        objects
    }

    pub fn game_loop(&mut self) {
        //process_input(window);
        let model = glm::identity::<f32, 4>();

        if !self.paused {
            //unsafe {
            //    glfwGetCursorPos(
            //        window,
            //        ptr::from_mut(&mut current_x),
            //        ptr::from_mut(&mut current_y),
            //    );
            //}
        }

        let mouse_x = if self.mouse_change_counter > 2 {
            self.current_x - self.last_x
        } else {
            0.0
        };

        let mouse_y = if self.mouse_change_counter > 2 {
            self.current_y - self.last_y
        } else {
            0.0
        };

        let mut move_x: f32 = 0.0;
        let mut move_z: f32 = 0.0;

        //self.input_manager.poll(&self.window);

        unsafe {
            if !self.paused {
                if self.input_manager.get_status(Key::Character("d".into())) >= KeyStatus::JustPressed {
                    move_x += self.camera_speed
                }
                if self.input_manager.get_status(Key::Character("a".into())) >= KeyStatus::JustPressed {
                    move_x -= self.camera_speed;
                }
                if self.input_manager.get_status(Key::Character("w".into())) >= KeyStatus::JustPressed {
                    move_z += self.camera_speed;
                }
                if self.input_manager.get_status(Key::Character("s".into())) >= KeyStatus::JustPressed {
                    move_z -= self.camera_speed;
                }
                if self.input_manager.get_status(Key::Named(NamedKey::ArrowLeft)) == KeyStatus::JustPressed {
                    if self.selected_obj > 0 {
                        self.selected_obj -= 1;
                    }
                }
                if self.input_manager.get_status(Key::Named(NamedKey::ArrowRight)) == KeyStatus::JustPressed {
                    if self.selected_obj < self.objects.len() - 1 {
                        self.selected_obj += 1;
                    }
                }

                if self.input_manager.get_status(Key::Character("f".into())) == KeyStatus::JustPressed {
                    self.light_manager.toggle_spotlight();
                }

                if let Some(obj) = self.objects.get_mut(self.selected_obj) {
                    if self.input_manager.get_status(Key::Character("i".into())) == KeyStatus::JustPressed {
                        obj.scale(0.1, 0.0, 0.0);
                    }
                    if self.input_manager.get_status(Key::Character("l".into())) == KeyStatus::JustPressed {
                        obj.scale(0.0, 0.0, 0.1);
                    }
                }
            }

            if self.input_manager.get_status(Key::Named(NamedKey::Escape)) == KeyStatus::JustPressed {
                self.paused = !self.paused;
                if self.paused {
                    //glfw::ffi::glfwSetInputMode(
                    //    window,
                    //    glfw::ffi::CURSOR,
                    //    glfw::ffi::CURSOR_NORMAL,
                    //);
                    self.gl_things.window.set_cursor_grab(winit::window::CursorGrabMode::None).unwrap();
                    self.gl_things.window.set_cursor_visible(true);
                    self.mouse_change_counter = 0;
                } else {
                    //glfw::ffi::glfwSetInputMode(
                    //    window,
                    //    glfw::ffi::CURSOR,
                    //    glfw::ffi::CURSOR_DISABLED,
                    //);
                    self.gl_things.window.set_cursor_grab(winit::window::CursorGrabMode::Locked).unwrap();
                    self.gl_things.window.set_cursor_visible(false);
                }
            }

            if self.input_manager.get_status(Key::Character("z".into())) == KeyStatus::JustPressed {
                self.light_manager.directional_light.direction = self.player.get_front();
            }
            if let Some(light) = self.light_manager.get_mut_pointlight(0) {}
        }

        self.player.translate(move_x, move_z, &self.objects);
        self.player.apply_gravity(-0.27, &self.objects);
        self.player.rotate_camera(
            (mouse_x * self.mouse_sensitivity) as f32,
            -(mouse_y * self.mouse_sensitivity) as f32,
        );
        let view = self.player.get_view();

        //self.lightpoint_pos.x = GLfloat::sin(time_value as f32) * 2.0 + 8.0;

        self.render(&model, &view);

        if (self.mouse_change_counter < 3)
            && ((self.last_x != self.current_x) || (self.last_y != self.current_y))
        {
            self.mouse_change_counter += 1;
        }

        self.last_x = self.current_x;
        self.last_y = self.current_y;

        self.input_manager.update_keys();
    }

    fn render(&mut self, model: &TMat<f32, 4, 4>, view: &TMat4<GLfloat>) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
            gl::Enable(gl::DEPTH_TEST);
        }

        // mat3(transpose(inverse(model)))

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

        self.load_lighting_shader(&view);

        unsafe {
            gl::StencilMask(0x00);
        }
        // Render all objects normally except selected
        for (i, obj) in self.objects.iter().enumerate() {
            if (i != self.selected_obj) {
                obj.render(&self.lighting_shader, &self.material_store);
            }
        }

        unsafe {
            // If depth and stencil tests pass, replace value in stencil
            // with the ref value in StencilFunc.
            // If any fail, don't change stencil buffer for that fragment
            gl::StencilOp(gl::KEEP, gl::KEEP, gl::REPLACE);

            // Stencil test ALWAYS passes. Ref value is 1 and because we used
            // REPLACE in StencilOp, we'll change the stencil value for each fragment
            // to 1
            gl::StencilFunc(gl::ALWAYS, 1, 0xFF);

            // Enable writing to the stencil buffer
            gl::StencilMask(0xFF);
        }

        // Render all objects normally except selected

        if let Some(obj) = self.objects.get_mut(self.selected_obj) {
            obj.render(&self.lighting_shader, &self.material_store);
        }

        unsafe {
            // Stencil test passes if the fragment wasn't written to
            gl::StencilFunc(gl::NOTEQUAL, 1, 0xFF);

            // Do NOT write to the stencil buffer
            gl::StencilMask(0x00);

            // We don't wan't the depth test involved in this
            //gl::Disable(gl::DEPTH_TEST);
        }

        self.border_shader.load();
        self.border_shader.setMat4(c"view", &view);
        self.border_shader.setMat4(c"projection", &self.proj);

        if let Some(obj) = self.objects.get_mut(self.selected_obj) {
            obj.render_scaled(&self.border_shader, &self.material_store);
        }

        unsafe {
            gl::StencilMask(0xFF);
            gl::StencilFunc(gl::ALWAYS, 1, 0xFF);
        }

        //self.aabb_shader.load();
        //self.aabb_shader.setMat4(c"view", &view);
        //self.aabb_shader.setMat4(c"projection", &self.proj);

        //if let Some(obj) = self.objects.get(self.selected_obj) {
        //    obj.debug_render_aabb(&self.aabb_shader);
        //}

        unsafe {
            self.gl_things.surface.swap_buffers(&self.gl_things.context).unwrap()
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
