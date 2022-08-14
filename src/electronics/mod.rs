pub mod mock;

#[cfg(target_arch = "aarch64")]
pub mod pi;
use color_eyre::eyre::{eyre, Context, Result};
use tokio::task::JoinHandle;

#[derive(Debug)]
pub enum Level {
    High,
    Low,
}

#[derive(Debug)]
pub enum Trigger {
    RisingEdge,
    FallingEdge,
    Both,
}

#[derive(Debug)]
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
#[derive(Debug)]
pub struct OutputPinHandle {
    pin_id: usize,
    pin_channel: tokio::sync::watch::Sender<Level>,
}

impl OutputPinHandle {
    pub fn new(pin_id: usize, pin_channel: tokio::sync::watch::Sender<Level>) -> Self {
        Self {
            pin_id,
            pin_channel,
        }
    }

    pub fn get_id(self) -> usize {
        self.pin_id
    }

    pub async fn set_pin_state(&mut self, level: Level) -> Result<()> {
        self.pin_channel
            .send(level)
            .wrap_err_with(|| eyre!("failed to send output pin state: {:?}", self))
    }
}

struct OutputPinWrapper {
    _background_task: JoinHandle<Result<()>>,
}

impl OutputPinWrapper {
    fn new(_background_task: JoinHandle<Result<()>>) -> Self {
        Self { _background_task }
    }
}

type Callback = Box<dyn FnMut(Level) + Send>;

pub trait IElectronicController {
    fn setup_input_pin(&mut self, pin_num: u8, pin_pull: PinPull) -> Result<InputPinHandle>;
    fn setup_output_pin(&mut self, pin_num: u8) -> Result<OutputPinHandle>;
    fn set_async_interrupt(
        &mut self,
        pin_handle: InputPinHandle,
        trigger: Trigger,
        callback: Callback,
    ) -> Result<()>;
}
