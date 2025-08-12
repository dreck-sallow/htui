use std::fs;

use keymap::KeyMap;
use ratatui::style::Color;
use serde::Deserialize;

use crate::paths::Paths;
pub mod keybinding;
// pub mod keybindings;
pub mod keymap;
// mod toml;

// #[derive(Deserialize)]
// pub struct Config {
//     #[serde(default)]
//     pub theme: Theme,
//     pub keymap: KeyMapV2,
// }

#[derive(Deserialize)]
pub struct ColorPair {
    pub fg: Color,
    pub bg: Color,
}

impl ColorPair {
    pub const fn new(fg: Color, bg: Color) -> Self {
        Self { fg, bg }
    }
}

#[derive(Deserialize)]
pub struct Theme {
    pub ui: ColorPair,
    pub tab: Color,
    pub tab_highlight: Color,
    /// Style for index selection (lists, table index cell)
    pub selection: ColorPair,
    pub dropdown: ColorPair,
    pub dropdown_highlight: ColorPair,
    pub border: Color,
    pub border_focus: Color,
}
impl Default for Theme {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl Theme {
    pub const DEFAULT: Self = Self {
        ui: ColorPair::new(Color::Indexed(254), Color::Indexed(236)),
        selection: ColorPair::new(Color::Indexed(16), Color::Indexed(107)),
        dropdown: ColorPair::new(Color::Indexed(15), Color::Indexed(60)),
        dropdown_highlight: ColorPair::new(Color::Indexed(15), Color::Indexed(61)),
        border: Color::Indexed(148),
        border_focus: Color::Indexed(107),
        tab: Color::Indexed(237),
        tab_highlight: Color::Indexed(148),
    };
}

const CONFIG_FILE: &str = "config.json";

pub fn load_config(paths: &Paths) -> Config {
    let config_folder = paths.config_folder();
    let config_file = config_folder.join(CONFIG_FILE);
    // serde_json::

    // This only return an error when the sematic or marlformed data is on the file (In this case I should show the error to user!)
    // TODO: show the error to user
    return serde_json::from_slice(&fs::read(config_file).unwrap()).unwrap();
}

// The keybinding deserialization
#[derive(Deserialize)]
pub struct Config {
    pub theme: Theme,
    pub keymap: KeyMap,
}
