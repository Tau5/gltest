use crate::cube::Cube;
use crate::model::Model;
use crate::object::Object;
use crate::textures::MaterialStore;
use nalgebra_glm as glm;
use crate::lighting::LightManager;

pub struct World {
    pub objects: Vec<Object>,
}

impl World {
    pub fn new(material_store: &mut MaterialStore) -> Self {
        Self {
            objects: Self::generate_objects(material_store),
        }
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
}

