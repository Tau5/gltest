use std::ops::Add;
use gl::types::GLfloat;
use nalgebra_glm as glm;
use nalgebra_glm::TVec3;

pub struct Camera {
    jaw: f32,
    pitch: f32,
    position: glm::Vec3,
    direction: glm::Vec3,
}

impl Camera {
    pub fn new(jaw: f32, pitch: f32, position: glm::Vec3) -> Self {
        let direction = glm::vec3(
            GLfloat::cos(jaw) * GLfloat::cos(pitch),
            GLfloat::sin(pitch),
            GLfloat::sin(jaw) * GLfloat::cos(pitch),
        );

        Camera {
            jaw, pitch, position, direction
        }
    }

    pub fn get_position(&self) -> glm::Vec3 {
       return self.position;
    }
    
    pub fn get_new_position(&self, x: f32, z: f32) -> TVec3<f32> {
        let mut translation = glm::vec3(0.0, 0.0, 0.0);
        translation += glm::normalize(&glm::cross(&self.get_camera_front(), &self.get_camera_up())) * x;
        translation += self.get_camera_front() * z;

        glm::vec3(self.position.x + translation.x, self.position.y, self.position.z + translation.z)
    }

    pub fn translate(&mut self, x: f32, z: f32, y: f32) {
        let mut translation = glm::vec3(0.0, 0.0, 0.0);
        translation += glm::normalize(&glm::cross(&self.get_camera_front(), &self.get_camera_up())) * x;
        translation += self.get_camera_front() * z;

        self.position.x += translation.x;
        self.position.z += translation.z;
        self.position.y += y;
    }

    fn get_camera_right(&self) -> glm::Vec3 {
        glm::normalize(
            &glm::cross(
                &glm::vec3(0.0, 1.0, 0.0),
                &self.get_camera_front()
            )
        )
    }

    pub fn get_camera_up(&self) -> glm::Vec3 {
        glm::normalize(
            &glm::cross(&self.get_camera_front(), &self.get_camera_right())
        )
    }

    pub fn get_camera_front(&self) -> glm::Vec3 {
        glm::normalize(&self.direction)
    }

    pub fn rotate(&mut self, jaw: f32, pitch: f32) {
        if (self.pitch + pitch >= std::f32::consts::PI/2.0 - 0.01) {
            self.pitch = std::f32::consts::PI/2.0 - 0.01
        } else if (self.pitch + pitch <= -std::f32::consts::PI/2.0 + 0.01) {
            self.pitch = -std::f32::consts::PI/2.0 + 0.01
        } else {
            self.pitch += pitch;
        }

        self.jaw += jaw;

        self.update_direction();
    }

    pub fn set_from_angles(&mut self, jaw: f32, pitch: f32) {
        self.jaw = jaw;
        self.pitch = pitch;

        self.update_direction();
    }

    fn update_direction(&mut self) {
        self.direction = glm::vec3(
            GLfloat::cos(self.jaw) * GLfloat::cos(self.pitch),
            GLfloat::sin(self.pitch),
            GLfloat::sin(self.jaw) * GLfloat::cos(self.pitch),
        );
    }

    pub fn set_direction(&mut self, direction: glm::Vec3) {
        self.direction = direction;
    }

    pub fn get_view(&self) -> glm::TMat4<f32> {
        glm::look_at(
            &self.position,
            &(self.position + self.get_camera_front()),
            &self.get_camera_up()
        )
    }
}