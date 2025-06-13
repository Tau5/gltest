use std::iter;
use std::slice::{Iter, IterMut};
use gl::types::GLint;
use thiserror::Error;
use crate::lighting::{DirLight, PointLight, SpotLight};
use crate::shader::ShaderProgram;

pub struct LightManager {
    light_points: Vec<PointLight>,
    max_lights: usize,
    pub spot_light: Option<SpotLight>,
    pub directional_light: DirLight
}

#[derive(Debug, Error)]
pub enum LightManagerError {
    #[error("exceeded max light count")]
    ExceedMaxLightCount
}

impl LightManager {
    
    pub fn iter(&self) -> Iter<'_, PointLight> {
        self.light_points.iter()
    }
    
    pub fn iter_mut(&mut self) -> IterMut<'_, PointLight> {
        self.light_points.iter_mut()
    }
    pub fn add_lightpoint(&mut self, light: PointLight) -> Result<usize, LightManagerError> {
        if self.light_points.len() >= self.max_lights {
            Err(LightManagerError::ExceedMaxLightCount)
        } else {
            self.light_points.push(light);
            Ok(self.light_points.len())
        }
    }

    pub fn get_pointlight(&self, index: usize) -> Option<&PointLight> {
        self.light_points.get(index)
    }

    pub fn get_mut_pointlight(&mut self, index: usize) -> Option<&mut PointLight> {
        self.light_points.get_mut(index)
    }

    pub fn load_lights(&self, shader: &ShaderProgram) {
        shader.setInt(c"lightpoint_num", self.light_points.len() as GLint);
        for (i, light) in self.light_points.iter().enumerate() {
            light.load(i, shader);
        }

        if let Some(spotlight) = &self.spot_light {
            spotlight.load(shader);
        }

        self.directional_light.load(shader);
    }

    pub fn new(max_lights: usize, directional_light: DirLight, spot_light: Option<SpotLight>) -> Self {
        let light_points = Vec::with_capacity(max_lights);
        Self { light_points, max_lights, spot_light, directional_light }
    }
}