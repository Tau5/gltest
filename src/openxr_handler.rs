use std::any::Any;
use std::ffi::c_void;
use std::ptr;
use fastrand::bool;
use gl::types::{GLfloat, GLint, GLsizei, GLuint};
use glutin::config::AsRawConfig;
use glutin::prelude::GlSurface;
use nalgebra_glm::{proj, quat, Mat4, TMat, TMat4, TVec3, TVec4};
use openxr as xr;
use openxr::{EnvironmentBlendMode, Event, ExtensionSet, FrameStream, FrameWaiter, Instance, OpenGL, SessionState, SessionStateChanged, Space, Swapchain, SystemId, ViewConfigurationView};
use openxr::sys::create_swapchain;
use crate::app::{App, GlThings};
use crate::renderer::{CameraRenderInfo, Renderer, RendererConfig};
use crate::util;
use crate::world::World;
use nalgebra_glm as glm;
use crate::openxr_input::OpenXRInput;
use crate::openxr_props::OpenxrProps;

struct HandlerFrameState {
    xr_state: xr::FrameState,
    view_flags: xr::ViewStateFlags,
    views: Vec<xr::View>,
}

pub struct OpenXRHandler {
    pub(crate) instance: xr::Instance,
    pub system: xr::SystemId,
    pub system_properties: xr::SystemProperties,
    pub session: xr::Session<OpenGL>,
    pub swapchains: Vec<Swapchain<OpenGL>>,
    pub images: Vec<Vec<u32>>,
    pub gl_framebuffers: Vec<GLuint>,
    pub frame_wait: FrameWaiter,
    pub frame_stream: FrameStream<OpenGL>,
    pub frame_state: Option<HandlerFrameState>,
    pub reference_space: Space,
    pub view_configuration_views: Vec<ViewConfigurationView>,
    pub do_framecycle: bool,
    pub resolution_multiplier: f32,
    pub input: OpenXRInput,
    pub world_scale: f32,
}

impl OpenXRHandler {
    pub fn new(gl_things: &GlThings) -> Self {
        let resolution_multiplier = 0.5;
        let application_info = xr::ApplicationInfo {
            application_name: "glTest OpenXR Application",
            application_version: 1,
            engine_name: "glTest",
            engine_version: 1,
            api_version: openxr::CURRENT_API_VERSION,
        };

        println!("Creating instance");
        let instance = Self::create_instance(&application_info);

        println!("Acquiring system id");
        let system = instance
            .system(xr::FormFactor::HEAD_MOUNTED_DISPLAY)
            .expect("Couldn't get system id");

        let system_properties = instance.system_properties(system).expect("Couldn't get sys prop");

        let view_configuration_views = Self::get_view_configuration_views(
            &instance, &system,
        );

        let reqs = instance.graphics_requirements::<xr::OpenGL>(system).expect("Couldn't get graphics requirements");

        let session_create_info = util::session_create_info(&gl_things.context, &gl_things.window).unwrap();

        let (session, mut frame_wait, mut frame_stream) = unsafe {
            instance.create_session::<xr::OpenGL>(
                system,
                &session_create_info,
            ).unwrap()
        };

        let (swapchains, images, gl_framebuffers) = Self::create_swapchains(&session, &view_configuration_views, resolution_multiplier);

        let space = Self::create_reference_space(&session);

        let input = OpenXRInput::new(&instance, &session);

        session.attach_action_sets(&[
            &input.action_set
        ]).unwrap();

        Self {
            instance,
            system,
            system_properties,
            session,
            swapchains,
            images,
            gl_framebuffers,
            frame_wait,
            frame_stream,
            reference_space: space,
            view_configuration_views,
            frame_state: None,
            do_framecycle: false,
            resolution_multiplier,
            input,
            world_scale: 2.0,
        }
    }

    fn get_enviroment_blend_mode(instance: &xr::Instance, system: xr::SystemId) -> EnvironmentBlendMode {
        let env_blend_modes = instance.enumerate_environment_blend_modes(
            system, xr::ViewConfigurationType::PRIMARY_STEREO,
        ).unwrap();

        env_blend_modes
            .iter()
            .copied()
            .find(|&m|
                m == xr::EnvironmentBlendMode::OPAQUE)
            .unwrap()
    }

