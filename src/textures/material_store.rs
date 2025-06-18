use crate::textures::material_store::MaterialStoreError::TextureLoadError;
use crate::textures::{GLTexture, Material, TextureSource};
use crate::textures::{load_image, load_texture};
use std::collections::HashMap;
use gl::types::GLfloat;
use thiserror::Error;
use crate::textures;

#[derive(Error, Debug)]
pub enum MaterialStoreError {
    #[error("couldn't load required texture")]
    TextureLoadError(String),
    #[error("id already exists")]
    IdAlreadyUsed,
}

pub struct MaterialStore {
    store: HashMap<String, Material>,
    fallback: Material,
}

impl MaterialStore {
    pub fn new(default_texture_path: String) -> Self {
        Self {
            store: HashMap::new(),
            fallback: Material {
                diffuse_map: Some(
                    MaterialStore::load_texture(TextureSource::ImagePath(default_texture_path))
                        .expect("Error loading fallback texture for MaterialStore"),
                ),
                specular_map: None,
                emission_map: None,
                factor: 4.0,
            },
        }
    }
    fn load_texture(source: TextureSource) -> Result<GLTexture, MaterialStoreError> {
        match source {
            TextureSource::ImagePath(path) => {
                if let Ok(img) = load_image(path.as_str()) {
                    Ok(load_texture(img))
                } else {
                    Err(TextureLoadError(path))
                }        
            }
            TextureSource::BaseColor(color) => {
                Ok(textures::generate_texture_from_color(color))
            }
        }
        
    }

    pub fn load(
        &mut self,
        id: &str,
        diffuse_path: Option<TextureSource>,
        specular_path: Option<TextureSource>,
        emission_path: Option<TextureSource>,
        factor: GLfloat
    ) -> Result<String, MaterialStoreError> {
        if self.store.contains_key(&id.to_string()) {
            return Err(MaterialStoreError::IdAlreadyUsed);
        }

        let mut mat = Material {
            diffuse_map: None,
            specular_map: None,
            emission_map: None,
            factor
        };

        if let Some(path) = diffuse_path {
            mat.diffuse_map = Some(MaterialStore::load_texture(path)?);
        }
        if let Some(path) = specular_path {
            mat.specular_map = Some(MaterialStore::load_texture(path)?);
        }
        if let Some(path) = emission_path {
            mat.emission_map = Some(MaterialStore::load_texture(path)?);
        }

        self.store.insert(id.to_string(), mat);

        Ok(id.to_string())
    }

    pub fn get(&self, id: &str) -> Material {
        self.store.get(id).copied().unwrap_or(self.fallback)
    }

    pub fn unload(&mut self, id: &String) {
        let material = self.get(id);
        if let Some(tex) = material.diffuse_map {
            let texes = [tex.id];
            unsafe {
                gl::DeleteTextures(1, texes.as_ptr());
            }
        }
        if let Some(tex) = material.emission_map {
            let texes = [tex.id];
            unsafe {
                gl::DeleteTextures(1, texes.as_ptr());
            }
        }
        if let Some(tex) = material.specular_map {
            let texes = [tex.id];
            unsafe {
                gl::DeleteTextures(1, texes.as_ptr());
            }
        }

        self.store.remove(id).unwrap();
    }
}
