use ratatui::layout::Rect;

use super::mode_input::{InputMode, ModeInput};

pub struct SelectOption {
    label: String,
    value: String,
}

impl SelectOption {
    pub fn new(label: String, value: String) -> Self {
        Self { label, value }
    }
}

impl From<&str> for SelectOption {
    fn from(value: &str) -> Self {
        Self {
            label: value.to_string(),
            value: value.to_string(),
        }
    }
}

impl From<String> for SelectOption {
    fn from(value: String) -> Self {
        Self {
            label: value.clone(),
            value: value,
        }
    }
}

pub struct SelectModeInput {
    input: ModeInput,
    options: Vec<SelectOption>,
    matching_options: Vec<usize>,
    selected_option: Option<usize>,
    range_replace: Option<(usize, usize)>,
    _render_area: Rect,
}

impl SelectModeInput {
    pub fn new(input: &str) -> Self {
        Self {
            input: ModeInput::new(input),
            options: Vec::new(),
            matching_options: Vec::new(),
            selected_option: None,
            range_replace: None,
            _render_area: Rect::default(),
        }
    }

    pub fn set_options(&mut self, options: Vec<SelectOption>) {
        //TODO: filter the matching options
        self.options = options;
        self.matching_options = self.options.iter().enumerate().map(|(i, _)| i).collect();
        self.selected_option = if self.matching_options.is_empty() {
            None
        } else {
            Some(0)
        };
    }

    pub fn txt(&self) -> &str {
        self.input.txt()
    }

    pub fn mode(&self) -> InputMode {
        self.input.mode()
    }

    pub fn have_selected_option(&self) -> bool {
        self.selected_option.is_some()
    }

    // fn replace_with_option(&mut self) {
    //     if let Some(opt) = self
    //         .selected_option
    //         .and_then(|i| self.matching_options.get(i))
    //     {
    //         let value = &self.options[*opt].value;
    //         // self.input.replace(input);
    //     }
    // }

    /// set the range for a substring allowed to select an option and replace
    pub fn set_range_replace(&mut self, range: (usize, usize)) {
        self.matching_options = vec![];

        let range_txt: String = {
            let start = range.0.min(range.1);
            let end = range.0.max(range.1);

            self.input
                .txt()
                .chars()
                .skip(start)
                .take(end.saturating_sub(start))
                .collect()
        };

        for (idx, opt) in self.options.iter().enumerate() {
            if opt.label.starts_with(&range_txt) {
                self.matching_options.push(idx);
            }
        }

        self.range_replace = Some(range);
    }

    pub fn reset_range_replace(&mut self) {
        self.range_replace = None;
    }

    pub fn select(&mut self, idx: usize) {
        //TODO: check the idx length bound with the options length
        self.selected_option = Some(idx);
    }

    pub fn deselect(&mut self) {
        self.selected_option = None;
    }
}

// impl UiElement for SelectModeInput {
//     type Params = ();

//     fn set_area(&mut self, area: ratatui::prelude::Rect) {
//         self.input.set_area(area);
//         self._render_area = Rect {
//             x: area.left(),
//             y: area.bottom(),
//             width: area.width,
//             // the size of length, add more height if need borders
//             height: 4,
//         };
//     }

//     fn draw(&self, params: Self::Params, frame: &mut ratatui::Frame) {
//         self.input.draw(params, frame);
//     }

//     fn draw_overlay(&self, params: Self::Params, frame: &mut ratatui::Frame) {
//         self.input.draw(params, frame);

//         if self.input.mode().is_write_mode() && !self.options.is_empty() {
//             let height = self._render_area.height as usize;

//             let (start_range, end_range) = {
//                 let start_idx = self
//                     .selected_option
//                     .map_or(0, |i| i.saturating_sub(i % height));

//                 let end_idx = start_idx + height;

//                 (
//                     start_idx,
//                     if end_idx < self.matching_options.len() {
//                         end_idx
//                     } else {
//                         self.matching_options.len()
//                     },
//                 )
//             };

//             let buf = frame.buffer_mut();
//             let mut area = self._render_area;

//             if end_range > 0 {
//                 for opt_idx in &self.matching_options[start_range..end_range] {
//                     let option = &self.options[*opt_idx];
//                     for x in area.left()..area.right() {
//                         buf[(x, area.top())].reset();
//                     }
//                     buf.set_stringn(
//                         area.left(),
//                         area.top(),
//                         &option.label,
//                         area.width as usize,
//                         Style::default(),
//                     );
//                     area.y += 1;
//                 }
//             }

//             // draw the selected option
//             if let Some(idx) = self.selected_option {
//                 let in_range_idx = idx.saturating_sub(start_range);
//                 buf.set_style(
//                     Rect {
//                         y: self._render_area.top() + in_range_idx as u16,
//                         height: 1,
//                         ..self._render_area // x: area,
//                                             // width: todo!(),
//                     },
//                     Style::default().red(),
//                 );
//             }
//         }
//     }
// }

// impl InteractiveElement<'_> for SelectModeInput {
//     type Params = ();

//     fn handle_key(&mut self, _params: Self::Params, key: crossterm::event::KeyEvent) {
//         match key.code {
//             KeyCode::Enter => {
//                 // select and append
//                 // Here I need applied the selection into the input
//                 // I need take the last keyword and replace
//                 // Or replace the entire string
//             }
//             KeyCode::Tab => {
//                 self.selected_option = list_utils::next(self.selected_option, self.options.len());
//             }
//             KeyCode::BackTab => {
//                 self.selected_option = list_utils::prev(self.selected_option);
//             }
//             _ => {
//                 self.input.handle_key(key);
//             }
//         }
//     }
// }