    fn create_reference_space(session: &xr::Session<xr::OpenGL>) -> Space {
        session.create_reference_space(xr::ReferenceSpaceType::STAGE,
                                       xr::Posef::IDENTITY,
        ).unwrap()
    }

    fn create_swapchains(session: &xr::Session<xr::OpenGL>, views: &Vec<xr::ViewConfigurationView>, resolution_multiplier: f32) -> (Vec<Swapchain<OpenGL>>, Vec<Vec<u32>>, Vec<GLuint>) {
        let swapchain_formats = session.enumerate_swapchain_formats().unwrap();

        let color_swapchain_format = swapchain_formats
            .iter()
            .copied()
            .find(|&f| f == gl::SRGB8_ALPHA8)
            .unwrap_or(swapchain_formats[0]);

        let depth_swapchain_format = swapchain_formats
            .iter()
            .copied()
            .find(|&f| f == gl::DEPTH_COMPONENT32F)
            .unwrap_or(swapchain_formats[0]);

        let mut swapchains = vec!();
        let mut images = vec!();

        for view in views {
            let swapchain = session.create_swapchain(&xr::SwapchainCreateInfo::<xr::OpenGL> {
                create_flags: xr::SwapchainCreateFlags::EMPTY,
                usage_flags: xr::SwapchainUsageFlags::SAMPLED | xr::SwapchainUsageFlags::COLOR_ATTACHMENT,
                format: color_swapchain_format,
                sample_count: view.recommended_swapchain_sample_count,
                width: (view.recommended_image_rect_width as f32 * resolution_multiplier) as u32,
                height: (view.recommended_image_rect_height as f32 * resolution_multiplier) as u32,
                face_count: 1,
                array_size: 1,
                mip_count: 1,
            }).unwrap();

            let swapchain_depth = session.create_swapchain(&xr::SwapchainCreateInfo::<xr::OpenGL> {
                create_flags: xr::SwapchainCreateFlags::EMPTY,
                usage_flags: xr::SwapchainUsageFlags::SAMPLED | xr::SwapchainUsageFlags::DEPTH_STENCIL_ATTACHMENT,
                format: depth_swapchain_format,
                sample_count: view.recommended_swapchain_sample_count,
                width: (view.recommended_image_rect_width as f32 * resolution_multiplier) as u32,
                height: (view.recommended_image_rect_height as f32 * resolution_multiplier) as u32,
                face_count: 1,
                array_size: 1,
                mip_count: 1,
            }).unwrap();

            let sp_images = swapchain.enumerate_images().unwrap();
            let spd_images = swapchain_depth.enumerate_images().unwrap();

            images.push(sp_images);
            swapchains.push(swapchain);

            images.push(spd_images);
            swapchains.push(swapchain_depth);
        }

        let mut gl_framebuffers = vec![];
        for _ in &images {
            let mut fb_id: GLuint = 0;
            unsafe {
                gl::CreateFramebuffers(1, ptr::from_mut(&mut fb_id));
            }
            gl_framebuffers.push(fb_id)
        }

        return (swapchains, images, gl_framebuffers);
    }

    fn get_view_configuration_views(instance: &Instance, system: &SystemId) -> Vec<ViewConfigurationView> {
        instance.enumerate_view_configuration_views(
            *system,
            xr::ViewConfigurationType::PRIMARY_STEREO,
        ).unwrap()
    }

    fn create_instance(application_info: &xr::ApplicationInfo) -> Instance {
        let entry = openxr::Entry::linked();

        let available_extensions = entry.enumerate_extensions().unwrap();
        let layers = entry.enumerate_layers().unwrap();

        if (!available_extensions.ext_debug_utils || !available_extensions.khr_opengl_enable ) {
            if (!available_extensions.ext_debug_utils) {
                panic!("OpenXR Error: Extension EXT_DEBUG_UTIL not available");
            } else {
                panic!("OpenXR Error: Extension KXR_OPENGL_ENABLE not available");
            }
        }

        let layers_str: Vec<&str> = Vec::new(); //layers.iter().map(|l| l.layer_name.as_str()).collect();

        let mut extensions = ExtensionSet::default();

        extensions.ext_debug_utils = true;
        extensions.khr_opengl_enable = true;

        let instance = entry.create_instance(
            &application_info,
            &extensions,
            &layers_str,
        ).unwrap();
        instance
    }

