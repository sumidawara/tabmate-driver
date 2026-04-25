use std::error::Error;
use std::time::Duration;
use evdev::Device as PhysDevice;

mod config;
mod mapping;

use config::Action;

/// モディファイアを押した状態でメインキーを click する（一回きり）
fn send_click(dev: &mut uinput::Device, actions: &[Action]) -> Result<(), uinput::Error> {
    let (modifiers, main) = actions.split_at(actions.len().saturating_sub(1));

    // 1. 修飾キーを全部セット
    for a in modifiers {
        a.press(dev)?;
    }
    // 2. メインキーを叩く
    if let Some(a) = main.first() {
        a.press(dev)?;
        dev.synchronize()?; // ここで「同時押し」状態を通知
        a.release(dev)?;
    }
    // 3. 修飾キーを離す
    for a in modifiers.iter().rev() {
        a.release(dev)?;
    }
    dev.synchronize()?; // ここで「全部離した」状態を通知
    Ok(())
}

/// コンボキーを全て押す（hold モード用）
fn send_press(dev: &mut uinput::Device, actions: &[Action]) -> Result<(), uinput::Error> {
    for a in actions {
        a.press(dev)?;
        dev.synchronize()?;
    }
    Ok(())
}

/// コンボキーを全て離す（hold モード用）
fn send_release(dev: &mut uinput::Device, actions: &[Action]) -> Result<(), uinput::Error> {
    for a in actions.iter().rev() {
        a.release(dev)?;
        dev.synchronize()?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?;
    adapter.set_powered(true).await?;

    let tabmate_addr: bluer::Address = "20:73:10:00:D5:52".parse()?;

    // ハードウェアコードと仮想キーのマッピングを用意
    // tabmate.toml が存在すればそこから読み込み、なければデフォルト設定を使用
    let hw_map = mapping::get_hardware_map();
    let key_maps = config::load_key_maps(std::path::Path::new("tabmate.toml"));

    // uinput 仮想キーボードは起動時に1回だけ作成して使い回す。
    // BT再接続のたびに作り直すと、OSがデバイスを認識する前にキーイベントが
    // 送られてしまい、最初の数イベントがバッファされて遅延出力される問題があった。
    let mut virtual_device = uinput::default()?
        .name("TABMATE Virtual Keyboard")?
        .event(uinput::event::Keyboard::All)?
        .event(uinput::event::Controller::Mouse(uinput::event::controller::Mouse::Left))?
        .event(uinput::event::Controller::Mouse(uinput::event::controller::Mouse::Right))?
        .event(uinput::event::Controller::Mouse(uinput::event::controller::Mouse::Middle))?
        .event(uinput::event::Controller::Mouse(uinput::event::controller::Mouse::Side))?
        .event(uinput::event::Controller::Mouse(uinput::event::controller::Mouse::Extra))?
        .event(uinput::event::Controller::Mouse(uinput::event::controller::Mouse::Forward))?
        .event(uinput::event::Controller::Mouse(uinput::event::controller::Mouse::Back))?
        .create()?;
    // デバイスがX11/Wayland等に認識されるまで少し待つ
    tokio::time::sleep(Duration::from_millis(500)).await;
    println!("仮想キーボード準備完了");

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

        // TABMATE のゲームパッドプロファイルノードを探す
        // TABMATEは2つのevdevノードを持つ:
        //   "TABMATE"          → ゲームパッドプロファイル (BTN_TL あり) ← これが必要
        //   "TABMATE Keyboard" → キーボードプロファイル  (BTN_TL なし)
        let mut phys_dev_path = None;
        for (path, dev) in evdev::enumerate() {
            let name = dev.name().unwrap_or("").to_lowercase();
            let has_btn_tl = dev.supported_keys()
                .map(|keys| keys.contains(evdev::KeyCode::BTN_TL))
                .unwrap_or(false);
            if name.contains("tabmate") && has_btn_tl {
                phys_dev_path = Some(path);
                break;
            }
        }

        let path = match phys_dev_path {
            Some(p) => p,
            None => {
                println!("evdev にTABMATEデバイスが見つかりません。再試行します...");
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

        // 非同期ストリームに変換する。
        // fetch_events() はBluetoothの遅延でプレスとリリースを同じバッチで返すことがある。
        // next_event().await はイベントを1つずつtokio経由で受け取るため、
        // プレスとリリースが必ず別タイミングで処理される。
        let mut stream = match phys_dev.into_event_stream() {
            Ok(s) => s,
            Err(e) => {
                println!("イベントストリームの作成に失敗しました: {}", e);
                tokio::time::sleep(Duration::from_secs(3)).await;
                continue;
            }
        };

        println!("ドライバー稼働開始...");

        // 各ボタンの現在の押下状態を管理するセット (重複した同方向イベントを無視)
        let mut button_state: std::collections::HashSet<(mapping::Mode, mapping::Button)> =
            std::collections::HashSet::new();
        // (ボタン, value) ごとの最後の発火時刻を保持し、デバウンスする
        // TABMATEが完全な press-release サイクルを短時間で複数回送る現象に対処
        let mut last_event_time: std::collections::HashMap<
            (mapping::Mode, mapping::Button, i32),
            std::time::Instant,
        > = std::collections::HashMap::new();
        const DEBOUNCE: Duration = Duration::from_millis(80);

        let session_start = std::time::Instant::now();

        loop {
            let event = match stream.next_event().await {
                Ok(ev) => ev,
                Err(e) => {
                    println!("イベントの読み出しに失敗しました (切断されましたか?): {}", e);
                    break;
                }
            };

            // EV_KEY (type=1) のみ処理
            if event.event_type().0 != 1 {
                continue;
            }
            let code = event.code();
            let value = event.value();

            let Some(&(mode, button)) = hw_map.get(&code) else {
                continue;
            };

            let now = std::time::Instant::now();
            let elapsed_ms = now.duration_since(session_start).as_millis();
            println!("[RAW {}ms] code={} value={} button={:?}", elapsed_ms, code, value, button);

            // 時間ベースのデバウンス: 同じ (button, value) が短時間で再到着したら無視
            let event_key = (mode, button, value);
            if let Some(last) = last_event_time.get(&event_key) {
                if now.duration_since(*last) < DEBOUNCE {
                    println!("  → デバウンスでスキップ ({}ms以内)", DEBOUNCE.as_millis());
                    continue;
                }
            }
            last_event_time.insert(event_key, now);

            let is_hold = key_maps.hold.contains(&(mode, button));

            // 押し込んだとき
            if value == 1 {
                if !button_state.insert((mode, button)) {
                    println!("  → 既に押下中なのでスキップ");
                    continue;
                }
                if let Some(keys) = key_maps.press.get(&(mode, button)) {
                    let action = if is_hold { "press(hold)" } else { "click" };
                    println!("  ⇒ {} keys={:?}", action, keys);
                    let result = if is_hold {
                        send_press(&mut virtual_device, keys)
                    } else {
                        send_click(&mut virtual_device, keys)
                    };
                    if let Err(e) = result {
                        eprintln!("uinput エラー (press): {}", e);
                    }
                }
            
            //　離したとき
            } else if value == 0 {
                if !button_state.remove(&(mode, button)) {
                    println!("  → 押下状態でないのでスキップ");
                    continue;
                }
                if is_hold {
                    if let Some(keys) = key_maps.press.get(&(mode, button)) {
                        println!("  ⇒ release(hold) keys={:?}", keys);
                        if let Err(e) = send_release(&mut virtual_device, keys) {
                            eprintln!("uinput エラー (release): {}", e);
                        }
                    }
                }
                if let Some(keys) = key_maps.release.get(&(mode, button)) {
                    println!("  ⇒ release-click keys={:?}", keys);
                    if let Err(e) = send_click(&mut virtual_device, keys) {
                        eprintln!("uinput エラー (click on release): {}", e);
                    }
                }
            }
        }

        // stream がドロップされるとgrabも自動解除される
        println!("切断を検知しました。再接続モードに入ります。");
    }
}
