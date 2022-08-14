use super::super::electronics::IElectronicController;
use super::super::electronics::Level;
use super::super::electronics::PinPull;
use super::super::electronics::Trigger;
use super::super::timer::Timer;
use super::Weigand;
use color_eyre::eyre::{eyre, Result};
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;
use tracing::{debug, error};

enum Transmission {
    None,
    Payload(u32),
}

pub struct WeigandReader {
    rx: Receiver<u8>,
}

impl WeigandReader {
    pub fn new(
        zero_pin: u8,
        one_pin: u8,
        controller: &mut Box<dyn IElectronicController>,
    ) -> Result<Self> {
        let zero_pin = controller.setup_input_pin(zero_pin, PinPull::PullUp)?;
        let one_pin = controller.setup_input_pin(one_pin, PinPull::PullUp)?;
        let (one_tx, rx) = mpsc::channel::<u8>(200);

        let zero_tx = one_tx.clone();
        let zero_call = move |level: Level| {
            match level {
                Level::Low => match zero_tx.try_send(0) {
                    Ok(_) => {}
                    Err(e) => error!("failed to send low bit: {}", e),
                },
                Level::High => {}
            };
        };

        controller.set_async_interrupt(zero_pin, Trigger::FallingEdge, Box::new(zero_call))?;

        let one_call = move |level: Level| {
            match level {
                Level::Low => match one_tx.try_send(1) {
                    Ok(_) => {}
                    Err(e) => {
                        error!("failed to send high bit: {}", e)
                    }
                },
                Level::High => {}
            };
        };
        controller.set_async_interrupt(one_pin, Trigger::FallingEdge, Box::new(one_call))?;

        Ok(Self { rx })
    }

    pub async fn run(&mut self, channel: tokio::sync::watch::Sender<Weigand>) -> Result<()> {
        debug!("weigand reader beginning to run");
        loop {
            match self.get_payload().await? {
                Transmission::None => { /* Most likely a timeout */ }
                Transmission::Payload(data) => match Weigand::new(data) {
                    Ok(weigand) => match channel.send(weigand) {
                        Ok(_) => {}
                        Err(e) => error!("failed to send wiegand payload: {}", e),
                    },
                    Err(e) => debug!("error getting payload: {}", e),
                },
            }
        }
    }

    async fn get_payload(&mut self) -> Result<Transmission> {
        let max_duration = std::time::Duration::new(0, 500);
        let mut buffer: u32 = 0;
        let mut bit_counter = 0;
        let mut timer = Timer::default();

        while bit_counter < 32 {
            match self.rx.recv().await {
                Some(byte) => {
                    buffer <<= 1;
                    if byte > 0 {
                        buffer |= 1;
                    }
                    bit_counter += 1;
                    timer.reset();
                    debug!("Recv Byte: {}", byte);
                }
                None => {
                    return Err(eyre!("channel closed unexpectadly"));
                }
            }
            if timer.progress(std::time::Instant::now()) > max_duration {
                debug!("timer elapsed on transmission, terminating connection");
                return Ok(Transmission::None);
            }
        }

        debug!(
            "transmission recv ({}), forwarding payload to be processed",
            buffer
        );
        Ok(Transmission::Payload(buffer))
    }
}
