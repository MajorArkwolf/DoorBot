use crate::electronics::Callback;
use crate::electronics::IElectronicController;
use crate::electronics::Level;
use crate::electronics::PinHandle;
use crate::electronics::PinPull;
use crate::electronics::Trigger;
use color_eyre::eyre::Result;
use rppal::gpio::Gpio;
use rppal::gpio::InputPin;
use rppal::gpio::OutputPin;

pub struct Controller {
    gpio: Gpio,
    input_pins: Vec<InputPin>,
}

impl Controller {
    pub fn new() -> Result<Self> {
        Ok(Self {
            gpio: Gpio::new()?,
            input_pins: vec![],
        })
    }

    pub fn setup_output_pin(&mut self, pin_num: u8) -> Result<OutputPin> {
        Ok(self.gpio.get(pin_num)?.into_output())
    }
}

impl IElectronicController for Controller {
    fn setup_input_pin(&mut self, pin_num: u8, pin_pull: PinPull) -> Result<PinHandle> {
        let input_pin = match pin_pull {
            PinPull::PullUp => self.gpio.get(pin_num)?.into_input_pullup(),
            PinPull::PullDown => self.gpio.get(pin_num)?.into_input_pulldown(),
            PinPull::None => self.gpio.get(pin_num)?.into_input(),
        };

        self.input_pins.push(input_pin);

        Ok(PinHandle::new(self.input_pins.len() - 1))
    }

    fn set_async_interrupt(
        &mut self,
        pin_handle: PinHandle,
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
}
