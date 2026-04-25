use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use uinput::event::keyboard::Key;

use crate::mapping::{Button, Mode};

#[derive(Deserialize, Default)]
pub struct ModeConfig {
    pub side_top: Option<String>,
    pub side_bottom: Option<String>,
    pub a: Option<String>,
    pub b: Option<String>,
    pub c: Option<String>,
    pub d: Option<String>,
    pub wheel_up: Option<String>,
    pub wheel_down: Option<String>,
    pub wheel_push: Option<String>,
    pub back: Option<String>,
    pub up: Option<String>,
    pub down: Option<String>,
    pub right: Option<String>,
    pub left: Option<String>,
    pub q: Option<String>,
}

#[derive(Deserialize, Default)]
pub struct Config {
    pub mode1: Option<ModeConfig>,
    pub mode2: Option<ModeConfig>,
    pub mode3: Option<ModeConfig>,
    pub mode4: Option<ModeConfig>,
}

/// TOMLのキー名文字列を uinput の Key に変換する
pub fn parse_key(name: &str) -> Option<Key> {
    match name.to_lowercase().as_str() {
        // アルファベット
        "a" => Some(Key::A),
        "b" => Some(Key::B),
        "c" => Some(Key::C),
        "d" => Some(Key::D),
        "e" => Some(Key::E),
        "f" => Some(Key::F),
        "g" => Some(Key::G),
        "h" => Some(Key::H),
        "i" => Some(Key::I),
        "j" => Some(Key::J),
        "k" => Some(Key::K),
        "l" => Some(Key::L),
        "m" => Some(Key::M),
        "n" => Some(Key::N),
        "o" => Some(Key::O),
        "p" => Some(Key::P),
        "q" => Some(Key::Q),
        "r" => Some(Key::R),
        "s" => Some(Key::S),
        "t" => Some(Key::T),
        "u" => Some(Key::U),
        "v" => Some(Key::V),
        "w" => Some(Key::W),
        "x" => Some(Key::X),
        "y" => Some(Key::Y),
        "z" => Some(Key::Z),

        // 数字
        "1" => Some(Key::_1),
        "2" => Some(Key::_2),
        "3" => Some(Key::_3),
        "4" => Some(Key::_4),
        "5" => Some(Key::_5),
        "6" => Some(Key::_6),
        "7" => Some(Key::_7),
        "8" => Some(Key::_8),
        "9" => Some(Key::_9),
        "0" => Some(Key::_0),

        // ファンクションキー
        "f1"  => Some(Key::F1),
        "f2"  => Some(Key::F2),
        "f3"  => Some(Key::F3),
        "f4"  => Some(Key::F4),
        "f5"  => Some(Key::F5),
        "f6"  => Some(Key::F6),
        "f7"  => Some(Key::F7),
        "f8"  => Some(Key::F8),
        "f9"  => Some(Key::F9),
        "f10" => Some(Key::F10),
        "f11" => Some(Key::F11),
        "f12" => Some(Key::F12),
        "f13" => Some(Key::F13),
        "f14" => Some(Key::F14),
        "f15" => Some(Key::F15),
        "f16" => Some(Key::F16),
        "f17" => Some(Key::F17),
        "f18" => Some(Key::F18),
        "f19" => Some(Key::F19),
        "f20" => Some(Key::F20),
        "f21" => Some(Key::F21),
        "f22" => Some(Key::F22),
        "f23" => Some(Key::F23),
        "f24" => Some(Key::F24),

        // 修飾キー
        "ctrl" | "leftctrl" | "lctrl" | "leftcontrol" => Some(Key::LeftControl),
        "rightctrl" | "rctrl" | "rightcontrol"        => Some(Key::RightControl),
        "alt" | "leftalt" | "lalt"                    => Some(Key::LeftAlt),
        "rightalt" | "ralt" | "altgr"                 => Some(Key::RightAlt),
        "shift" | "leftshift" | "lshift"              => Some(Key::LeftShift),
        "rightshift" | "rshift"                       => Some(Key::RightShift),
        "super" | "meta" | "win" | "leftmeta" | "leftsuper" | "lsuper" => Some(Key::LeftMeta),
        "rightmeta" | "rightsuper" | "rmeta" | "rsuper"                => Some(Key::RightMeta),

        // 特殊キー
        "space"                         => Some(Key::Space),
        "enter" | "return"              => Some(Key::Enter),
        "esc" | "escape"                => Some(Key::Esc),
        "tab"                           => Some(Key::Tab),
        "backspace"                     => Some(Key::BackSpace),
        "delete" | "del"                => Some(Key::Delete),
        "insert" | "ins"                => Some(Key::Insert),
        "home"                          => Some(Key::Home),
        "end"                           => Some(Key::End),
        "pageup" | "pgup"               => Some(Key::PageUp),
        "pagedown" | "pgdown" | "pgdn"  => Some(Key::PageDown),
        "capslock"                      => Some(Key::CapsLock),
        "scrolllock"                    => Some(Key::ScrollLock),
        "numlock"                       => Some(Key::NumLock),
        "sysrq" | "printscreen"         => Some(Key::SysRq),

        // 矢印キー
        "up"    => Some(Key::Up),
        "down"  => Some(Key::Down),
        "left"  => Some(Key::Left),
        "right" => Some(Key::Right),

        // スクロール
        "scrollup"   => Some(Key::ScrollUp),
        "scrolldown" => Some(Key::ScrollDown),

        // 記号
        "minus" | "-"         => Some(Key::Minus),
        "equal" | "="         => Some(Key::Equal),
        "leftbrace" | "["     => Some(Key::LeftBrace),
        "rightbrace" | "]"    => Some(Key::RightBrace),
        "semicolon" | ";"     => Some(Key::SemiColon),
        "apostrophe" | "'"    => Some(Key::Apostrophe),
        "grave" | "`"         => Some(Key::Grave),
        "backslash" | "\\"    => Some(Key::BackSlash),
        "slash" | "/"         => Some(Key::Slash),
        "comma" | ","         => Some(Key::Comma),
        "dot" | "period" | "." => Some(Key::Dot),

        _ => None,
    }
}

