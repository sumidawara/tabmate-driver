use evdev::Device as PhysDevice;
use std::error::Error;
use uinput::event::keyboard;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 1. Bluetooth セッションとアダプタの取得
    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?;
    adapter.set_powered(true).await?;

    let tabmate_addr: bluer::Address = "20:73:10:00:D5:52".parse()?;
    println!("TABMATEを探しています (MAC: {})...", tabmate_addr);

    // 2. Bluetooth 接続ループ
    let device = adapter.device(tabmate_addr)?;

    if !device.is_connected().await? {
        println!("TABMATEに接続を試みます...");
        loop {
            match device.connect().await {
                Ok(_) => {
                    println!("Bluetooth 接続成功！");
                    break;
                }
                Err(e) => {
                    println!("接続待ち... (電源を入れ直してください): {}", e);
                    tokio::time::sleep(Duration::from_secs(3)).await;
                }
            }
        }
    }

    tokio::time::sleep(Duration::from_secs(2)).await;

    // 3. ドライバー処理
    // event3が見つからない可能性を考慮
    let mut phys_dev = PhysDevice::open("/dev/input/event3")?;
    phys_dev.grab()?;

    let mut virtual_device = uinput::default()?
        .name("TABMATE Virtual Keyboard")?
        .event(uinput::event::Keyboard::All)?
        .create()?;

    println!("ドライバー稼働開始...");

    loop {
        for event in phys_dev.fetch_events()? {
            if event.event_type().0 == 1 && event.value() == 1 {
                match event.code() {
                    310 => virtual_device.click(&keyboard::Key::Up)?,
                    311 => virtual_device.click(&keyboard::Key::Down)?,
                    _ => println!("Raw Code: {}", event.code()),
                }
                virtual_device.synchronize()?;
            }
        }
    }
}