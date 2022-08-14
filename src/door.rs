use crate::electronics::{Level, OutputPinHandle};
use color_eyre::eyre::Result;
use std::sync::Arc;
use tokio::task::JoinHandle;

use tokio::{sync::Notify, time};
use tracing::debug;
#[derive(Debug)]
pub struct Door {
    door_id: u32,
    notify_tx: Arc<Notify>,
    _background_task: JoinHandle<Result<()>>,
}

impl Door {
    pub fn new(mut relay_pin: OutputPinHandle, door_id: u32, time_out: time::Duration) -> Self {
        let notify_rx = Arc::new(Notify::new());
        let notify_tx = notify_rx.clone();

        let _background_task = tokio::task::spawn(async move {
            let mut is_door_locked = true;
            let door_id = door_id;
            // Since a notify resets the sleep, we do not need to restart the timer.
            loop {
                tokio::select! {
                    _ = notify_rx.notified() => {
                        if is_door_locked {
                            debug!("Unlocking door {}", door_id);
                            relay_pin.set_pin_state(Level::High).await?;
                            is_door_locked = false;
                        }
                    }
                    _ = tokio::time::sleep(time_out) => {
                        if !is_door_locked {
                            debug!("Locking door {}", door_id);
                            relay_pin.set_pin_state(Level::Low).await?;
                            is_door_locked = true;
                        }
                    }
                }
            }
        });

        Self {
            door_id,
            notify_tx,
            _background_task,
        }
    }

    pub fn get_door_id(&self) -> u32 {
        self.door_id
    }

    pub fn open_door(&self) {
        self.notify_tx.notify_one();
    }
}
