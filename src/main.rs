use std::error::Error;
use std::time::Duration;
use evdev::Device as PhysDevice;


mod mapping;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?;
    adapter.set_powered(true).await?;

    let tabmate_addr: bluer::Address = "20:73:10:00:D5:52".parse()?;

    // ハードウェアコードと仮想キーのマッピングを事前に用意
    let hw_map = mapping::get_hardware_map();
    let key_config = mapping::default_key_config();

    loop {
        println!("TABMATEを探しています (MAC: {})...", tabmate_addr);

        let device = match adapter.device(tabmate_addr) {
            Ok(d) => d,
            Err(e) => {
                println!("デバイスが見つかりません: {}", e);
                tokio::time::sleep(Duration::from_secs(3)).await;
                continue;
            }
        };

        if !device.is_connected().await.unwrap_or(false) {
            println!("TABMATEに接続を試みます...");
            match device.connect().await {
                Ok(_) => {
                    println!("Bluetooth 接続成功！");
                }
                Err(e) => {
                    println!("接続待ち... (電源を入れ直してください): {}", e);
                    tokio::time::sleep(Duration::from_secs(3)).await;
                    continue;
                }
            }
        }

        tokio::time::sleep(Duration::from_secs(2)).await;

        // /dev/input/eventX の中から名前が "TABMATE" を含むものを探す
        let mut phys_dev_path = None;
        for (path, dev) in evdev::enumerate() {
            if let Some(name) = dev.name() {
                if name.contains("TABMATE") {
                    phys_dev_path = Some(path);
                    break;
                }
            }
        }

        let path = match phys_dev_path {
            Some(p) => p,
            None => {
                println!("evdev に TABMATE が見つかりません。再試行します...");
                tokio::time::sleep(Duration::from_secs(3)).await;
                continue;
            }
        };

        println!("デバイスを検出しました: {:?}", path);

        let mut phys_dev = match PhysDevice::open(&path) {
            Ok(d) => d,
            Err(e) => {
                println!("デバイスを開けません: {}", e);
                tokio::time::sleep(Duration::from_secs(3)).await;
                continue;
            }
        };

        // デバイスをGrabして他のシステムに入力がいかないようにする
        if let Err(e) = phys_dev.grab() {
            println!("警告: デバイスの grab に失敗しました: {}", e);
        }

        let mut virtual_device = uinput::default()?
            .name("TABMATE Virtual Keyboard")?
            .event(uinput::event::Keyboard::All)?
            .create()?;

        println!("ドライバー稼働開始...");

        // evdevからイベントを読み出すループ
        loop {
            let events = match phys_dev.fetch_events() {
                Ok(evs) => evs,
                Err(e) => {
                    println!("イベントの読み出しに失敗しました (切断されましたか?): {}", e);
                    // 切断されたと思われるので、Bluetooth接続待ちのループに戻る
                    break;
                }
            };

            for event in events {
                // event_type == 1 は EV_KEY, value == 1 は Pressed, value == 0 は Released
                if event.event_type().0 == 1 {
                    let code = event.code();
                    let value = event.value();

                    if value == 1 || value == 0 {
                        if let Some(&(mode, button)) = hw_map.get(&code) {
                            if let Some(&key) = key_config.get(&(mode, button)) {
                                if value == 1 {
                                    let _ = virtual_device.press(&key);
                                } else {
                                    let _ = virtual_device.release(&key);
                                }
                                let _ = virtual_device.synchronize();
                            }
                        } else {
                            if value == 1 {
                                println!("Unknown raw code: {}", code);
                            }
                        }
                    }
                }
            }
        }

        // Grabの解除（念のため）
        let _ = phys_dev.ungrab();
        println!("切断を検知しました。再接続モードに入ります。");
    }
}