    pub fn process_events(&mut self) {
        let mut data_buffer = xr::EventDataBuffer::new();
        while let Some(event) = self.instance.poll_event(&mut data_buffer).unwrap() {
            match event {
                Event::SessionStateChanged(state_change) => {
                    match state_change.state() {
                        SessionState::READY => {
                            println!("\n[INFO] [STATE] READY!");
                            self.session.begin(xr::ViewConfigurationType::PRIMARY_STEREO).unwrap();
                            self.do_framecycle = true;
                        },
                        SessionState::IDLE => {
                            println!("\n[INFO] [STATE] IDLE");
                            self.do_framecycle = false;
                        },
                        SessionState::UNKNOWN => {
                            println!("\n[INFO] [STATE] UNKNOWN");
                            self.do_framecycle = false;
                        },
                        ev => {
                            println!("\n[INFO] [UNHANDLED SESSION STATE] {:?}", ev)
                        },
                    }
                }
                ev => {
                    println!("\n[INFO] [UNHANDLED EVENT]");
                }

            }
        }
        //   fun(event);
        //}

    }

    pub fn wait_frame(&mut self) {
        let frame_state = self.frame_wait.wait().unwrap();

        self.input.poll_actions(&self.session, frame_state.predicted_display_time);

        let (view_flags, views) = self.session
            .locate_views(
                openxr::ViewConfigurationType::PRIMARY_STEREO,
                frame_state.predicted_display_time,
                &self.reference_space,
            )
            .unwrap();

        self.frame_state = Some(HandlerFrameState {
            xr_state: frame_state,
            view_flags,
            views,
        })
    }

    pub fn begin_frame(&mut self) {
        self.frame_stream.begin().unwrap();
    }

    pub fn get_quat(&self) -> Option<glm::Quat> {
        if let Some(state) = &self.frame_state {
            let qua = glm::quat(
                state.views[0].pose.orientation.x,
                state.views[0].pose.orientation.y,
                state.views[0].pose.orientation.z,
                state.views[0].pose.orientation.w,
            );

            return Some(qua)
        }

        None
    }

