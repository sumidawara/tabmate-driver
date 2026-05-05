use std::error::Error;
use std::sync::Arc;

mod config;
mod mapping;
mod driver;
mod tui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (enabled_tx, enabled_rx) = tokio::sync::watch::channel(true);
    let (state_tx, state_rx) = tokio::sync::watch::channel(driver::DriverState::new());

    // Arc でラップし、TUI が終了しても Sender を保持し続ける
    let enabled_tx = Arc::new(enabled_tx);

    let driver_handle = tokio::spawn(driver::run(enabled_rx, state_tx));

    tui::run(Arc::clone(&enabled_tx), state_rx).await?;

    // TUI 終了後もドライバーはそのまま動かし続ける
    eprintln!("TUI を終了しました。ドライバーは継続中です (Ctrl+C で停止)。");

    tokio::select! {
        result = driver_handle => { result?; }
        _ = tokio::signal::ctrl_c() => {}
    }

    Ok(())
}
