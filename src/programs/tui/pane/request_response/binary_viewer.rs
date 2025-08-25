use std::{cell::RefCell, rc::Rc};

use arboard::Clipboard;
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    widgets::Widget,
};

use crate::programs::tui::{
    common::{component::Interactive, list_utils},
    config::{keybinding, Config},
};

pub struct BinaryViewer {
    hexdump: HexDump,
}

impl BinaryViewer {
    pub fn new() -> Self {
        Self {
            hexdump: HexDump::new(&[], Style::default()),
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            hexdump: HexDump::new(bytes, Style::default().italic().bold()),
        }
    }

    pub fn set_data(&mut self, bytes: &[u8]) {
        self.hexdump.set_data(bytes);
    }
}

impl Widget for &BinaryViewer {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        (&self.hexdump).render(area, buf);
    }
}

impl Interactive for BinaryViewer {
    type Effect = ();

    type Params = (Rc<Config>, Rc<RefCell<Clipboard>>);

    fn on_key(
        &mut self,
        key: crossterm::event::KeyEvent,
        (config, clipboard): Self::Params,
    ) -> Option<Self::Effect> {
        if let Some(action) = config.keymap.match_global_action(key) {
            match action {
                keybinding::GlobalKeyAction::MoveDown => self.hexdump.next(),
                keybinding::GlobalKeyAction::MoveUp => self.hexdump.previous(),
                keybinding::GlobalKeyAction::CopyToClipboard => {
                    if let Some(txt) = self.hexdump.line_to_txt() {
                        let _ = clipboard.borrow_mut().set_text(txt);
                    }
                }
                _ => {}
            }
        }

        None
    }
}

// TODO: use the section area, to calculate the BYTES_PER_LINE / MAX_LINES
const BYTES_PER_LINE: usize = 30;

// Hexdump representation for ui
struct HexDump {
    bytes_per_line: usize,
    lines: Vec<HexLine>,
    selected: Option<usize>,
    selected_style: Style,
}

impl HexDump {
    pub fn new(bytes: &[u8], selected_style: Style) -> Self {
        let lines = Self::to_hex_lines(bytes, 15, BYTES_PER_LINE);
        let selected = if lines.is_empty() { None } else { Some(0) };

        Self {
            bytes_per_line: BYTES_PER_LINE,
            lines,
            selected,
            selected_style,
        }
    }

    pub fn set_data(&mut self, bytes: &[u8]) {
        self.lines = Self::to_hex_lines(bytes, 15, BYTES_PER_LINE);
        self.selected = if self.lines.is_empty() { None } else { Some(0) };
    }

    pub fn next(&mut self) {
        self.selected = list_utils::next(self.selected, self.lines.len());
    }

    pub fn previous(&mut self) {
        self.selected = list_utils::prev(self.selected);
    }

    pub fn line_to_txt(&self) -> Option<String> {
        self.selected
            .map(|idx| self.lines[idx].as_text(self.bytes_per_line))
    }

    fn to_hex_lines(bytes: &[u8], max_lines: usize, line_bytes: usize) -> Vec<HexLine> {
        bytes
            .chunks(line_bytes)
            .take(max_lines)
            .enumerate()
            .map(|(idx, line)| HexLine {
                offset: (idx * line_bytes) as u32,
                bytes: line.to_vec(),
            })
            .collect()
    }
}

impl Widget for &HexDump {
    fn render(self, mut area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        if area.is_empty() {
            return;
        }

        for (idx, line) in self.lines.iter().enumerate() {
            let is_selected = self
                .selected
                .map(|selected| selected == idx)
                .unwrap_or(false);

            // Draw selector symbol
            let symbol = if is_selected { "►" } else { " " };
            buf.set_string(area.left(), area.top(), symbol, Style::default());

            let line_area = Rect {
                height: 1,
                x: area.x + (symbol.chars().count() as u16) + 1,
                ..area
            };
            line.render_with_params(self.bytes_per_line, line_area, buf);

            if is_selected {
                buf.set_style(line_area, self.selected_style);
            }

            area.y += 1;
        }
    }
}

pub struct HexLine {
    offset: u32,
    bytes: Vec<u8>,
}

