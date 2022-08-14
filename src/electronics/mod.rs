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
#[derive(Debug, Clone)]
pub struct InputPinHandle {
    pin_id: usize,
}

impl InputPinHandle {
    pub fn new(pin_id: usize) -> Self {
        Self { pin_id }
    }

    pub fn get_id(self) -> usize {
        self.pin_id
    }
}
#[derive(Debug, Clone)]
pub struct OutputPinHandle {
    pin_id: usize,
}

impl OutputPinHandle {
    pub fn new(pin_id: usize) -> Self {
        Self { pin_id }
    }

    pub fn get_id(self) -> usize {
        self.pin_id
    }
}

type Callback = Box<dyn FnMut(Level) + Send>;

pub trait IElectronicController {
    fn setup_input_pin(&mut self, pin_num: u8, pin_pull: PinPull) -> Result<InputPinHandle>;
    fn setup_output_pin(&mut self, pin_num: u8) -> Result<OutputPinHandle> ;
    fn set_async_interrupt(
        &mut self,
        pin_handle: InputPinHandle,
        trigger: Trigger,
        callback: Callback,
    ) -> Result<()>;
}
