use std::collections::HashMap;
use uinput::event::keyboard::Key;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mode {
    Mode1,
    Mode2,
    Mode3,
    Mode4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Button {
    SideTop,
    SideBottom,
    A,
    B,
    C,
    D,
    WheelUp,
    WheelDown,
    WheelPush,
    Back,
    Up,
    Down,
    Right,
    Left,
    Q,
}

/// 物理的なスキャンコードから論理的なボタン（モードとボタンの組み合わせ）へのマッピング
pub fn get_hardware_map() -> HashMap<u16, (Mode, Button)> {
    let mut map = HashMap::new();

    let modes = [
        (Mode::Mode1, [304, 305, 306, 307, 308, 309, 310, 311, 312, 313, 314, 315, 316, 317, 318]),
        (Mode::Mode2, [319, 704, 705, 706, 707, 708, 709, 710, 711, 712, 713, 714, 715, 716, 717]),
        (Mode::Mode3, [718, 719, 720, 721, 722, 723, 724, 725, 726, 727, 728, 729, 730, 731, 732]),
        (Mode::Mode4, [733, 734, 735, 736, 737, 738, 739, 740, 741, 742, 743, 744, 745, 746, 747]),
    ];

    let buttons = [
        Button::SideTop,
        Button::SideBottom,
        Button::A,
        Button::B,
        Button::C,
        Button::D,
        Button::WheelUp,
        Button::WheelDown,
        Button::WheelPush,
        Button::Back,
        Button::Up,
        Button::Down,
        Button::Right,
        Button::Left,
        Button::Q,
    ];

    for (mode, mode_codes) in modes {
        for (i, code) in mode_codes.into_iter().enumerate() {
            map.insert(code, (mode, buttons[i]));
        }
    }

    map
}

/// 論理的なボタンから出力する仮想キーへのマッピング（将来的にはJSON/TOML等から読み込む想定）
pub fn default_key_config() -> HashMap<(Mode, Button), Key> {
    let mut config = HashMap::new();
    let hw_map = get_hardware_map();

    // デフォルトの設定として、各ボタンに適当なキーを割り当てる
    for (_code, (mode, button)) in hw_map {
        let key = match (mode, button) {
            (_, Button::WheelUp) => Key::Up,
            (_, Button::WheelDown) => Key::Down,
            (_, Button::WheelPush) => Key::Enter,
            (_, Button::Up) => Key::W,
            (_, Button::Down) => Key::S,
            (_, Button::Left) => Key::A,
            (_, Button::Right) => Key::D,
            (Mode::Mode1, Button::A) => Key::A,
            (Mode::Mode1, Button::B) => Key::B,
            // 適宜割り当て。エラーにならないよう全ての組み合わせにデフォルトを設定
            _ => Key::Space,
        };
        config.insert((mode, button), key);
    }

    config
}