impl HexLine {
    fn as_text(&self, line_bytes: usize) -> String {
        let mut hexadecimals: Vec<String> = self
            .bytes
            .iter()
            .map(|byte| format!("{:02X}", byte))
            .collect();

        for _ in 0..(line_bytes - self.bytes.len()) {
            hexadecimals.push(String::from("  "));
        }

        let ascii: String = self.bytes.iter().map(|byte| byte_to_ascci(*byte)).collect();
        format!("{:08X}  {}  {}", self.offset, hexadecimals.join(" "), ascii)
    }

    fn render_with_params(
        &self,
        line_bytes: usize,
        mut area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) where
        Self: Sized,
    {
        if area.is_empty() {
            return;
        }

        // Print the offset
        area.x =
            buf.set_stringn(
                area.left(),
                area.top(),
                format!("{:08X}", self.offset),
                8,
                Style::default().dark_gray(),
            )
            .0 + 1;

        area.x = buf
            .set_stringn(area.left(), area.top(), " ", 1, Style::default())
            .0;

        // Use for apply styles in hexadecimals and printable ascci pane
        fn byte_style(byte: &u8) -> Style {
            if is_printable(&byte) {
                Style::default().green()
            } else {
                Style::default().red()
            }
        }

        // Print hexadecimals
        for byte in &self.bytes {
            area.x =
                buf.set_stringn(
                    area.left(),
                    area.top(),
                    format!("{:02X}", byte),
                    2,
                    byte_style(byte),
                )
                .0 + 1;
        }

        //Print missing
        for _ in 0..(line_bytes - self.bytes.len()) {
            area.x = buf
                .set_stringn(area.left(), area.top(), "  ", 2, Style::default())
                .0
                + 1;
        }

        area.x += 1;

        // Print the ascci text
        for byte in &self.bytes {
            area.x = buf
                .set_stringn(
                    area.left(),
                    area.top(),
                    byte_to_ascci(*byte).to_string(),
                    1,
                    byte_style(byte),
                )
                .0;
        }
    }
}

fn is_printable(byte: &u8) -> bool {
    byte.is_ascii() && 0x20 <= *byte && *byte <= 0x7e
}

fn byte_to_ascci(byte: u8) -> char {
    if is_printable(&byte) {
        return byte as char;
    }

    '.'
}

#[cfg(test)]
mod test {

    use ratatui::style::Style;

    use super::{byte_to_ascci, HexDump};

    #[cfg(test)]
    impl HexDump {
        pub fn new_using_sizes(bytes: &[u8], max_lines: usize, line_bytes: usize) -> Self {
            let lines = Self::to_hex_lines(bytes, max_lines, line_bytes);
            let selected = if lines.is_empty() { None } else { Some(0) };

            Self {
                bytes_per_line: line_bytes,
                lines,
                selected,
                selected_style: Style::default(),
            }
        }

        pub fn set(&mut self, bytes: &[u8], max_lines: usize, line_bytes: usize) {
            let lines = Self::to_hex_lines(bytes, max_lines, line_bytes);
            let selected = if lines.is_empty() { None } else { Some(0) };
            self.lines = lines;
            self.selected = selected;
        }

        pub fn to_string(&self) -> String {
            let mut raw = String::new();

            let line_bytes = self.bytes_per_line;

            for line in &self.lines {
                raw.push_str(&format!("{:08X}", line.offset));
                raw.push(':');
                raw.push(' ');

                for byte in &line.bytes {
                    raw.push_str(&format!("{:02X}", &byte));
                    raw.push(' ');
                }

                for _ in 0..(line_bytes - line.bytes.len()) {
                    raw.push_str("   ");
                }

                raw.push(' ');

                for byte in &line.bytes {
                    raw.push_str(&byte_to_ascci(*byte).to_string());
                }

                raw.push('\n');
            }

            raw.trim_end().to_string()
        }
    }

    #[test]
    fn test_byte_to_ascci() {
        let text: String = [68, 82, 69, 67, 107, 33]
            .iter()
            .map(|byte| byte_to_ascci(*byte))
            .collect();

        assert_eq!(text, "DRECk!")
    }

