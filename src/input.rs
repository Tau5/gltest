use std::collections::HashMap;
use std::os::linux::raw::stat;
use std::os::raw::c_int;
use glfw::ffi::{glfwGetKey, GLFWwindow};
use glfw::Key;

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum KeyStatus {
    NoInput = 0,
    Released = 1,
    JustPressed = 2,
    Hold = 3
}

pub struct InputManager {
    keys: HashMap<Key, KeyStatus>
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            keys: HashMap::new()
        }
    }
    
    pub fn add_key(&mut self, key: Key) {
       self.keys.insert(key, KeyStatus::NoInput); 
    }
    
    pub fn poll(&mut self, window: *mut GLFWwindow) {
        for (key, status) in self.keys.iter_mut() {
            unsafe { 
                let glfw_status = glfwGetKey(window, *key as c_int);
                
                match glfw_status {
                    glfw::ffi::PRESS => {
                        if *status == KeyStatus::NoInput {
                            *status = KeyStatus::JustPressed;
                        } else {
                            *status = KeyStatus::Hold;
                        }
                    }
                    
                    glfw::ffi::RELEASE => {
                        if *status == KeyStatus::Hold {
                            *status = KeyStatus::Released;
                        } else {
                            *status = KeyStatus::NoInput;
                        }
                    }
                   _ => {
                       *status = KeyStatus::NoInput;
                   }
                }
            }
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