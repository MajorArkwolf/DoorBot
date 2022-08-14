use crate::electronics::OutputPinHandle;

use tracing::{debug};
#[derive(Debug, Clone)]
pub struct Door {
    relay_pin: OutputPinHandle,
    door_id: u32,
}

impl Door {
    pub fn new(relay_pin: OutputPinHandle, door_id: u32) -> Self {
        Self{relay_pin, door_id}
    }

    pub fn get_door_id(&self) -> u32 {
        self.door_id
    }

    pub fn open_door(&self) {
        debug!("Unlocking door {}", self.door_id);
    }

    pub fn close_door(&self) {
        debug!("Locking door {}", self.door_id);
    }
}