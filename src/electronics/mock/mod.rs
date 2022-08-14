use std::sync::Arc;
use std::time::Duration;

use crate::electronics::Callback;
use crate::electronics::IElectronicController;
use crate::electronics::InputPinHandle;
use crate::electronics::PinPull;
use crate::electronics::Trigger;
use bit_field::BitField;
use color_eyre::eyre::Result;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tracing::debug;

use super::Level;
use super::OutputPinHandle;

pub struct Controller {
    input_pins: Vec<u8>,
    output_pins: Vec<u8>,
    call_back: Arc<Mutex<Vec<Callback>>>,
    _background_task: JoinHandle<()>,
}

impl Controller {
    pub fn new() -> Result<Self> {
        let call_back: Arc<Mutex<Vec<Callback>>> = Arc::new(Mutex::new(vec![]));
        let background_callback = call_back.clone();
        //let test_code: u32 = 13175734; // Site code: 201 Card Number: 02998
        let test_code: u32 = 2802361858; // From rfid_converter_tests.py
        let _background_task = tokio::task::spawn(async move {
            loop {
                let mut x = background_callback.lock().await;

                if x.len() > 1 {
                    // Send code 13175734 or 110010010000101110110110
                    for i in 0usize..32usize {
                        let bit = test_code.get_bit(31 - i);
                        if !bit {
                            x[0](Level::Low);
                            x[1](Level::High);
                        } else {
                            x[0](Level::High);
                            x[1](Level::Low);
                        }
                        tokio::time::sleep(Duration::new(0, 5)).await;
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

        Ok(())
    }

    fn setup_output_pin(&mut self, pin_num: u8) -> Result<OutputPinHandle> {
        self.output_pins.push(pin_num);
        Ok(OutputPinHandle::new(self.output_pins.len() - 1))
    }
}
