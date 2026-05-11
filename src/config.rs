use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use uinput::event::controller::Mouse;
use uinput::event::keyboard::Key;

use crate::mapping::{Button, Mode};

/// キーボードキーまたはマウスボタンのいずれかの操作
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    Key(Key),
    Mouse(Mouse),
}

impl Action {
    pub fn press(&self, dev: &mut uinput::Device) -> Result<(), uinput::Error> {
        match self {
            Action::Key(k) => dev.press(k),
            Action::Mouse(m) => dev.press(&uinput::event::Controller::Mouse(*m)),
        }
    }
    pub fn release(&self, dev: &mut uinput::Device) -> Result<(), uinput::Error> {
        match self {
            Action::Key(k) => dev.release(k),
            Action::Mouse(m) => dev.release(&uinput::event::Controller::Mouse(*m)),
        }
    }
}

#[derive(Deserialize, Default)]
pub struct ModeConfig {
    pub side_top: Option<String>,
    pub side_top_release: Option<String>,
    pub side_top_hold: Option<bool>,
    pub side_bottom: Option<String>,
    pub side_bottom_release: Option<String>,
    pub side_bottom_hold: Option<bool>,
    pub a: Option<String>,
    pub a_release: Option<String>,
    pub a_hold: Option<bool>,
    pub b: Option<String>,
    pub b_release: Option<String>,
    pub b_hold: Option<bool>,
    pub c: Option<String>,
    pub c_release: Option<String>,
    pub c_hold: Option<bool>,
    pub d: Option<String>,
    pub d_release: Option<String>,
    pub d_hold: Option<bool>,
    pub wheel_up: Option<String>,
    pub wheel_up_release: Option<String>,
    pub wheel_up_hold: Option<bool>,
    pub wheel_down: Option<String>,
    pub wheel_down_release: Option<String>,
    pub wheel_down_hold: Option<bool>,
    pub wheel_push: Option<String>,
    pub wheel_push_release: Option<String>,
    pub wheel_push_hold: Option<bool>,
    pub back: Option<String>,
    pub back_release: Option<String>,
    pub back_hold: Option<bool>,
    pub up: Option<String>,
    pub up_release: Option<String>,
    pub up_hold: Option<bool>,
    pub down: Option<String>,
    pub down_release: Option<String>,
    pub down_hold: Option<bool>,
    pub right: Option<String>,
    pub right_release: Option<String>,
    pub right_hold: Option<bool>,
    pub left: Option<String>,
    pub left_release: Option<String>,
    pub left_hold: Option<bool>,
    pub q: Option<String>,
    pub q_release: Option<String>,
    pub q_hold: Option<bool>,
}

#[derive(Deserialize, Default)]
pub struct Config {
    pub mode1: Option<ModeConfig>,
    pub mode2: Option<ModeConfig>,
    pub mode3: Option<ModeConfig>,
    pub mode4: Option<ModeConfig>,
}

/// 押したとき・手放したときのアクションマップをまとめた構造体
pub struct KeyMaps {
    /// ボタン押下時に発火するアクション列（複数でコンボキー）
    pub press: HashMap<(Mode, Button), Vec<Action>>,
    /// ボタン解放時に発火するアクション列
    pub release: HashMap<(Mode, Button), Vec<Action>>,
    /// hold モード（長押しで連打）のボタン集合。それ以外は click（一回きり）
    pub hold: HashSet<(Mode, Button)>,
}

/// マウスボタン名文字列を Mouse に変換する
fn parse_mouse(name: &str) -> Option<Mouse> {
    match name {
        "mouseleft"  | "mouse1" | "leftclick"   | "lclick" | "lmb" => Some(Mouse::Left),
        "mouseright" | "mouse2" | "rightclick"  | "rclick" | "rmb" => Some(Mouse::Right),
        "mousemiddle"| "mouse3" | "middleclick" | "mclick" | "mmb" => Some(Mouse::Middle),
        "mouseside"     => Some(Mouse::Side),
        "mouseextra"    => Some(Mouse::Extra),
        "mouseforward"  => Some(Mouse::Forward),
        "mouseback"     => Some(Mouse::Back),
        _ => None,
    }
}

/// TOMLのキー名文字列を Action に変換する。
/// 先にキーボードキーとして解釈を試み、ダメならマウスボタンとして解釈する。
pub fn parse_key(name: &str) -> Option<Action> {
    let lower = name.to_lowercase();
    if let Some(k) = parse_keyboard_key(&lower) {
        return Some(Action::Key(k));
    }
    if let Some(m) = parse_mouse(&lower) {
        return Some(Action::Mouse(m));
    }
    None
}

