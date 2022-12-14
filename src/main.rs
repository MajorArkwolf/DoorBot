pub mod artifactory;
pub mod door;
pub mod electronics;
pub mod timer;
pub mod weigand;

use color_eyre::eyre::Result;
use electronics::IElectronicController;
use tokio::{sync::watch, task::JoinHandle};
use tracing::{debug, info};

use crate::artifactory::keys::weigand_to_key;

#[cfg(target_arch = "aarch64")]
fn generate() -> Result<Box<dyn IElectronicController>> {
    Ok(Box::new(electronics::pi::controller::Controller::new()?))
}

#[cfg(not(target_arch = "aarch64"))]
fn generate() -> Result<Box<dyn IElectronicController>> {
    Ok(Box::new(electronics::mock::Controller::new()?))
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    let mut enclave = artifactory::enclave::Enclave::load()?;
    info!("Loaded the Enclave");

    const GPIO_ZERO_LINE: u8 = 0;
    const GPIO_ONE_LINE: u8 = 1;
    const GPIO_RELAY_PIN: u8 = 2;

    let mut controller = generate()?;
    let mut reader =
        weigand::reader::WeigandReader::new(GPIO_ZERO_LINE, GPIO_ONE_LINE, &mut controller)?;

    let front_door = door::Door::new(
        controller.setup_output_pin(GPIO_RELAY_PIN)?,
        1,
        tokio::time::Duration::new(5, 0),
    );
    enclave.register_door(front_door);

    let (tx, mut rx) = watch::channel::<weigand::Weigand>(weigand::Weigand::new_unchecked(0));

    let _x: JoinHandle<Result<()>> = tokio::task::spawn(async move {
        reader.run(tx).await?;
        Ok(())
    });

    info!("Beginning main listener");
    loop {
        if rx.has_changed()? {
            let message = rx.borrow_and_update();
            debug!(
                "Building: {}, Card: {}",
                message.get_facility_code(),
                message.get_card_number()
            );
            debug!("Final Code: {}", weigand_to_key(&message));
            enclave.weigand_open_door_request(&message, 1)?;
        }
    }
}