    #[test]
    fn test_binary_to_hexdump() {
        let binary = [
            0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0x57, 0x6F, 0x72, 0x6C, 0x64, 0x21,
        ];

        let mut hexdump = HexDump::new_using_sizes(&binary, 2, 16);

        let result = "00000000: 48 65 6C 6C 6F 20 57 6F 72 6C 64 21              Hello World!";
        assert_eq!(hexdump.to_string(), result);

        // Test case 2
        let binary_2 = [
            // Line 0
            0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x2C, 0x20, 0x77, 0x6F, 0x72, 0x6C, 0x64, 0x21, 0x20,
            0x3A, 0x29, // Line 1
            0x00, 0x01, 0x02, 0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70, 0x7F, 0x80, 0x90, 0xA0,
            0xB0, 0xC0, // Line 2
            0xFF, 0xEE, 0xDD, 0xCC, 0xBB, 0xAA, 0x99, 0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22,
            0x11, 0x00, // Line 3
            0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4A, 0x4B, 0x4C, 0x4D, 0x4E,
            0x4F, 0x50, // Line 4
            0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5A, 0x61, 0x62, 0x63, 0x64,
            0x65, 0x66, // Line 5
            0x67, 0x68, 0x69, 0x6A, 0x6B, 0x6C, 0x6D, 0x6E, 0x6F, 0x70, 0x71, 0x72, 0x73, 0x74,
            0x75, 0x76, // Line 6
            0x77, 0x78, 0x79, 0x7A, 0x20, 0x2E, 0x2C, 0x3F, 0x21, 0x27, 0x22, 0x28, 0x29, 0x5B,
            0x5D, 0x7B, // Line 7
            0x7D, 0x3C, 0x3E, 0x2F, 0x5C, 0x7C, 0x60, 0x7E, 0x23, 0x24, 0x25, 0x5E, 0x26, 0x2A,
            0x3A, 0x3B, // Line 8
            0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x30, 0x2B, 0x2D, 0x3D, 0x5F,
            0x40, 0x00, // Line 9
            0xC3, 0xA9, 0xE2, 0x82, 0xAC, 0xF0, 0x9F, 0x98, 0x80, 0xEF, 0xBB, 0xBF, 0x00, 0x01,
            0x02, 0x03,
        ];

        hexdump.set(&binary_2, 11, 16);

        let result_2 = [
            "00000000: 48 65 6C 6C 6F 2C 20 77 6F 72 6C 64 21 20 3A 29  Hello, world! :)",
            "00000010: 00 01 02 10 20 30 40 50 60 70 7F 80 90 A0 B0 C0  .... 0@P`p......",
            "00000020: FF EE DD CC BB AA 99 88 77 66 55 44 33 22 11 00  ........wfUD3\"..",
            "00000030: 41 42 43 44 45 46 47 48 49 4A 4B 4C 4D 4E 4F 50  ABCDEFGHIJKLMNOP",
            "00000040: 51 52 53 54 55 56 57 58 59 5A 61 62 63 64 65 66  QRSTUVWXYZabcdef",
            "00000050: 67 68 69 6A 6B 6C 6D 6E 6F 70 71 72 73 74 75 76  ghijklmnopqrstuv",
            "00000060: 77 78 79 7A 20 2E 2C 3F 21 27 22 28 29 5B 5D 7B  wxyz .,?!\'\"()[]{",
            "00000070: 7D 3C 3E 2F 5C 7C 60 7E 23 24 25 5E 26 2A 3A 3B  }<>/\\|`~#$%^&*:;",
            "00000080: 31 32 33 34 35 36 37 38 39 30 2B 2D 3D 5F 40 00  1234567890+-=_@.",
            "00000090: C3 A9 E2 82 AC F0 9F 98 80 EF BB BF 00 01 02 03  ................",
        ]
        .join("\n");
        assert_eq!(hexdump.to_string(), result_2);

        let binary_3 = [
            0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0x57, 0x6F, 0x72, 0x6C, 0x64, 0x21, 0x0A, 0x00,
            0xFF, 0x7F, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4A,
        ];

        hexdump.set(&binary_3, 11, 16);

        let result_3 = [
            "00000000: 48 65 6C 6C 6F 20 57 6F 72 6C 64 21 0A 00 FF 7F  Hello World!....",
            "00000010: 41 42 43 44 45 46 47 48 49 4A                    ABCDEFGHIJ",
        ]
        .join("\n");
        assert_eq!(hexdump.to_string(), result_3);
    }
}
