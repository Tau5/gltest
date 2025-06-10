use std::ffi::c_void;
use gl::types::{GLenum, GLint, GLsizei, GLuint};
use image::{DynamicImage, ImageReader, RgbImage, RgbaImage};

type GLTextureID = GLuint;
#[derive(Clone, Copy)]
pub(crate) struct GLTexture {
    kind: GLenum,
    id: GLTextureID
}

impl GLTexture {
    pub fn new(kind: GLenum, id: GLTextureID) -> Self {
        Self { kind, id }
    }
}

pub trait BindableTexture {
    fn bind(&self, _slot: GLenum) {}
}

impl BindableTexture for GLTexture {
    fn bind(&self, slot: GLenum) {
        unsafe {
            gl::ActiveTexture(slot);
            gl::BindTexture(self.kind, self.id);
        }
    }
}

pub fn load_image(path: &str) -> DynamicImage {
    ImageReader::open(path).unwrap()
        .decode().unwrap().flipv()
}

pub fn load_texture(image: DynamicImage) -> GLTexture {
    let mut tex_id: GLuint = 0;

    unsafe {
        gl::GenTextures(1, std::ptr::from_mut(&mut tex_id));
        gl::BindTexture(gl::TEXTURE_2D, tex_id);

        match image {
            DynamicImage::ImageRgba8(rgba) => {
                gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as GLint, rgba.width() as GLsizei, rgba.height() as GLsizei,
                               0, gl::RGBA, gl::UNSIGNED_BYTE, rgba.as_raw().as_slice().as_ptr() as *const c_void);
            }
            img => {
                let rgb = img.into_rgb8();
                gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as GLint, rgb.width() as GLsizei, rgb.height() as GLsizei,
                               0, gl::RGB, gl::UNSIGNED_BYTE, rgb.as_raw().as_slice().as_ptr() as *const c_void);
            }
        }

        gl::GenerateMipmap(gl::TEXTURE_2D);
    }

    GLTexture::new(gl::TEXTURE_2D, tex_id)
}