    pub fn render(&mut self, renderer: &mut Renderer, world: &mut World, glview: &TMat4<GLfloat>, glmodel: &TMat4<GLfloat>, renderer_config: &RendererConfig, camera_render_info: CameraRenderInfo, rot_offset: f32, props: &OpenxrProps) {
        self.begin_frame();

        let mut vr_camera_render_info = CameraRenderInfo {
            position: Default::default(),
            front: Default::default(),
        };

        if let Some(state) = &self.frame_state {

            for (view_idx, view) in self.view_configuration_views.iter().enumerate() {
                if !state.xr_state.should_render {
                    continue;
                }

                let qua = glm::quat(
                    state.views[view_idx].pose.orientation.x,
                    state.views[view_idx].pose.orientation.y,
                    state.views[view_idx].pose.orientation.z,
                    state.views[view_idx].pose.orientation.w,
                );
                
                let turn_mat = glm::rotation(rot_offset, &glm::vec3(0.0, 1.0, 0.0));


                let local_pos = glm::vec3(
                    state.views[view_idx].pose.position.x,
                    state.views[view_idx].pose.position.y,
                    state.views[view_idx].pose.position.z,
                );

                let local_pos_other_eye = glm::vec3(
                    state.views[(view_idx + 1) % 2].pose.position.x,
                    state.views[(view_idx + 1) % 2].pose.position.y,
                    state.views[(view_idx + 1) % 2].pose.position.z,
                );

                let ipd = local_pos - local_pos_other_eye;

                let roomscale = local_pos - ipd;

                let ipd_scaled = ipd * props.ipd_scale;

                let roomscale_scaled = glm::vec3(
                    roomscale.x * props.roomscale_scale,
                    roomscale.y * props.roomscale_scale,
                    roomscale.z * props.roomscale_scale
                );

                let local_pos = ipd_scaled + roomscale_scaled;

                //let qua = glm::quat_rotate(&qua, rot_offset, &glm::vec3(0.0, 1.0, 0.0)).normalize();
                let local_pos_rotated = &turn_mat * &glm::vec3_to_vec4(&local_pos);

                let pos: TVec3<f32> = local_pos_rotated.xyz() + camera_render_info.position;

                let proj = make_proj(state.views[view_idx].fov, 0.01, 100.0);
                //let proj = glm::perspective_fov(
                //    state.views[view_idx].fov.angle_up * 2.0,
                //    (view.recommended_image_rect_width as f32 * self.resolution_multiplier),
                //    (view.recommended_image_rect_height as f32 * self.resolution_multiplier),
                //    0.01, 100.0
                //);
                let rotmat = turn_mat * glm::quat_to_mat4(&qua);
                let transmat: TMat4<GLfloat> = glm::translation(&pos);

                if view_idx == 0 {
                    vr_camera_render_info.front = glm::quat_euler_angles(&qua);
                    vr_camera_render_info.position = pos;
                }
                let viewmat = transmat * rotmat;
                let viewmat = glm::inverse(&viewmat);

                let xr_swapchain_img_idx = self.swapchains[view_idx * 2].acquire_image().unwrap();
                self.swapchains[view_idx * 2].wait_image(xr::Duration::INFINITE).unwrap();

                let xr_depth_swapchain_img_idx = self.swapchains[view_idx * 2 + 1].acquire_image().unwrap();
                self.swapchains[view_idx * 2 + 1].wait_image(xr::Duration::INFINITE).unwrap();

                let gl_framebuffer = self.gl_framebuffers[view_idx * 2];
                let gl_framebuffer_depth = self.gl_framebuffers[view_idx * 2 + 1];

                unsafe {
                    gl::BindFramebuffer(gl::FRAMEBUFFER, gl_framebuffer);
                    gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, self.images[view_idx * 2][xr_swapchain_img_idx as usize], 0);
                    gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::TEXTURE_2D, self.images[view_idx * 2 + 1][xr_depth_swapchain_img_idx as usize], 0);

                    gl::Viewport(0, 0,
                                 (view.recommended_image_rect_width as f32 * self.resolution_multiplier) as GLsizei,
                                 (view.recommended_image_rect_height as f32 * self.resolution_multiplier) as GLsizei);

                    gl::Scissor(0, 0,
                                (view.recommended_image_rect_width as f32 * self.resolution_multiplier) as GLsizei,
                                (view.recommended_image_rect_height as f32 * self.resolution_multiplier) as GLsizei);
                }



                //let texture = self.images[view_idx][xr_swapchain_img_idx as usize];
                //let texture = unsafe {
                //    gl::CreateTe
                //}

                //app.render(&glmodel, &glview);
                renderer.render(
                    world,
                    &viewmat,
                    &glmodel,
                    renderer_config,
                    camera_render_info,
                    &proj
                );

                unsafe {
                    gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
                }

                //if (view_idx == 0) {
                //    unsafe {
                //        gl::BlitNamedFramebuffer(self.gl_framebuffers[0],
                //                                 0, 0, 0, view.recommended_image_rect_width as GLint, view.recommended_image_rect_height as GLint, 800, 600, gl::COLOR_BUFFER_BIT as GLint, gl::LINEAR as GLint, 0, 0);

                //        renderer.gl_things.surface.swap_buffers(&renderer.gl_things.context);
                //    }
                //}

                self.swapchains[view_idx * 2].release_image();
                self.swapchains[view_idx * 2 + 1].release_image();
            }
            self.end_frame(state.xr_state.predicted_display_time);;
        }
    }

    pub fn end_frame(&mut self, display_time: xr::Time) {
        if let Some(state) = &self.frame_state {
            if (state.xr_state.should_render) {
                self.frame_stream
                    .end(
                        state.xr_state.predicted_display_time,
                        xr::EnvironmentBlendMode::OPAQUE,
                        &[&openxr::CompositionLayerProjection::new()
                            .space(&self.reference_space)
                            .views(&[
                                openxr::CompositionLayerProjectionView::new()
                                    .pose(state.views[1].pose)
                                    .fov(state.views[1].fov)
                                    .sub_image(
                                        openxr::SwapchainSubImage::new()
                                            .swapchain(&self.swapchains[2])
                                            .image_array_index(0)
                                            .image_rect(openxr::Rect2Di {
                                                offset: openxr::Offset2Di { x: 0, y: 0 },
                                                extent: xr::Extent2Di {
                                                    width: (self.view_configuration_views[0].recommended_image_rect_width as f32 * self.resolution_multiplier) as i32,
                                                    height: (self.view_configuration_views[0].recommended_image_rect_height as f32 * self.resolution_multiplier) as i32,
                                                },
                                            }),
                                    ),
                                openxr::CompositionLayerProjectionView::new()
                                    .pose(state.views[0].pose)
                                    .fov(state.views[0].fov)
                                    .sub_image(
                                        openxr::SwapchainSubImage::new()
                                            .swapchain(&self.swapchains[0])
                                            .image_array_index(0)
                                            .image_rect(openxr::Rect2Di {
                                                offset: openxr::Offset2Di { x: 0, y: 0 },
                                                extent: xr::Extent2Di {
                                                    width: (self.view_configuration_views[1].recommended_image_rect_width as f32 * self.resolution_multiplier) as i32,
                                                    height: (self.view_configuration_views[1].recommended_image_rect_height as f32 * self.resolution_multiplier) as i32,
                                                },
                                            }),
                                    ),
                            ])],
                    )
                    .unwrap();
            } else {
                self.frame_stream.end(state.xr_state.predicted_display_time, EnvironmentBlendMode::OPAQUE, &[]).unwrap()
            }
        }
    }
}

