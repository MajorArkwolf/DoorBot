use crate::weigand::Weigand;
use bit_field::BitField;
use byteorder::BigEndian;
use byteorder::ByteOrder;
use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::DisplayFromStr;
use std::collections::HashMap;
use std::num::ParseIntError;
use tracing::debug;

#[derive(Serialize, Deserialize, Debug)]
pub struct KeyConfig {
    #[serde(default)]
    pub remote: String,
    pub local: String,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Key {
    pub name: String,
    pub groups: Vec<String>,
    #[serde(rename = "door")]
    pub door_id: u32,
}

impl Key {
    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_door_id(&self) -> u32 {
        self.door_id
    }
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct Keys {
    #[serde_as(as = "HashMap<DisplayFromStr, _>")]
    keys: Vec<(String, Key)>,
}

impl Keys {
    pub fn get_keys(&self) -> Result<HashMap<u64, Key>> {
        let mut keys: HashMap<u64, Key> = HashMap::new();
        for key in &self.keys {
            keys.insert(old_door_pi_hex_str_to_rfid(&key.0)?, key.1.clone());
        }
        debug!("{:?}", keys);
        Ok(keys)
    }
}

pub fn weigand_to_key(weigand: &Weigand) -> u64 {
    let mut key_id: u64 = weigand.get_card_number().into();
    //key_id.set_bits(30..32, weigand.get_facility_code().get_bits(0..1).into());
    key_id <<= 2;
    key_id.set_bit(0, weigand.get_facility_code().get_bit(0));
    key_id.set_bit(1, weigand.get_facility_code().get_bit(1));
    key_id
}

fn old_door_pi_hex_str_to_rfid(hex_str: &str) -> Result<u64> {
    // Converts the hex string from old doorpi arduino to the
    // decimal string printed on keyfobs and also what USB RFID
    // receiver outputs.

    // Chop off the first 4 hex chars (2 bytes). These two dont
    // appear in the hex conversion of the decimal value written
    // on keyfobs and output from USB RFID reader.
    let bytes = decode_hex(&hex_str[2..])?;
    let rfid_value: u64 = BigEndian::read_u32(&bytes).into();
    Ok(rfid_value)
}

fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}
