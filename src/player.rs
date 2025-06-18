use gl::types::GLfloat;
use crate::aabb::AABB;
use crate::camera::Camera;
use nalgebra_glm as glm;
use nalgebra_glm::{scale, translate, vec3, TVec3, Vec3};
use crate::collidable::Collidable;
use crate::object::Object;

pub struct Player {
    camera: Camera,
    scale: Vec3,
    pub aabb: AABB,
}

impl Player {
    pub(crate) fn get_front(&self) -> TVec3<GLfloat> {
        self.camera.get_camera_front()
    }
}

impl Player {
    pub(crate) fn get_position(&self) -> Vec3 {
        self.camera.get_position()
    }
}

impl Player {
    pub fn new(camera: Camera) -> Self {
        // FIXME: Posiblemente no funcione porque la camara está en el centro, no abajo a la izquierda

        let scale = glm::vec3(1.0, 4.5, 1.0);

        let aabb = AABB::new(
            -(scale/2.0),
            scale/2.0
        ).translate(camera.get_position() + vec3(0.0, -2.0, 0.0));

        Player {
            camera, aabb, scale
        }
    }

    pub fn rotate_camera(&mut self, jaw: f32, pitch: f32) {
        self.camera.rotate(jaw, pitch);
    }

    pub fn get_view(&self) -> glm::TMat4<GLfloat> {
        self.camera.get_view()
    }

    pub fn translate(&mut self, x: GLfloat, z: GLfloat, collidable_objects: &Vec<Object>) {
        // TODO: Implementar BSP para no calcular colisiones con todos los objetos del mapa
        
        let relative_new_pos = self.camera.get_new_position(x, z);
        let new_aabb = AABB::new(
            -(self.scale/2.0),
            self.scale/2.0
        ).translate(relative_new_pos + vec3(0.0, -2.0, 0.0));

        let collides = collidable_objects.iter().any(|f| {
            f.collides(&new_aabb)
        });

        if (!collides) {
            self.aabb = new_aabb;
            self.camera.translate(x, z, 0.0);
        }

    }
    
    pub fn apply_gravity(&mut self, y: GLfloat, collidable_objects: &Vec<Object>) {
        
        let mut new_aabb = self.aabb.translate(
            vec3(0.0, y, 0.0)
        );

        let collides = collidable_objects.iter().any(|f| {
            f.collides(&new_aabb)
        });
        
        if (!collides) {
            self.aabb = new_aabb;
            self.camera.translate(0.0, 0.0, y);
        }
    }
}