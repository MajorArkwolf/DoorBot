use crate::{weigand::Weigand, door::Door};

use super::keys::{weigand_to_key, Key, KeyConfig, Keys};
use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use tracing::{debug, info};

pub enum DoorAuthResponse {
    Success,
    UserNotAuthorised,
    DoorIdNotFound,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonFile {
    pub enclave: EnclaveLoader,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EnclaveLoader {
    #[serde(rename = "config")]
    config: EnclaveConfig,
    #[serde(rename = "keys")]
    key_config: KeyConfig,
    #[serde(rename = "auth")]
    auth_config: AuthConfig,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EnclaveConfig {
    #[serde(default)]
    pub remote: String,
    pub local: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthConfig {
    #[serde(default)]
    pub is_enabled: bool,
    #[serde(default)]
    pub local: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthCredential {
    #[serde(rename = "enclave_http_username")]
    pub username: String,
    #[serde(rename = "enclave_http_password")]
    pub password: String,
}

#[derive(Debug)]

pub struct Enclave {
    credentials: Option<AuthCredential>,
    keys: HashMap<u64, Key>,
    doors: Vec<Door>,
}

impl Enclave {
    pub fn load() -> Result<Self> {
        let file = File::open("./config/main_config.json")?;
        let config: JsonFile = serde_json::from_reader(file)?;
        let config = config.enclave;

        let credentials = match config.auth_config.is_enabled {
            true => {
                // Load Auth Credentials
                let cred: AuthCredential =
                    serde_json::from_reader(File::open("./config/enclave_auth.json")?)?;
                Enclave::update_files_from_cred(&cred)?;
                Some(cred)
            }
            false => None,
        };

        let keys: Keys = serde_json::from_reader(File::open("./config/enclave_keys.json")?)?;
        let keys = keys.get_keys()?;
        let doors: Vec<Door> = vec![];
        debug!("{:?}", keys);
        Ok(Self { credentials, keys, doors })
    }

    pub fn update_files(&mut self) -> Result<()> {
        match &self.credentials {
            Some(cred) => Enclave::update_files_from_cred(cred)?,
            None => {}
        }
        Ok(())
    }

    fn update_files_from_cred(_credentials: &AuthCredential) -> Result<()> {
        info!("Downloading new files");
        // Load new files here

        Ok(())
    }

    pub fn key_to_door_check(&self, key_code: u64, door_id: u32) -> bool {
        match self.keys.get(&key_code) {
            Some(user) => {
                info!(
                    "User {} with id {}, is attempting to open door {}",
                    user.get_name(),
                    key_code,
                    user.get_door_id()
                );
                user.door_id == door_id
            }
            None => false,
        }
    }

    pub fn weigand_auth_check(&self, key_code: &Weigand, door_id: u32) -> bool {
        self.key_to_door_check(weigand_to_key(key_code), door_id)
    }

    pub fn weigand_open_door_request(&self, key_code: &Weigand, door_id: u32) -> Result<DoorAuthResponse> {
        if self.weigand_auth_check(key_code, door_id) {
            for door in &self.doors {
                if door.get_door_id() == door_id {
                    door.open_door();
                    return Ok(DoorAuthResponse::Success);
                }
            }
            return Ok(DoorAuthResponse::DoorIdNotFound);
        } else {
            Ok(DoorAuthResponse::UserNotAuthorised)
        }
        
    }

    pub fn run() {}
}
