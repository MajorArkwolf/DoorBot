pub mod reader;
use bit_field::BitField;
use color_eyre::eyre::{eyre, Result};

#[derive(Debug)]
pub struct Weigand {
    facility_code: u16,
    card_number: u32,
}

impl Weigand {
    pub fn new(data: u32) -> Result<Self> {
        let parity_even = bit_field::BitField::get_bit(&data, 0) as bool;
        let parity_odd = bit_field::BitField::get_bit(&data, 31) as bool;

        let even_calc_bit = (Weigand::count_ones(data, 1, 17) % 2) == 0;
        let odd_calc_bit = (Weigand::count_ones(data, 18, 30) % 2) == 1;

        if parity_even != even_calc_bit {
            return Err(eyre!(
                "odd parity bit was incorrect, Expected: {}, Calculated: {}",
                parity_even,
                even_calc_bit,
            ));
        }

        if parity_odd != odd_calc_bit {
            return Err(eyre!(
                "odd parity bit was incorrect, Expected: {}, Calculated: {}",
                parity_odd,
                odd_calc_bit,
            ));
        }
        Ok(Weigand::new_unchecked(data))
    }

    pub fn new_unchecked(data: u32) -> Self {
        let facility_code = data.get_bits(1..8) as u16;
        let card_number = data.get_bits(9..30) as u32;

        Self {
            facility_code,
            card_number,
        }
    }

    fn count_ones(data: u32, start_index: usize, end_index: usize) -> usize {
        let mut counter: usize = 0;
        for i in start_index..end_index {
            if data.get_bit(i) {
                counter += 1;
            }
        }
        counter
    }

    pub fn get_facility_code(&self) -> u16 {
        self.facility_code
    }

    pub fn get_card_number(&self) -> u32 {
        self.card_number
    }
}
