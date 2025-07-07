use crate::app::GlThings;
use crate::lighting::LightManager;
use crate::prefabs;
use crate::shader::ShaderProgram;
use crate::textures::MaterialStore;
use crate::vao::{TriangleArrayVAO, VAO};
use crate::world::World;
use gl::types::GLfloat;
use glutin::prelude::GlSurface;
use nalgebra_glm as glm;
use nalgebra_glm::{proj, TMat4, Vec3};

pub struct RendererConfig {
    pub selected_obj: usize
}

#[derive(Clone, Copy)]
pub struct CameraRenderInfo {
    pub position: Vec3,
    pub front: Vec3
}

pub struct Renderer {
    lighting_shader: ShaderProgram,
    lightpoint_shader: ShaderProgram,
    aabb_shader: ShaderProgram,
    border_shader: ShaderProgram,
    lamp_vao: TriangleArrayVAO,
    pub material_store: MaterialStore,
    pub gl_things: GlThings,
    pub light_manager: LightManager,
}

impl Renderer {
    pub fn new(width: f32, height: f32, material_store: MaterialStore, gl_things: GlThings, light_manager: LightManager) -> Self {
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


        Self {
            lighting_shader,
            lightpoint_shader,
            aabb_shader,
            border_shader,
            lamp_vao: prefabs::cube(),
            material_store,
            gl_things,
            light_manager,
        }
    }
    pub fn render(&mut self, world: &mut World, view: &TMat4<GLfloat>, model: &TMat4<GLfloat>, renderer_config: &RendererConfig, camera_render_info: CameraRenderInfo, proj: &TMat4<GLfloat>) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::Enable(gl::DEPTH_TEST);
        }

        // mat3(transpose(inverse(model)))

        self.lightpoint_shader.load();
        self.lightpoint_shader.setMat4(c"view", &view);
        self.lightpoint_shader.setMat4(c"projection", &proj);

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

        self.load_lighting_shader(&view, camera_render_info, &proj);

        //unsafe {
        //    gl::StencilMask(0x00);
        //}
        // Render all objects
        for (i, obj) in world.objects.iter().enumerate() {
            obj.render(&self.lighting_shader, &self.material_store);
        }

        //unsafe {
        //    // If depth and stencil tests pass, replace value in stencil
        //    // with the ref value in StencilFunc.
        //    // If any fail, don't change stencil buffer for that fragment
        //    gl::StencilOp(gl::KEEP, gl::KEEP, gl::REPLACE);

        //    // Stencil test ALWAYS passes. Ref value is 1 and because we used
        //    // REPLACE in StencilOp, we'll change the stencil value for each fragment
        //    // to 1
        //    gl::StencilFunc(gl::ALWAYS, 1, 0xFF);

        //    // Enable writing to the stencil buffer
        //    gl::StencilMask(0xFF);
        //}

        // Render all objects normally except selected

        //if let Some(obj) = world.objects.get(renderer_config.selected_obj) {
        //    obj.render(&self.lighting_shader, &self.material_store);
        //}

        //unsafe {
        //    // Stencil test passes if the fragment wasn't written to
        //    gl::StencilFunc(gl::NOTEQUAL, 1, 0xFF);

        //    // Do NOT write to the stencil buffer
        //    gl::StencilMask(0x00);

        //    // We don't wan't the depth test involved in this
        //    //gl::Disable(gl::DEPTH_TEST);
        //}

        //self.border_shader.load();
        //self.border_shader.setMat4(c"view", &view);
        //self.border_shader.setMat4(c"projection", &proj);

        //if let Some(obj) = world.objects.get_mut(renderer_config.selected_obj) {
        //    obj.render_scaled(&self.border_shader, &self.material_store);
        //}

        //unsafe {
        //    gl::StencilMask(0xFF);
        //    gl::StencilFunc(gl::ALWAYS, 1, 0xFF);
        //}

        //self.aabb_shader.load();
        //self.aabb_shader.setMat4(c"view", &view);
        //self.aabb_shader.setMat4(c"projection", &self.proj);

        //if let Some(obj) = self.objects.get(self.selected_obj) {
        //    obj.debug_render_aabb(&self.aabb_shader);
        //}


    }

    fn load_lighting_shader(&mut self, view: &TMat4<GLfloat>, camera: CameraRenderInfo, proj: &TMat4<GLfloat>) {
        self.lighting_shader.load();
        self.lighting_shader.setMat4(c"view", &view);
        self.lighting_shader.setMat4(c"projection", &proj);
        self.lighting_shader
            .setVec3(c"viewPos", &camera.position);

        if let Some(spot_light) = &mut self.light_manager.spot_light {
            spot_light.position = camera.position;
            spot_light.direction = camera.front;
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