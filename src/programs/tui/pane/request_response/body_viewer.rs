use crate::programs::tui::pane::text_editor::TextEditor;
use ratatui::{layout::Rect, style::Stylize, text::Span, widgets::Widget};

pub enum BodyContentView {
    Text(TextEditor),
    Binary(HexDumpViewer),
    Empty,
}

impl Widget for &BodyContentView {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        match self {
            BodyContentView::Text(text_editor) => text_editor.render(area, buf),
            BodyContentView::Binary(hex_dump_viewer) => hex_dump_viewer.render(area, buf),
            BodyContentView::Empty => {
                if area.is_empty() {
                    return;
                }

                buf.set_span(
                    area.left(),
                    area.top(),
                    &Span::from("No body content").italic(),
                    area.width,
                );
            }
        }
    }
}

pub struct HexDumpViewer {
    lines: Vec<HexDumpLine>,
}

impl HexDumpViewer {
    pub fn new(bytes: &[u8]) -> Self {
        let lines = binary_to_hexdump(bytes, 15, 16);

        Self { lines }
    }
}

impl Widget for &HexDumpViewer {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        if area.is_empty() {
            return;
        }

        let mut top = area.top();

        for line in &self.lines {
            if top >= area.bottom() {
                break;
            }

            let line_txt = ratatui::text::Line::from(line.as_line_text());
            line_txt.render(Rect { y: top, ..area }, buf);
            top += 1;
        }
    }
}

pub struct HexDumpLine {
    bytes_per_line: usize,
    offset: usize,
    values: Vec<String>,
    ascii: String,
}

impl HexDumpLine {
    pub fn as_line_text(&self) -> String {
        let offset_hex = format!("{:08X}", self.offset * self.bytes_per_line);
        let mut hex_values = String::new();

        for (i, value) in self.values[0..self.values.len() - 1].iter().enumerate() {
            hex_values.push_str(value);
            hex_values.push(' ');
            if i == 7 {
                hex_values.push(' ');
            }
        }

        if let Some(value) = self.values.last() {
            hex_values.push_str(value);
        }

        let missing_values = self.bytes_per_line - self.values.len();
        hex_values.push_str(&"   ".repeat(missing_values));

        format!("{}: {} | {}", offset_hex, hex_values, self.ascii)
    }
}

fn binary_to_hexdump(bytes: &[u8], max_lines: usize, line_bytes: usize) -> Vec<HexDumpLine> {
    bytes
        .chunks(line_bytes)
        .take(max_lines)
        .enumerate()
        .map(|(offset, line)| {
            let hex_line = HexDumpLine {
                bytes_per_line: line_bytes,
                offset,
                values: line.iter().map(|byte| format!("{:02X}", byte)).collect(),
                ascii: line.iter().map(|byte| byte_to_ascci(*byte)).collect(),
            };
            hex_line
        })
        .collect()
}

fn byte_to_ascci(byte: u8) -> char {
    if byte.is_ascii() && 0x20 <= byte && byte <= 0x7e {
        return byte as char;
    }

    '.'
}

#[cfg(test)]
mod test {
    use super::{binary_to_hexdump, byte_to_ascci};

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

        let hexdump = binary_to_hexdump(&binary, 2, 16)
            .iter()
            .map(|line| line.as_line_text())
            .collect::<Vec<String>>()
            .join("\n");

        let result = "00000000: 48 65 6C 6C 6F 20 57 6F  72 6C 64 21             | Hello World!";
        assert_eq!(hexdump, result);

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

        let hexdump_2 = binary_to_hexdump(&binary_2, 11, 16)
            .iter()
            .map(|line| line.as_line_text())
            .collect::<Vec<String>>()
            .join("\n");

        let result_2 = [
            "00000000: 48 65 6C 6C 6F 2C 20 77  6F 72 6C 64 21 20 3A 29 | Hello, world! :)",
            "00000010: 00 01 02 10 20 30 40 50  60 70 7F 80 90 A0 B0 C0 | .... 0@P`p......",
            "00000020: FF EE DD CC BB AA 99 88  77 66 55 44 33 22 11 00 | ........wfUD3\"..",
            "00000030: 41 42 43 44 45 46 47 48  49 4A 4B 4C 4D 4E 4F 50 | ABCDEFGHIJKLMNOP",
            "00000040: 51 52 53 54 55 56 57 58  59 5A 61 62 63 64 65 66 | QRSTUVWXYZabcdef",
            "00000050: 67 68 69 6A 6B 6C 6D 6E  6F 70 71 72 73 74 75 76 | ghijklmnopqrstuv",
            "00000060: 77 78 79 7A 20 2E 2C 3F  21 27 22 28 29 5B 5D 7B | wxyz .,?!\'\"()[]{",
            "00000070: 7D 3C 3E 2F 5C 7C 60 7E  23 24 25 5E 26 2A 3A 3B | }<>/\\|`~#$%^&*:;",
            "00000080: 31 32 33 34 35 36 37 38  39 30 2B 2D 3D 5F 40 00 | 1234567890+-=_@.",
            "00000090: C3 A9 E2 82 AC F0 9F 98  80 EF BB BF 00 01 02 03 | ................",
        ]
        .join("\n");
        assert_eq!(hexdump_2, result_2);

        let binary_3 = [
            0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0x57, 0x6F, 0x72, 0x6C, 0x64, 0x21, 0x0A, 0x00,
            0xFF, 0x7F, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4A,
        ];

        let hexdump_3 = binary_to_hexdump(&binary_3, 11, 16)
            .iter()
            .map(|line| line.as_line_text())
            .collect::<Vec<String>>()
            .join("\n");

        let result_3 = [
            "00000000: 48 65 6C 6C 6F 20 57 6F  72 6C 64 21 0A 00 FF 7F | Hello World!....",
            "00000010: 41 42 43 44 45 46 47 48  49 4A                   | ABCDEFGHIJ",
        ]
        .join("\n");
        assert_eq!(hexdump_3, result_3);
    }
}
