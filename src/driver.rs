use std::collections::{HashSet, HashMap, VecDeque};
use std::time::Duration;
use evdev::Device as PhysDevice;
use tokio::sync::watch;

use crate::config::{self, Action};
use crate::mapping;

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Disabled,
    Searching,
    Connecting,
    Connected,
}

#[derive(Debug, Clone)]
pub struct DriverState {
    pub status: ConnectionStatus,
    pub last_button: Option<String>,
    pub logs: VecDeque<String>,
}

impl DriverState {
    pub fn new() -> Self {
        Self {
            status: ConnectionStatus::Searching,
            last_button: None,
            logs: VecDeque::new(),
        }
    }
}

fn push_log(state_tx: &watch::Sender<DriverState>, msg: String) {
    state_tx.send_modify(|s| {
        if s.logs.len() >= 200 {
            s.logs.pop_front();
        }
        s.logs.push_back(msg);
    });
}

fn set_status(state_tx: &watch::Sender<DriverState>, status: ConnectionStatus) {
    state_tx.send_modify(|s| s.status = status);
}

pub async fn run(
    mut enabled_rx: watch::Receiver<bool>,
    state_tx: watch::Sender<DriverState>,
) {
    let hw_map = mapping::get_hardware_map();
    let key_maps = config::load_key_maps(std::path::Path::new("config.toml"));

    let build_result = (|| -> Result<uinput::Device, uinput::Error> {
        Ok(uinput::default()?
            .name("TABMATE Virtual Keyboard")?
            .event(uinput::event::Keyboard::All)?
            .event(uinput::event::Controller::Mouse(uinput::event::controller::Mouse::Left))?
            .event(uinput::event::Controller::Mouse(uinput::event::controller::Mouse::Right))?
            .event(uinput::event::Controller::Mouse(uinput::event::controller::Mouse::Middle))?
            .event(uinput::event::Controller::Mouse(uinput::event::controller::Mouse::Side))?
            .event(uinput::event::Controller::Mouse(uinput::event::controller::Mouse::Extra))?
            .event(uinput::event::Controller::Mouse(uinput::event::controller::Mouse::Forward))?
            .event(uinput::event::Controller::Mouse(uinput::event::controller::Mouse::Back))?
            .create()?)
    })();
    let mut virtual_device = match build_result {
        Ok(d) => d,
        Err(e) => {
            push_log(&state_tx, format!("uinput デバイスの作成に失敗: {}", e));
            return;
        }
    };
    tokio::time::sleep(Duration::from_millis(500)).await;
    push_log(&state_tx, "仮想キーボード準備完了".to_string());

    let tabmate_addr: bluer::Address = match "20:73:10:00:D5:52".parse() {
        Ok(a) => a,
        Err(e) => {
            push_log(&state_tx, format!("MACアドレスのパースエラー: {}", e));
            return;
        }
    };

    let session = match bluer::Session::new().await {
        Ok(s) => s,
        Err(e) => {
            push_log(&state_tx, format!("Bluetoothセッションの作成に失敗: {}", e));
            return;
        }
    };
    let adapter = match session.default_adapter().await {
        Ok(a) => a,
        Err(e) => {
            push_log(&state_tx, format!("Bluetoothアダプターの取得に失敗: {}", e));
            return;
        }
    };
    if let Err(e) = adapter.set_powered(true).await {
        push_log(&state_tx, format!("Bluetooth電源ONに失敗: {}", e));
        return;
    }

    'outer: loop {
        if !*enabled_rx.borrow() {
            set_status(&state_tx, ConnectionStatus::Disabled);
            loop {
                match enabled_rx.changed().await {
                    Err(_) => return,
                    Ok(_) => if *enabled_rx.borrow() { break },
                }
            }
        }

        set_status(&state_tx, ConnectionStatus::Searching);
        push_log(&state_tx, "TABMATEを探しています...".to_string());

        let device = match adapter.device(tabmate_addr) {
            Ok(d) => d,
            Err(e) => {
                push_log(&state_tx, format!("デバイスが見つかりません: {}", e));
                tokio::time::sleep(Duration::from_secs(3)).await;
                continue;
            }
        };

        if !device.is_connected().await.unwrap_or(false) {
            set_status(&state_tx, ConnectionStatus::Connecting);
            push_log(&state_tx, "接続を試みます...".to_string());
            match device.connect().await {
                Ok(_) => push_log(&state_tx, "Bluetooth 接続成功！".to_string()),
                Err(e) => {
                    push_log(&state_tx, format!("接続待ち (電源を入れ直してください): {}", e));
                    tokio::time::sleep(Duration::from_secs(3)).await;
                    continue;
                }
            }
        }

        tokio::time::sleep(Duration::from_secs(2)).await;

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
                push_log(&state_tx, "evdevにTABMATEが見つかりません。再試行...".to_string());
                tokio::time::sleep(Duration::from_secs(3)).await;
                continue;
            }
        };

        push_log(&state_tx, format!("デバイス: {:?}", path));

        let mut phys_dev = match PhysDevice::open(&path) {
            Ok(d) => d,
            Err(e) => {
                push_log(&state_tx, format!("デバイスを開けません: {}", e));
                tokio::time::sleep(Duration::from_secs(3)).await;
                continue;
            }
        };

        if let Err(e) = phys_dev.grab() {
            push_log(&state_tx, format!("警告: grab失敗: {}", e));
        }

        let mut stream = match phys_dev.into_event_stream() {
            Ok(s) => s,
            Err(e) => {
                push_log(&state_tx, format!("イベントストリーム作成失敗: {}", e));
                tokio::time::sleep(Duration::from_secs(3)).await;
                continue;
            }
        };

        set_status(&state_tx, ConnectionStatus::Connected);
        push_log(&state_tx, "ドライバー稼働開始".to_string());

        let mut button_state: HashSet<(mapping::Mode, mapping::Button)> = HashSet::new();
        let mut last_event_time: HashMap<(mapping::Mode, mapping::Button, i32), std::time::Instant> = HashMap::new();
        const DEBOUNCE: Duration = Duration::from_millis(80);

        loop {
            let event = tokio::select! {
                result = stream.next_event() => {
                    match result {
                        Ok(ev) => ev,
                        Err(e) => {
                            push_log(&state_tx, format!("切断: {}", e));
                            break;
                        }
                    }
                }
                result = enabled_rx.changed() => {
                    match result {
                        Err(_) => return,
                        Ok(_) => {
                            if !*enabled_rx.borrow() {
                                push_log(&state_tx, "停止します。BT切断中...".to_string());
                                let _ = device.disconnect().await;
                                continue 'outer;
                            }
                        }
                    }
                    continue;
                }
            };

            if event.event_type().0 != 1 { continue; }
            let code = event.code();
            let value = event.value();

            let Some(&(mode, button)) = hw_map.get(&code) else { continue; };

            let now = std::time::Instant::now();
            let event_key = (mode, button, value);
            if let Some(last) = last_event_time.get(&event_key) {
                if now.duration_since(*last) < DEBOUNCE { continue; }
            }
            last_event_time.insert(event_key, now);

            let is_hold = key_maps.hold.contains(&(mode, button));

            if value == 1 {
                if !button_state.insert((mode, button)) { continue; }
                if let Some(keys) = key_maps.press.get(&(mode, button)) {
                    let label = format!("{:?} {:?} → {:?}", mode, button, keys);
                    push_log(&state_tx, format!("⇒ {} {}", if is_hold { "hold" } else { "click" }, label));
                    state_tx.send_modify(|s| s.last_button = Some(format!("{:?}/{:?}", mode, button)));
                    let result = if is_hold {
                        send_press(&mut virtual_device, keys)
                    } else {
                        send_click(&mut virtual_device, keys)
                    };
                    if let Err(e) = result {
                        push_log(&state_tx, format!("uinput エラー: {}", e));
                    }
                }
            } else if value == 0 {
                if !button_state.remove(&(mode, button)) { continue; }
                if is_hold {
                    if let Some(keys) = key_maps.press.get(&(mode, button)) {
                        if let Err(e) = send_release(&mut virtual_device, keys) {
                            push_log(&state_tx, format!("uinput エラー: {}", e));
                        }
                    }
                }
                if let Some(keys) = key_maps.release.get(&(mode, button)) {
                    if let Err(e) = send_click(&mut virtual_device, keys) {
                        push_log(&state_tx, format!("uinput エラー: {}", e));
                    }
                }
            }
        }

        push_log(&state_tx, "再接続モードに入ります".to_string());
    }
}

fn send_click(dev: &mut uinput::Device, actions: &[Action]) -> Result<(), uinput::Error> {
    let (modifiers, main) = actions.split_at(actions.len().saturating_sub(1));
    for a in modifiers { a.press(dev)?; }
    if let Some(a) = main.first() {
        a.press(dev)?;
        dev.synchronize()?;
        a.release(dev)?;
    }
    for a in modifiers.iter().rev() { a.release(dev)?; }
    dev.synchronize()?;
    Ok(())
}

fn send_press(dev: &mut uinput::Device, actions: &[Action]) -> Result<(), uinput::Error> {
    for a in actions {
        a.press(dev)?;
        dev.synchronize()?;
    }
    Ok(())
}

fn send_release(dev: &mut uinput::Device, actions: &[Action]) -> Result<(), uinput::Error> {
    for a in actions.iter().rev() {
        a.release(dev)?;
        dev.synchronize()?;
    }
    Ok(())
}
