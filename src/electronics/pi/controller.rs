use crate::electronics::Callback;
use crate::electronics::IElectronicController;
use crate::electronics::Level;
use crate::electronics::PinPull;
use crate::electronics::Trigger;
use color_eyre::eyre::Result;
use rppal::gpio::Gpio;
use rppal::gpio::InputPin;
use rppal::gpio::OutputPin;
use super::InputPinHandle;
use super::OutputPinHandle;
use super::OutputPinWrapper;
use tokio::task::JoinHandle;
use tokio::sync::watch;


pub struct Controller {
    gpio: Gpio,
    input_pins: Vec<InputPin>,
    output_pins: Vec<OutputPinWrapper>,
}

impl Controller {
    pub fn new() -> Result<Self> {
        Ok(Self {
            gpio: Gpio::new()?,
            input_pins: vec![],
            output_pins: Vec<OutputPinWrapper>,
        })
    }
}

impl IElectronicController for Controller {
    fn setup_input_pin(&mut self, pin_num: u8, pin_pull: PinPull) -> Result<InputPinHandle> {
        let input_pin = match pin_pull {
            PinPull::PullUp => self.gpio.get(pin_num)?.into_input_pullup(),
            PinPull::PullDown => self.gpio.get(pin_num)?.into_input_pulldown(),
            PinPull::None => self.gpio.get(pin_num)?.into_input(),
        };

        self.input_pins.push(input_pin);

        Ok(InputPinHandle::new(self.input_pins.len() - 1))
    }

    fn set_async_interrupt(
        &mut self,
        pin_handle: InputPinHandle,
        trigger: Trigger,
        mut callback: Callback,
    ) -> Result<()> {
        let pi_trigger = match trigger {
            Trigger::RisingEdge => rppal::gpio::Trigger::RisingEdge,
            Trigger::FallingEdge => rppal::gpio::Trigger::FallingEdge,
            Trigger::Both => rppal::gpio::Trigger::Both,
        };

        let pi_callback = move |level: rppal::gpio::Level| {
            let level = match level {
                rppal::gpio::Level::Low => Level::Low,
                rppal::gpio::Level::High => Level::High,
            };
            callback(level);
        };

        self.input_pins[pin_handle.get_id()].set_async_interrupt(pi_trigger, pi_callback)?;
        Ok(())
    }

    fn setup_output_pin(&mut self, pin_num: u8) -> Result<OutputPinHandle> {
        let output_pin = self.gpio.get(pin_num)?.into_output();
        let (tx, mut rx) = watch::channel(Level::Low);
        let task: JoinHandle<Result<()>> = tokio::task::spawn(async move {
            let pin_num = pin_num;
            loop {
                if rx.has_changed()? {
                    let set_pin_high = rx.borrow_and_update();
                    match *set_pin_high {
                        Level::High => output_pin.set_high(),
                        Level::Low => output_pin.set_low(),
                    }
                }
            }
        });
        let pin_wrapper = OutputPinWrapper::new(task);

        self.output_pins.push(pin_wrapper);
        Ok(OutputPinHandle::new(self.output_pins.len() - 1, tx))
    }
}
