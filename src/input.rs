use std::collections::HashMap;
use std::os::linux::raw::stat;
use std::os::raw::c_int;
use winit::event::{ElementState, KeyEvent};
use winit::keyboard::Key;
use winit::window::Window;

#[derive(Eq, PartialEq, Copy, Clone, PartialOrd, Debug)]
pub enum KeyStatus {
    NoInput = 0,
    Released = 1,
    JustPressed = 2,
    Hold = 3,
}

pub struct InputManager {
    keys: HashMap<Key, KeyStatus>,
}

impl InputManager {
    pub(crate) fn winit_key_event(&mut self, event: KeyEvent) {
        let status = match event.state {
            ElementState::Pressed => KeyStatus::JustPressed,
            ElementState::Released => KeyStatus::Released,
        };

        let key = self.keys.insert(event.logical_key, status);
    }
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
        }
    }

    pub fn add_key(&mut self, key: Key) {
        self.keys.insert(key, KeyStatus::NoInput);
    }

    pub fn update_keys(&mut self) {
        for (key, status) in self.keys.iter_mut() {
            let updated_status = match status {
                KeyStatus::NoInput => KeyStatus::NoInput,
                KeyStatus::Released => KeyStatus::NoInput,
                KeyStatus::JustPressed => KeyStatus::Hold,
                KeyStatus::Hold => KeyStatus::Hold,
            };

            *status = updated_status;
        }
    }

    pub fn get_status(&self, key: Key) -> KeyStatus {
        if let Some(status) = self.keys.get(&key) {
            *status
        } else {
            KeyStatus::NoInput
        }
    }
}
