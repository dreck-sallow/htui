use std::fmt::Display;

use crossterm::event::KeyEvent;
use serde::Deserialize;

#[macro_export]
macro_rules! key {
    ($kb:expr) => {
        $crate::tui::config::keybinding::key_event::str_to_key_event($kb).unwrap()
    };
}

/// Macro rule for generate & implements a enum KeyAction
macro_rules! default_key_actions {
    ($name:ident { $($branch:ident => $default:expr), +}) => {

        #[derive(Deserialize, Copy, Clone, Debug, PartialEq)]
        #[serde(rename_all = "snake_case")]
        pub enum $name {
            $($branch),*
        }

        impl KeyAction for $name {
            fn _default_key(&self) -> KeyEvent {
                match self {
                    $(Self::$branch => key!($default),)*
                }
            }

            fn _default_list() -> Vec<Self> {
                vec![
                    $(Self::$branch,)*
                ]
            }
        }

        impl Display for $name {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(Self::$branch => formatter.write_str(stringify!($branch)),)*
                }
            }
        }

    };
}

pub trait KeyAction: Sized + Display {
    fn _default_key(&self) -> KeyEvent;
    fn _default_list() -> Vec<Self>;
}

// #[derive(Debug)]
pub struct KeyBinding<T: Display> {
    pub(super) key_event: KeyEvent,
    pub(super) action: T,
}

impl<T: KeyAction> KeyBinding<T> {
    pub fn from_action(action: T) -> Self {
        Self {
            key_event: action._default_key(),
            action,
        }
    }

    /// Method for returning the default list of actions (the list of enums of the key_binding)
    /// Used for implement the default keybindings of T: KeyAction
    pub(super) fn default_list() -> Vec<KeyBinding<T>> {
        let mut list = Vec::new();
        for action in T::_default_list() {
            list.push(Self::from_action(action));
        }

        list
    }

    pub fn new(action: T, key_event: KeyEvent) -> Self {
        Self { key_event, action }
    }
}

impl<T: KeyAction> std::fmt::Debug for KeyBinding<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}(key: {:?}, modifiers: {:?}) )",
            self.action, self.key_event.code, self.key_event.modifiers
        )
    }
}

default_key_actions!(GlobalKeyAction {
    NextFocus => "Tab",
    PreviousFocus=> "Shift+BackTab",
    FocusEnvContext=> "Alt+Shift+F",

    MoveDown  => "j",
    MoveUp => "k",
    MoveLeft => "h",
    MoveRight => "l",

    NextTab => "Shift+L",
    PreviousTab => "Shift+H",

    SendRequest => "Alt+Enter",
    SaveProject => "Alt+s",

    Undo => "Alt+u",
    Redo => "Alt+y",
    CopyToClipboard => "y",

    ClosePopup => "Esc",
    SubmitPopup => "Enter"
});

default_key_actions!(CollectionsKeyAction {
    Delete => "d",
    Edit => "e",
    SelectRequest => "Enter",
    CreateCollection => "c",
    CreateRequest => "r"
});

default_key_actions!(TableKeyAction {
    New => "n",
    Delete => "d",
    Edit => "e"
});

default_key_actions!(AppKeyAction {
    SearchProject => "Alt+Shift+N",
    CloseProject => "Alt+Shift+D",
    NextProject => "Alt+Shift+L",
    PreviousProject => "Alt+Shift+H",
    RenameProject => "Alt+Shift+R"
});

default_key_actions!(MethodUrlKeyAction {
    OpenDropdown => "Enter"
});

default_key_actions!(RequestBuilderKeyAction {
    OpenDropdown => "Shift+O",
    OpenEditor => "Shift+E"
});

default_key_actions!(ResponseViewerAction {
    // SaveBytes => "s",
    EditFilePath => "e"
});

// Implementations for deserializing a key_event
pub mod key_event {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    pub fn str_to_key_modifiers(value: &str) -> Option<KeyModifiers> {
        Some(match value.to_lowercase().as_ref() {
            "ctrl" => KeyModifiers::CONTROL,
            "shift" => KeyModifiers::SHIFT,
            "alt" | "option" => KeyModifiers::ALT,
            "super" | "command" | "windows" => KeyModifiers::SUPER,
            "meta" => KeyModifiers::META,
            "hyper" => KeyModifiers::HYPER,
            _ => return None,
        })
    }

    // pub fn char_to_key_code(ch: char) -> KeyCode {
    //     KeyCode::Char(ch)
    // }

    pub fn str_to_key_code(value: &str) -> Option<KeyCode> {
        Some(match value {
            "Backspace" => KeyCode::Backspace,
            "Enter" => KeyCode::Enter,
            "Left" => KeyCode::Left,
            "Right" => KeyCode::Right,
            "Up" => KeyCode::Up,
            "Down" => KeyCode::Down,
            "Home" => KeyCode::Home,
            "End" => KeyCode::End,
            "PageUp" => KeyCode::PageUp,
            "PageDown" => KeyCode::PageDown,
            "Tab" => KeyCode::Tab,
            "BackTab" => KeyCode::BackTab,
            "Delete" => KeyCode::Delete,
            "Insert" => KeyCode::Insert,
            "Null" => KeyCode::Null,
            "Esc" => KeyCode::Esc,
            "CapsLock" => KeyCode::CapsLock,
            "ScrollLock" => KeyCode::ScrollLock,
            "NumLock" => KeyCode::NumLock,
            "PrintScreen" => KeyCode::PrintScreen,
            "Pause" => KeyCode::Pause,
            "Menu" => KeyCode::Menu,
            "KeypadBegin" => KeyCode::KeypadBegin,
            txt => {
                let mut chars = txt.chars();

                let ch = chars.next()?;

                if chars.next().is_none() {
                    KeyCode::Char(ch)
                } else {
                    return None;
                }
            } // KeyCode::F(_) => todo!(),
        })
    }

    pub fn str_to_key_event(value: &str) -> Option<KeyEvent> {
        const SEPARATOR: &str = "+";
        let parts = value.split(SEPARATOR);

        let mut modifiers = KeyModifiers::NONE;

        for part in parts {
            match str_to_key_modifiers(part) {
                Some(_modifier) => {
                    if _modifier == KeyModifiers::NONE {
                        modifiers = modifiers
                    } else {
                        modifiers = modifiers | _modifier
                    }
                }
                None => {
                    return str_to_key_code(part).map(|code| KeyEvent::new(code, modifiers));
                }
            }
        }
        None
    }
}
