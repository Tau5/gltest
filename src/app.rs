use std::cell::Cell;
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
use std::rc::Rc;
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
use crate::openxr_handler::OpenXRHandler;
use crate::renderer::{CameraRenderInfo, Renderer, RendererConfig};
use crate::world::World;

use std::f32::consts::TAU as f32_TAU;
use std::ops::{Add, Mul};
use crate::openxr_props::OpenxrProps;

struct AppState {
    gl_surface: Surface<WindowSurface>,
    // NOTE: Window should be dropped after all resources created using its
    // raw-window-handle.
    window: Window,
}

pub struct GlThings {
    pub surface: Surface<WindowSurface>,
    pub context: glutin::context::PossiblyCurrentContext,
    pub display: glutin::display::Display,
    pub config: Config,
    pub window: Window
}

pub struct App {
    camera_speed: GLfloat,
    mouse_sensitivity: c_double,
    last_y: c_double,
    last_x: c_double,
    current_x: c_double,
    current_y: c_double,
    player: Player,
    paused: bool,
    mouse_change_counter: i32,
    input_manager: InputManager,
    renderer_config: RendererConfig,
    renderer: Renderer,
    world: World,
    exit_state: Result<(), Box<dyn Error>>,
    openxr_handler: Option<OpenXRHandler>,
    proj: TMat4<GLfloat>,
    pub xr_rot_offset: f32,
    pub openxr_props: OpenxrProps,
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
        self.renderer.gl_things.window.request_redraw();
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
        gl::load_with(|s| {
            let symbol = CString::new(s).unwrap();
            gl_things.config.display().get_proc_address(symbol.as_c_str()).cast()
        });

        let openxr_handler = OpenXRHandler::new(&gl_things);

        event_loop.set_control_flow(ControlFlow::Wait);
        gl_things.surface.set_swap_interval(
            &gl_things.context, SwapInterval::Wait(NonZero::new(60).unwrap())
        ).unwrap();

        let proj: TMat4<GLfloat> = glm::perspective(
            (width / height) as GLfloat,
            std::f32::consts::TAU / 8.0,
            0.1,
            100.0,
        );
        //gl_things.window.set_cursor_grab(winit::window::CursorGrabMode::Locked).unwrap();

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

        let mut renderer = Renderer::new(
            width as f32,
            height as f32,
            material_store,
            gl_things,
            light_manager
        );

        renderer.material_store
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

        renderer.material_store
            .load(
                "sand",
                Some(TextureSource::ImagePath("textures/sand.png".into())),
                Some(TextureSource::ImagePath("textures/sand_spec.png".into())),
                None,
                4.0,
            )
            .unwrap();

        renderer.material_store
            .load(
                "water",
                Some(TextureSource::ImagePath("textures/water.png".into())),
                None,
                None,
                16.0,
            )
            .unwrap();

        renderer.material_store
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

        let world = World::new(&mut renderer.material_store);
        let mut input_manager = InputManager::new();
        let renderer_config = RendererConfig {
            selected_obj: 0
        };


