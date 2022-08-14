use std::sync::Arc;
use std::time::Duration;

use crate::electronics::Callback;
use crate::electronics::IElectronicController;
use crate::electronics::InputPinHandle;
use crate::electronics::PinPull;
use crate::electronics::Trigger;
use bit_field::BitField;
use color_eyre::eyre::Result;
use tokio::sync::watch;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tracing::debug;

use super::Level;
use super::OutputPinHandle;
use super::OutputPinWrapper;

pub struct Controller {
    input_pins: Vec<u8>,
    output_pins: Vec<OutputPinWrapper>,
    call_back: Arc<Mutex<Vec<Callback>>>,
    _background_task: JoinHandle<()>,
}

impl Controller {
    pub fn new() -> Result<Self> {
        let call_back: Arc<Mutex<Vec<Callback>>> = Arc::new(Mutex::new(vec![]));
        let background_callback = call_back.clone();
        let _background_task = tokio::task::spawn(async move {
            let test_code: u32 = 2802361858; // From rfid_converter_tests.py
            loop {
                let mut x = background_callback.lock().await;

                if x.len() > 1 {
                    for i in 0usize..32usize {
                        let bit = test_code.get_bit(31 - i);
                        debug!("Sent Byte: {}", bit as u8);
                        if !bit {
                            x[0](Level::Low);
                            x[1](Level::High);
                        } else {
                            x[0](Level::High);
                            x[1](Level::Low);
                        }
                        tokio::time::sleep(Duration::new(0, 6)).await;
                    }
                    debug!("mock payload sent, sleeping");
                }
                tokio::time::sleep(Duration::new(10, 0)).await;
            }
        });

        Ok(Self {
            input_pins: vec![],
            output_pins: vec![],
            call_back,
            _background_task,
        })
    }
}

impl IElectronicController for Controller {
    fn setup_input_pin(&mut self, pin_num: u8, _pin_pull: PinPull) -> Result<InputPinHandle> {
        self.input_pins.push(pin_num);

        Ok(InputPinHandle::new(self.input_pins.len() - 1))
    }

    fn set_async_interrupt(
        &mut self,
        _pin_handle: InputPinHandle,
        _trigger: Trigger,
        callback: Callback,
    ) -> Result<()> {
        let arc_mutex = self.call_back.clone();
        tokio::task::spawn(async move {
            let mut mutex = arc_mutex.lock().await;
            mutex.push(callback);
        });

        std::thread::sleep(std::time::Duration::new(2, 0)); // Since we need to do an async execution we do a wait to ensure operation completes.

        Ok(())
    }

    fn setup_output_pin(&mut self, pin_num: u8) -> Result<OutputPinHandle> {
        let (tx, mut rx) = watch::channel(Level::Low);
        let task: JoinHandle<Result<()>> = tokio::task::spawn(async move {
            let pin_num = pin_num;
            loop {
                if rx.has_changed()? {
                    let set_pin_high = rx.borrow_and_update();
                    match *set_pin_high {
                        Level::High => debug!("mock pin {}, set high", pin_num),
                        Level::Low => debug!("mock pin {}, set low", pin_num),
                    }
                }
            }
        });
        let pin_wrapper = OutputPinWrapper::new(task);

        self.output_pins.push(pin_wrapper);
        Ok(OutputPinHandle::new(self.output_pins.len() - 1, tx))
    }
}
