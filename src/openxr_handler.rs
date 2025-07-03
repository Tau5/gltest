use std::ffi::c_void;
use std::ptr;
use gl::types::{GLfloat, GLuint};
use glutin::config::AsRawConfig;
use nalgebra_glm::{TMat, TMat4};
use openxr as xr;
use openxr::{EnvironmentBlendMode, Event, ExtensionSet, FrameStream, FrameWaiter, Instance, OpenGL, SessionState, SessionStateChanged, Space, Swapchain, SystemId, ViewConfigurationView};
use openxr::sys::create_swapchain;
use crate::app::{App, GlThings};
use crate::util;

struct HandlerFrameState {
    xr_state: xr::FrameState,
    view_flags: xr::ViewStateFlags,
    views: Vec<xr::View>,
}

pub struct OpenXRHandler {
    instance: xr::Instance,
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
}

impl OpenXRHandler {
    pub fn new(gl_things: &GlThings) -> Self {
        let application_info = xr::ApplicationInfo {
            application_name: "glTest OpenXR Application",
            application_version: 1,
            engine_name: "glTest",
            engine_version: 1,
            api_version: openxr::CURRENT_API_VERSION,
        };

        let instance = Self::create_instance(&application_info);

        let system = instance
            .system(xr::FormFactor::HEAD_MOUNTED_DISPLAY)
            .unwrap();

        let system_properties = instance.system_properties(system).unwrap();

        let view_configuration_views = Self::get_view_configuration_views(
            &instance, &system,
        );

        let reqs = instance.graphics_requirements::<xr::OpenGL>(system).unwrap();

        let session_create_info = util::session_create_info(&gl_things.context, &gl_things.window).unwrap();

        let (session, mut frame_wait, mut frame_stream) = unsafe {
            instance.create_session::<xr::OpenGL>(
                system,
                &session_create_info,
            ).unwrap()
        };

        let (swapchains, images, gl_framebuffers) = Self::create_swapchains(&session, &view_configuration_views);

        let space = Self::create_reference_space(&session);

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

    fn create_swapchains(session: &xr::Session<xr::OpenGL>, views: &Vec<xr::ViewConfigurationView>) -> (Vec<Swapchain<OpenGL>>, Vec<Vec<u32>>, Vec<GLuint>) {
        let swapchain_formats = session.enumerate_swapchain_formats().unwrap();

        let color_swapchain_format = swapchain_formats
            .iter()
            .copied()
            .find(|&f| f == gl::SRGB8_ALPHA8)
            .unwrap_or(swapchain_formats[0]);

        let mut swapchains = vec!();
        let mut images = vec!();

        for view in views {
            let swapchain = session.create_swapchain(&xr::SwapchainCreateInfo::<xr::OpenGL> {
                create_flags: xr::SwapchainCreateFlags::EMPTY,
                usage_flags: xr::SwapchainUsageFlags::SAMPLED | xr::SwapchainUsageFlags::COLOR_ATTACHMENT,
                format: color_swapchain_format,
                sample_count: view.recommended_swapchain_sample_count,
                width: view.recommended_image_rect_width,
                height: view.recommended_image_rect_height,
                face_count: 1,
                array_size: 1,
                mip_count: 1,
            }).unwrap();

            let sp_images = swapchain.enumerate_images().unwrap();

            images.push(sp_images);
            swapchains.push(swapchain);
        }

        let mut gl_framebuffers = vec![];
        for _ in views {
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

        if (!available_extensions.ext_debug_utils || !available_extensions.khr_opengl_enable) {
            if (!available_extensions.ext_debug_utils) {
                panic!("OpenXR Error: Extension EXT_DEBUG_UTIL not available");
            } else {
                panic!("OpenXR Error: Extension KXR_OPENGL_ENABLE not available");
            }
        }

        let layers_str: Vec<&str> = layers.iter().map(|l| l.layer_name.as_str()).collect();

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

    pub fn process_events(&self) {
        let mut data_buffer = xr::EventDataBuffer::new();
        while let Some(event) = self.instance.poll_event(&mut data_buffer).unwrap() {
            match event {
                Event::SessionStateChanged(state_change) => {
                    match state_change.state() {
                        SessionState::READY => {
                            self.session.begin(xr::ViewConfigurationType::PRIMARY_STEREO).unwrap();
                        }
                        _ => todo!(),
                    }
                }
                _ => {}
            }
        }
        //   fun(event);
        //}

    }

    pub fn wait_frame(&mut self) {
        let frame_state = self.frame_wait.wait().unwrap();

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

    pub fn render(&mut self, app: &mut App, glmodel: &TMat4<GLfloat>, glview: &TMat<f32, 4, 4>) {
        for (view_idx, view) in self.view_configuration_views.iter().enumerate() {
            let xr_swapchain_img_idx = self.swapchains[view_idx].acquire_image().unwrap();
            let gl_framebuffer = self.gl_framebuffers[view_idx];
            unsafe {
                gl::BindFramebuffer(gl::FRAMEBUFFER, gl_framebuffer);

                gl::Viewport(0, 0,
                             view.recommended_image_rect_width as i32,
                             view.recommended_image_rect_height as i32);

                gl::Scissor(0, 0,
                            view.recommended_image_rect_width as i32,
                            view.recommended_image_rect_height as i32);
            }

            //let texture = self.images[view_idx][xr_swapchain_img_idx as usize];
            //let texture = unsafe {
            //    gl::CreateTe
            //}

            app.render(&glmodel, &glview);

            self.swapchains[view_idx].release_image();
        }
    }

    pub fn end_frame(&mut self, display_time: xr::Time) {
        if let Some(state) = &self.frame_state {
            self.frame_stream
                .end(
                    state.xr_state.predicted_display_time,
                    xr::EnvironmentBlendMode::OPAQUE,
                    &[&openxr::CompositionLayerProjection::new()
                        .space(&self.reference_space)
                        .views(&[
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
                                                width: self.view_configuration_views[0].recommended_image_rect_width as i32,
                                                height: self.view_configuration_views[0].recommended_image_rect_height as i32,
                                            },
                                        }),
                                ),
                            openxr::CompositionLayerProjectionView::new()
                                .pose(state.views[1].pose)
                                .fov(state.views[1].fov)
                                .sub_image(
                                    openxr::SwapchainSubImage::new()
                                        .swapchain(&self.swapchains[1])
                                        .image_array_index(1)
                                        .image_rect(openxr::Rect2Di {
                                            offset: openxr::Offset2Di { x: 0, y: 0 },
                                            extent: xr::Extent2Di {
                                                width: self.view_configuration_views[1].recommended_image_rect_width as i32,
                                                height: self.view_configuration_views[1].recommended_image_rect_height as i32,
                                            },
                                        }),
                                ),
                        ])],
                )
                .unwrap();
        }
    }
}
