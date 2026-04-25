use std::error::Error;
use std::time::Duration;
use evdev::Device as PhysDevice;


mod config;
mod mapping;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?;
    adapter.set_powered(true).await?;

    let tabmate_addr: bluer::Address = "20:73:10:00:D5:52".parse()?;

    // ハードウェアコードと仮想キーのマッピングを用意
    // tabmate.toml が存在すればそこから読み込み、なければデフォルト設定を使用
    let hw_map = mapping::get_hardware_map();
    let key_config = config::load_key_config(std::path::Path::new("tabmate.toml"));

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

        // 全デバイスを列挙して TABMATE を探す
        // 優先度1: BTN_TL (code=310, masterブランチで動作確認済み) をサポートするデバイス
        // 優先度2: 名前に "tabmate" を含むデバイス (大文字小文字無視)
        println!("=== 利用可能な入力デバイス一覧 ===");
        let mut path_by_cap: Option<std::path::PathBuf> = None;
        let mut path_by_name: Option<std::path::PathBuf> = None;
        for (path, dev) in evdev::enumerate() {
            let name = dev.name().unwrap_or("(名前なし)");
            let has_btn_tl = dev.supported_keys()
                .map(|keys| keys.contains(evdev::KeyCode::BTN_TL))
                .unwrap_or(false);
            println!("  {:?}  名前={}  BTN_TL={}", path, name, has_btn_tl);

            if has_btn_tl && path_by_cap.is_none() {
                path_by_cap = Some(path.clone());
            }
            if name.to_lowercase().contains("tabmate") && path_by_name.is_none() {
                path_by_name = Some(path);
            }
        }
        println!("===================================");

        let path = match path_by_cap.or(path_by_name) {
            Some(p) => p,
            None => {
                println!("TABMATEデバイスが見つかりません。再試行します...");
                tokio::time::sleep(Duration::from_secs(3)).await;
                continue;
            }
        };

        println!("デバイスを選択しました: {:?}", path);

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
                let ev_type = event.event_type().0;
                let code = event.code();
                let value = event.value();

                // 全イベントをデバッグ出力 (type=0はEV_SYN同期イベントなので除外)
                if ev_type != 0 {
                    println!("[RAW] type={} code={} value={}", ev_type, code, value);
                }

                // event_type == 1 は EV_KEY
                if ev_type == 1 {
                    if value == 1 || value == 0 {
                        if let Some(&(mode, button)) = hw_map.get(&code) {
                            if let Some(&key) = key_config.get(&(mode, button)) {
                                let result = if value == 1 {
                                    virtual_device.press(&key)
                                } else {
                                    virtual_device.release(&key)
                                };
                                if let Err(e) = result {
                                    println!("uinput エラー (press/release): {}", e);
                                }
                                if let Err(e) = virtual_device.synchronize() {
                                    println!("uinput エラー (synchronize): {}", e);
                                }
                            } else {
                                println!("key_config にキーが見つかりません: {:?} {:?}", mode, button);
                            }
                        } else {
                            println!("hw_map に未登録のコード: {} (value={})", code, value);
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
