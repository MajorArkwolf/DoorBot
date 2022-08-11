use std::sync::Arc;
use std::time::Duration;

use crate::electronics::Callback;
use crate::electronics::IElectronicController;
use crate::electronics::PinHandle;
use crate::electronics::PinPull;
use crate::electronics::Trigger;
use bitvec::prelude::*;
use color_eyre::eyre::Result;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tracing::debug;

use super::Level;

pub struct Controller {
    input_pins: Vec<u8>,
    call_back: Arc<Mutex<Vec<Callback>>>,
    background_task: JoinHandle<()>,
}

impl Controller {
    pub fn new() -> Result<Self> {
        let call_back: Arc<Mutex<Vec<Callback>>> = Arc::new(Mutex::new(vec![]));
        let background_callback = call_back.clone();
        let test_code: u32 = 13175734; // Site code: 201 Card Number: 02998
        let background_task = tokio::task::spawn(async move {
            loop {
                let mut x = background_callback.lock().await;
                if x.len() > 1 {
                    // Send code 13175734 or 110010010000101110110110
                    for i in test_code.view_bits::<Msb0>() {
                        if !*i {
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
            call_back,
            background_task,
        })
    }
}

impl IElectronicController for Controller {
    fn setup_input_pin(&mut self, pin_num: u8, _pin_pull: PinPull) -> Result<PinHandle> {
        self.input_pins.push(pin_num);

        Ok(PinHandle::new(self.input_pins.len() - 1))
    }

    fn set_async_interrupt(
        &mut self,
        _pin_handle: PinHandle,
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
}
