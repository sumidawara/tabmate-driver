use std::error::Error;

mod config;
mod mapping;
mod driver;
mod tui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (enabled_tx, enabled_rx) = tokio::sync::watch::channel(true);
    let (state_tx, state_rx) = tokio::sync::watch::channel(driver::DriverState::new());

    tokio::spawn(driver::run(enabled_rx, state_tx));

    tui::run(enabled_tx, state_rx).await?;

    Ok(())
}