fn apply_mode_config(
    map: &mut HashMap<(Mode, Button), Key>,
    mode: Mode,
    mc: &ModeConfig,
) {
    let entries: &[(&Option<String>, Button)] = &[
        (&mc.side_top,    Button::SideTop),
        (&mc.side_bottom, Button::SideBottom),
        (&mc.a,           Button::A),
        (&mc.b,           Button::B),
        (&mc.c,           Button::C),
        (&mc.d,           Button::D),
        (&mc.wheel_up,    Button::WheelUp),
        (&mc.wheel_down,  Button::WheelDown),
        (&mc.wheel_push,  Button::WheelPush),
        (&mc.back,        Button::Back),
        (&mc.up,          Button::Up),
        (&mc.down,        Button::Down),
        (&mc.right,       Button::Right),
        (&mc.left,        Button::Left),
        (&mc.q,           Button::Q),
    ];
    for (key_name_opt, button) in entries {
        if let Some(name) = key_name_opt {
            match parse_key(name) {
                Some(key) => {
                    map.insert((mode, *button), key);
                }
                None => {
                    eprintln!(
                        "警告: 不明なキー名 '{}' ({:?} / {:?}) は無視します",
                        name, mode, button
                    );
                }
            }
        }
    }
}

/// TOMLファイルを読み込み、デフォルト設定にマージしたキーコンフィグを返す。
/// ファイルが存在しない場合やパースエラーの場合はデフォルト設定をそのまま返す。
pub fn load_key_config(path: &Path) -> HashMap<(Mode, Button), Key> {
    let mut map = crate::mapping::default_key_config();

    let content = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => {
            println!(
                "設定ファイルが見つかりません ({:?})。デフォルト設定を使用します。",
                path
            );
            return map;
        }
    };

    let config: Config = match toml::from_str(&content) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("設定ファイルのパースエラー: {}。デフォルト設定を使用します。", e);
            return map;
        }
    };

    println!("設定ファイルを読み込みました: {:?}", path);

    for (mode, mc_opt) in [
        (Mode::Mode1, config.mode1.as_ref()),
        (Mode::Mode2, config.mode2.as_ref()),
        (Mode::Mode3, config.mode3.as_ref()),
        (Mode::Mode4, config.mode4.as_ref()),
    ] {
        if let Some(mc) = mc_opt {
            apply_mode_config(&mut map, mode, mc);
        }
    }

    map
}