pub fn make_proj(fov: xr::Fovf, nearZ: f32, farZ: f32) -> Mat4 {
    let tanAngleLeft = f32::tan(fov.angle_left);
    let tanAngleRight = f32::tan(fov.angle_right);

    let tanAngleDown = f32::tan(fov.angle_down);
    let tanAngleUp = f32::tan(fov.angle_up);

    let tanAngleWidth = tanAngleRight - tanAngleLeft;

    // Set to tanAngleDown - tanAngleUp for a clip space with positive Y
    // down (Vulkan). Set to tanAngleUp - tanAngleDown for a clip space with
    // positive Y up (OpenGL / D3D / Metal).
    let tanAngleHeight = (tanAngleUp - tanAngleDown);

    // Set to nearZ for a [-1,1] Z clip space (OpenGL / OpenGL ES).
    // Set to zero for a [0,1] Z clip space (Vulkan / D3D / Metal).
    let offsetZ = nearZ;

    let mut result = Mat4::identity();

    if (farZ <= nearZ) {
        // place the far plane at infinity
        result.m11 = 2.0 / tanAngleWidth;
        result.m12 = 0.0;
        result.m13 = (tanAngleRight + tanAngleLeft) / tanAngleWidth;
        result.m14 = 0.0;

        result.m21 = 0.0;
        result.m22 = 2.0 / tanAngleHeight;
        result.m23 = (tanAngleUp + tanAngleDown) / tanAngleHeight;
        result.m24 = 0.0;

        result.m31 = 0.0;
        result.m32 = 0.0;
        result.m33 = -1.0;
        result.m34 = -(nearZ + offsetZ);

        result.m41 = 0.0;
        result.m42 = 0.0;
        result.m43 = -1.0;
        result.m44 = 0.0;
    } else {
        // normal projection
        result.m11= 2.0 / tanAngleWidth;
        result.m12= 0.0;
        result.m13= (tanAngleRight + tanAngleLeft) / tanAngleWidth;
        result.m14 = 0.0;

        result.m21= 0.0;
        result.m22= 2.0 / tanAngleHeight;
        result.m23= (tanAngleUp + tanAngleDown) / tanAngleHeight;
        result.m24 = 0.0;

        result.m31= 0.0;
        result.m32= 0.0;
        result.m33 = -(farZ + offsetZ) / (farZ - nearZ);
        result.m34 = -(farZ * (nearZ + offsetZ)) / (farZ - nearZ);

        result.m41= 0.0;
        result.m42= 0.0;
        result.m43 = -1.0;
        result.m44 = 0.0;
    }

    result
}