/// 内部用: キーボードキー名のみをパースする
fn parse_keyboard_key(name: &str) -> Option<Key> {
    match name {
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
        "minus" | "-"          => Some(Key::Minus),
        "equal" | "="          => Some(Key::Equal),
        "leftbrace" | "["      => Some(Key::LeftBrace),
        "rightbrace" | "]"     => Some(Key::RightBrace),
        "semicolon" | ";"      => Some(Key::SemiColon),
        "apostrophe" | "'"     => Some(Key::Apostrophe),
        "grave" | "`"          => Some(Key::Grave),
        "backslash" | "\\"     => Some(Key::BackSlash),
        "slash" | "/"          => Some(Key::Slash),
        "comma" | ","          => Some(Key::Comma),
        "dot" | "period" | "." => Some(Key::Dot),

        _ => None,
    }
}

/// "Ctrl+Shift+Z" や "Ctrl+MouseRight" のような文字列を Action のリストに変換する
pub fn parse_combo(s: &str) -> Option<Vec<Action>> {
    let actions: Option<Vec<Action>> = s.split('+')
        .map(|part| parse_key(part.trim()))
        .collect();
    match actions {
        Some(v) if !v.is_empty() => Some(v),
        _ => None,
    }
}

fn apply_mode_config(
    press: &mut HashMap<(Mode, Button), Vec<Action>>,
    release: &mut HashMap<(Mode, Button), Vec<Action>>,
    hold: &mut HashSet<(Mode, Button)>,
    mode: Mode,
    mc: &ModeConfig,
) {
    // (押したときキー, 手放したときキー, holdフラグ, ボタン) の組
    let entries: &[(&Option<String>, &Option<String>, &Option<bool>, Button)] = &[
        (&mc.side_top,    &mc.side_top_release,    &mc.side_top_hold,    Button::SideTop),
        (&mc.side_bottom, &mc.side_bottom_release,  &mc.side_bottom_hold, Button::SideBottom),
        (&mc.a,           &mc.a_release,            &mc.a_hold,           Button::A),
        (&mc.b,           &mc.b_release,            &mc.b_hold,           Button::B),
        (&mc.c,           &mc.c_release,            &mc.c_hold,           Button::C),
        (&mc.d,           &mc.d_release,            &mc.d_hold,           Button::D),
        (&mc.wheel_up,    &mc.wheel_up_release,     &mc.wheel_up_hold,    Button::WheelUp),
        (&mc.wheel_down,  &mc.wheel_down_release,   &mc.wheel_down_hold,  Button::WheelDown),
        (&mc.wheel_push,  &mc.wheel_push_release,   &mc.wheel_push_hold,  Button::WheelPush),
        (&mc.back,        &mc.back_release,         &mc.back_hold,        Button::Back),
        (&mc.up,          &mc.up_release,           &mc.up_hold,          Button::Up),
        (&mc.down,        &mc.down_release,         &mc.down_hold,        Button::Down),
        (&mc.right,       &mc.right_release,        &mc.right_hold,       Button::Right),
        (&mc.left,        &mc.left_release,         &mc.left_hold,        Button::Left),
        (&mc.q,           &mc.q_release,            &mc.q_hold,           Button::Q),
    ];

    for (press_name, release_name, hold_flag, button) in entries {
        if let Some(name) = press_name {
            if name.is_empty() {
                // 空文字列は「何もしない」を明示。デフォルトのバインドも削除する
                press.remove(&(mode, *button));
            } else {
                match parse_combo(name) {
                    Some(keys) => { press.insert((mode, *button), keys); }
                    None => eprintln!("警告: 不明なキー名 '{}' ({:?}/{:?} press) は無視します", name, mode, button),
                }
            }
        }
        if let Some(name) = release_name {
            if name.is_empty() {
                release.remove(&(mode, *button));
            } else {
                match parse_combo(name) {
                    Some(keys) => { release.insert((mode, *button), keys); }
                    None => eprintln!("警告: 不明なキー名 '{}' ({:?}/{:?} release) は無視します", name, mode, button),
                }
            }
        }
        if hold_flag.unwrap_or(false) {
            hold.insert((mode, *button));
        }
    }
}

/// TOMLファイルを読み込み、デフォルト設定にマージした KeyMaps を返す。
/// ファイルが存在しない場合やパースエラーの場合はデフォルト設定で返す。
pub fn load_key_maps(path: &Path) -> KeyMaps {
    let mut press = crate::mapping::default_key_config();
    let mut release = crate::mapping::default_release_config();
    let mut hold = crate::mapping::default_hold_config();

    let content = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => {
            println!("設定ファイルが見つかりません ({:?})。デフォルト設定を使用します。", path);
            return KeyMaps { press, release, hold };
        }
    };

    let config: Config = match toml::from_str(&content) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("設定ファイルのパースエラー: {}。デフォルト設定を使用します。", e);
            return KeyMaps { press, release, hold };
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
            apply_mode_config(&mut press, &mut release, &mut hold, mode, mc);
        }
    }

    KeyMaps { press, release, hold }
}
