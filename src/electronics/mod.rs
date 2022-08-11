pub mod mock;
#[cfg(target_os = "arm")]
pub mod pi;
use color_eyre::eyre::Result;

pub enum Level {
    High,
    Low,
}

pub enum Trigger {
    RisingEdge,
    FallingEdge,
    Both,
}

pub enum PinPull {
    PullUp,
    PullDown,
    None,
}

pub struct PinHandle {
    pin_id: usize,
}

impl PinHandle {
    pub fn new(pin_id: usize) -> Self {
        Self { pin_id }
    }

    pub fn get_id(self) -> usize {
        self.pin_id
    }
}

type Callback = Box<dyn FnMut(Level) + Send>;

pub trait IElectronicController {
    fn setup_input_pin(&mut self, pin_num: u8, pin_pull: PinPull) -> Result<PinHandle>;
    fn set_async_interrupt(
        &mut self,
        pin_handle: PinHandle,
        trigger: Trigger,
        callback: Callback,
    ) -> Result<()>;
}
