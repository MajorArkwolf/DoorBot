pub mod reader;
use bit_field::BitField;
use color_eyre::eyre::{eyre, Result, WrapErr};
use rfid_debug::WiegandFormat;

#[derive(Debug)]
pub struct Weigand {
    facility_code: u8,
    card_number: u16,
}

impl Weigand {
    pub fn new(data: u32) -> Result<Self> {
        let standard_format = WiegandFormat {
            parity_even: 0,
            parity_odd: 25,
            card_number: (0, 16),    // bit range [lower, upper)
            facility_code: (16, 24), // bit range [lower, upper)
        };

        let facility_code = data.get_bits(16..24) as u8;
        let card_number = data.get_bits(0..16) as u16;

        let parity_even = data.get_bit(0) as u8;
        let parity_odd = data.get_bit(25) as u8;

        Ok(Self {
            facility_code,
            card_number,
        })
    }

    pub fn get_facility_code(&self) -> u8 {
        self.facility_code
    }

    pub fn get_card_number(&self) -> u16 {
        self.card_number
    }
}