        Self {
            last_x: 0.0,
            last_y: 0.0,
            current_x: 0.0,
            current_y: 0.0,
            camera_speed: 0.2,
            mouse_sensitivity: 0.01,
            mouse_change_counter: 0,
            xr_rot_offset: 0.0,
            paused: false,
            player,
            input_manager,
            exit_state: Ok(()),
            openxr_handler: Some(openxr_handler),
            renderer_config,
            renderer,
            world,
            proj,
            openxr_props: OpenxrProps::default()
        }
        //      while unsafe { glfwWindowShouldClose(window) == 0 }
    }

    pub fn game_loop(&mut self) {
        //process_input(window);
        if let Some(openxr) = &mut self.openxr_handler {
            openxr.process_events();

            if (!openxr.do_framecycle) {
               return;
            }
            openxr.wait_frame();
        }
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
                    if self.renderer_config.selected_obj > 0 {
                        self.renderer_config.selected_obj -= 1;
                    }
                }
                if self.input_manager.get_status(Key::Named(NamedKey::ArrowRight)) == KeyStatus::JustPressed {
                    if self.renderer_config.selected_obj < self.world.objects.len() - 1 {
                        self.renderer_config.selected_obj += 1;
                    }
                }

                if self.input_manager.get_status(Key::Character("f".into())) == KeyStatus::JustPressed {
                    self.renderer.light_manager.toggle_spotlight();
                }

                if let Some(obj) = self.world.objects.get_mut(self.renderer_config.selected_obj) {
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
                    self.renderer.gl_things.window.set_cursor_grab(winit::window::CursorGrabMode::None).unwrap();
                    self.renderer.gl_things.window.set_cursor_visible(true);
                    self.mouse_change_counter = 0;
                } else {
                    //glfw::ffi::glfwSetInputMode(
                    //    window,
                    //    glfw::ffi::CURSOR,
                    //    glfw::ffi::CURSOR_DISABLED,
                    //);
                    self.renderer.gl_things.window.set_cursor_grab(winit::window::CursorGrabMode::Locked).unwrap();
                    self.renderer.gl_things.window.set_cursor_visible(false);
                }
            }

            if self.input_manager.get_status(Key::Character("z".into())) == KeyStatus::JustPressed {
                self.renderer.light_manager.directional_light.direction = self.player.get_front();
            }
            if let Some(light) = self.renderer.light_manager.get_mut_pointlight(0) {}
        }


        self.update_controller_objects();

        if let Some(openxr) = &self.openxr_handler {
           //if openxr.input.action_move.state(&openxr.session,
           //                            openxr.instance.string_to_path("/user/hand/right").unwrap()).unwrap().current_state {
           //    move_z += self.camera_speed;
           //}

            (move_x, move_z) = openxr.input.get_move(&openxr.session);
            
            move_x *= 0.1;
            move_z *= 0.1;

            // DIR * 30º
            self.xr_rot_offset += -openxr.input.snapturn_dir * f32_TAU * 1.0/4.0 * 1.0/3.0;
        }

        self.player.translate(move_x, move_z, &self.world.objects);
        self.player.apply_gravity(-0.27, &self.world.objects);

        if let Some(openxr) = &self.openxr_handler {
            if (openxr.input.action_roomscale_dec.state(&openxr.session, openxr.input.user_hand_left).unwrap().current_state) {
                self.openxr_props.roomscale_scale -= 0.1f32;
            } else if (openxr.input.action_roomscale_inc.state(&openxr.session, openxr.input.user_hand_right).unwrap().current_state) {
                self.openxr_props.roomscale_scale += 0.1f32;
            }

            if (openxr.input.action_ipd_dec.state(&openxr.session, openxr.input.user_hand_left).unwrap().current_state) {
                self.openxr_props.ipd_scale -= 0.1f32;
            } else if (openxr.input.action_ipd_inc.state(&openxr.session, openxr.input.user_hand_right).unwrap().current_state) {
                self.openxr_props.ipd_scale += 0.1f32;
            }
            
           if let Some(mut quat) = openxr.get_quat() {
               let rotmat = glm::rotation(self.xr_rot_offset, &glm::vec3(0.0, 1.0, 0.0)) *glm::quat_to_mat4(&quat)  ;
               let look_dir = rotmat *
                   glm::vec4(0.0, 0.0, 1.0, 1.0);
               //let new_dir = glm::quat_to_mat4(&quat) *
               //    glm::vec4(1.0, 0.0, 0.0, 1.0);
               self.player.set_camera_direction(-look_dir.xyz());
           }
        } else {
            self.player.rotate_camera(
                (mouse_x * self.mouse_sensitivity) as f32,
                -(mouse_y * self.mouse_sensitivity) as f32,
            );
        }

        let view = self.player.get_view();

        //self.lightpoint_pos.x = GLfloat::sin(time_value as f32) * 2.0 + 8.0;

        let maybe_handler = &mut self.openxr_handler;

        if let Some(openxr) = maybe_handler {
            let mut position_bottom = self.player.get_position();
            let player_size = (self.player.aabb.end - self.player.aabb.start);
            position_bottom.y -= player_size.y.abs();

            openxr.render(
                &mut self.renderer,
                &mut self.world,
                &view,
                &model,
                &self.renderer_config,
                CameraRenderInfo {
                    front: self.player.get_front(),
                    position: position_bottom,
                },
                self.xr_rot_offset,
                &self.openxr_props
            )
        }
        {
            self.renderer.render(
                &mut self.world,
                &view,
                &model,
                &self.renderer_config,
                CameraRenderInfo {
                    front: self.player.get_front(),
                    position: self.player.get_position()
                },
                &self.proj
            );

            unsafe {
                self.renderer.gl_things.surface.swap_buffers(&self.renderer.gl_things.context).unwrap()
            }
        }

        if (self.mouse_change_counter < 3)
            && ((self.last_x != self.current_x) || (self.last_y != self.current_y))
        {
            self.mouse_change_counter += 1;
        }

        self.last_x = self.current_x;
        self.last_y = self.current_y;

        self.input_manager.update_keys();
    }

    fn update_controller_objects(&mut self) {
        if let Some(handler) = &self.openxr_handler {
            if let Some(framestate) = &handler.frame_state {
                let (cont_left, cont_right) = handler.input.get_controller_locations(&handler.reference_space, framestate.xr_state.predicted_display_time);

                let cont_left_q = glm::quat(
                    cont_left.pose.orientation.x,
                    cont_left.pose.orientation.y,
                    cont_left.pose.orientation.z,
                    cont_left.pose.orientation.w,
                );

                let cont_right_q = glm::quat(
                    cont_right.pose.orientation.x,
                    cont_right.pose.orientation.y,
                    cont_right.pose.orientation.z,
                    cont_right.pose.orientation.w,
                );

                self.world.controller_objects[0].set_rotation(
                    cont_left_q
                );

                let mut position_bottom = self.player.get_position();
                let player_size = (self.player.aabb.end - self.player.aabb.start);
                position_bottom.y -= player_size.y.abs();

                let vec_roomscale = glm::vec3(
                    self.openxr_props.roomscale_scale / 2.0,
                    self.openxr_props.roomscale_scale / 2.0,
                    self.openxr_props.roomscale_scale / 2.0,
                );
                self.world.controller_objects[0].set_scale(vec_roomscale);
                self.world.controller_objects[1].set_scale(vec_roomscale);

                let size_l = self.world.controller_objects[0].get_size();
                let size_r = self.world.controller_objects[1].get_size();
                self.world.controller_objects[0].set_translate(
                    glm::vec3(
                        cont_left.pose.position.x,
                        cont_left.pose.position.y,
                        cont_left.pose.position.z
                    )
                        .mul(self.openxr_props.roomscale_scale)
                        .add(&position_bottom)
                );

                self.world.controller_objects[1].set_rotation(
                    cont_right_q
                );

                self.world.controller_objects[1].set_translate(
                    glm::vec3(
                        cont_right.pose.position.x,
                         cont_right.pose.position.y,
                        cont_right.pose.position.z
                    )
                        .mul(self.openxr_props.roomscale_scale)
                        .add(&position_bottom)
                )
            }
        }
    }
}
