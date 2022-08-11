pub mod electronics;
pub mod timer;
pub mod weigand;

use color_eyre::eyre::Result;
use electronics::IElectronicController;
use tokio::{sync::watch, task::JoinHandle};
use tracing::{debug, info};

#[cfg(target_os = "arm")]
fn generate() -> Result<Box<dyn IElectronicController>> {
    Box::new(electronics::pi::controller::Controller::new()?)
}

#[cfg(not(target_os = "arm"))]
fn generate() -> Result<Box<dyn IElectronicController>> {
    Ok(Box::new(electronics::mock::Controller::new()?))
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    const GPIO_ZERO_LINE: u8 = 0;
    const GPIO_ONE_LINE: u8 = 1;

    let mut controller = generate()?;
    let mut reader =
        weigand::reader::WeigandReader::new(GPIO_ZERO_LINE, GPIO_ONE_LINE, &mut controller)?;

    let (tx, mut rx) = watch::channel::<weigand::Weigand>(weigand::Weigand::new(0)?);

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
            )
        }
    }
}
