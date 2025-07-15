use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout, Margin, Rect},
    style::{Style, Stylize},
    widgets::{Block, Borders, Tabs},
};

use crate::programs::tui::common::component::{Drawable, Interactive};

use super::{body_editor::BodyEditorView, state::ElementFocus, text_editor::TextEditor};

#[derive(Clone, Copy)]
pub enum Tab {
    Headers,
    Body,
    Params,
}

impl Tab {
    pub fn as_idx(&self) -> usize {
        match self {
            Tab::Params => 0,
            Tab::Headers => 1,
            Tab::Body => 2,
        }
    }
}

impl AsRef<str> for Tab {
    fn as_ref(&self) -> &str {
        match self {
            Tab::Headers => "Headers",
            Tab::Body => "Body",
            Tab::Params => "Params",
        }
    }
}

pub struct RequestEditorComponent {
    tab: Tab,
    header_area: Rect,
    render_area: Rect,
    headers_editor: TextEditor,
    body_editor: BodyEditorView,
}

impl RequestEditorComponent {
    pub fn new() -> Self {
        Self {
            tab: Tab::Headers,
            header_area: Rect::default(),
            render_area: Rect::default(),
            headers_editor: TextEditor::new(true),
            body_editor: BodyEditorView::new(true),
        }
    }
}

impl Drawable for RequestEditorComponent {
    type Params = ElementFocus;

    fn draw<'a: 'painter, 'painter>(
        &'a self,
        painter: &mut crate::programs::tui::common::component::Painter<'painter>,
        params: Self::Params,
    ) {
        painter.render(move |frame| {
            let style = if params == super::state::ElementFocus::RequestBuilder {
                Style::default().blue()
            } else {
                Style::default()
            };

            let block = Block::bordered().border_style(style);
            frame.render_widget(block, self.render_area);

            let tabs = Tabs::new([
                format!(" {} ", Tab::Headers.as_ref()),
                format!(" {} ", Tab::Body.as_ref()),
            ])
            .select(self.tab.as_idx())
            .block(Block::new().borders(Borders::BOTTOM).border_style(style))
            .highlight_style(Style::default().blue());

            frame.render_widget(tabs, self.header_area);
        });

        match self.tab {
            Tab::Headers => {
                // painter.render(|frame| {
                //     frame.render_widget(&self.headers_editor, self.content_area);
                // });
            }
            Tab::Body => {
                // self.body_editor.draw(painter, state);
            }
            Tab::Params => {}
        }
    }

    fn set_area(&mut self, area: Rect) {
        let main_areas = Layout::vertical([Constraint::Length(2), Constraint::Fill(50)])
            .split(area.inner(Margin::new(1, 1)));

        self.render_area = area;
        self.header_area = main_areas[0];
        // self.content_area = main_areas[1];
        // self.body_editor.set_render_area(main_areas[1]);
    }
}

impl Interactive for RequestEditorComponent {
    type Effect = RequestEditorEffect;

    fn on_key(&mut self, key: crossterm::event::KeyEvent) -> Option<Self::Effect> {
        if key.kind == KeyEventKind::Press {
            let is_editing = match self.tab {
                Tab::Headers => self.headers_editor.mode().is_write_mode(),
                Tab::Body => self.body_editor.is_editing(),
                Tab::Params => false,
            };

            // let mut mutate_on_blur = |focus_navigation: FocusNavigation| {
            //     let idx = state.reader().current_request_idx().unwrap();
            //     let headers = {
            //         let mut map = HashMap::new();

            //         for line in self.headers_editor.lines() {
            //             let mut parts = line.splitn(1, ':');
            //             let key = parts.next().map(|txt| txt.trim().to_string());
            //             let value = parts.next().map(|txt| txt.trim().to_string());

            //             if let (Some(k), Some(v)) = (key, value) {
            //                 map.insert(k, v);
            //             }
            //         }

            //         map
            //     };

            //     mutator.add(EditRequest::new(idx, RequestEditType::Headers(headers)));
            //     mutator.add(EditRequest::new(
            //         idx,
            //         RequestEditType::Body(self.body_editor.body_type().clone()),
            //     ));
            //     mutator.add(SetFocus::new(focus_navigation));
            // };

            match key.code {
                KeyCode::Tab => match self.tab {
                    Tab::Headers => {
                        if is_editing {
                            self.headers_editor.handle_key(key)
                        } else {
                            self.tab = Tab::Body;
                        }
                    }
                    Tab::Body => {
                        if is_editing {
                            // self.body_editor.on_key(key, mutator, state);
                        } else {
                            // mutate_on_blur(FocusNavigation::Next);
                        }
                    }
                    Tab::Params => {}
                },
                KeyCode::BackTab => match self.tab {
                    Tab::Headers => {
                        if is_editing {
                            self.headers_editor.handle_key(key);
                        } else {
                            // mutate_on_blur(FocusNavigation::Prev);
                        }
                    }
                    Tab::Body => {
                        if is_editing {
                            // self.body_editor.on_key(key, mutator, state);
                        } else {
                            self.tab = Tab::Headers;
                        }
                    }
                    Tab::Params => {}
                },
                _ => match self.tab {
                    Tab::Headers => self.headers_editor.handle_key(key),
                    Tab::Body => {
                        // self.body_editor.on_key(key, mutator, state);
                    }
                    Tab::Params => {}
                },
            }
        }
        None
    }
}

pub enum RequestEditorEffect {
    NextFocus,
    PreviousFocus,
}

// pub struct RequestEditorView {
//     render_area: Rect,
//     header_area: Rect,
//     content_area: Rect,
//     tab: Tab,
//     headers_editor: TextEditor,
//     body_editor: BodyEditorView,
// }

// impl RequestEditorView {
//     pub fn new() -> Self {
//         Self {
//             render_area: Rect::default(),
//             header_area: Rect::default(),
//             content_area: Rect::default(),
//             tab: Tab::Headers,
//             headers_editor: TextEditor::new(true),
//             body_editor: BodyEditorView::new(true),
//         }
//     }

//     pub fn draw<'painter, 'this: 'painter, 'state: 'painter>(
//         &'this self,
//         painter: &mut Painter<'painter>,
//         state: &'state PaneState,
//     ) {
//         painter.render(|frame| {
//             let style = if state.is_focus(super::state::ElementFocus::RequestBuilder) {
//                 Style::default().blue()
//             } else {
//                 Style::default()
//             };

//             let block = Block::bordered().border_style(style);
//             frame.render_widget(block, self.render_area);

//             let tabs = Tabs::new([
//                 format!(" {} ", Tab::Headers.as_ref()),
//                 format!(" {} ", Tab::Body.as_ref()),
//             ])
//             .select(self.tab.as_idx())
//             .block(Block::new().borders(Borders::BOTTOM).border_style(style))
//             .highlight_style(Style::default().blue());

//             frame.render_widget(tabs, self.header_area);
//         });

//         match self.tab {
//             Tab::Headers => {
//                 painter.render(|frame| {
//                     frame.render_widget(&self.headers_editor, self.content_area);
//                 });
//             }
//             Tab::Body => {
//                 // self.body_editor.draw(painter, state);
//             } // Tab::Params => todo!(),
//         }
//     }

//     pub fn set_render_area(&mut self, area: Rect) {
//         let main_areas = Layout::vertical([Constraint::Length(2), Constraint::Fill(50)])
//             .split(area.inner(Margin::new(1, 1)));

//         self.render_area = area;
//         self.header_area = main_areas[0];
//         self.content_area = main_areas[1];
//         // self.body_editor.set_render_area(main_areas[1]);
//     }

//     pub fn handle_key(
//         &mut self,
//         key: KeyEvent,
//         collector: &mut MutationCollector,
//         state: &PaneState,
//     ) {
//         if key.kind == KeyEventKind::Press {
//             let is_editing = match self.tab {
//                 Tab::Headers => self.headers_editor.mode().is_write_mode(),
//                 Tab::Body => self.body_editor.is_editing(),
//                 // Tab::Params => false,
//             };

//             // let mut mutate_on_blur = |focus_navigation: FocusNavigation| {
//             //     let idx = state.reader().current_request_idx().unwrap();
//             //     let headers = {
//             //         let mut map = HashMap::new();

//             //         for line in self.headers_editor.lines() {
//             //             let mut parts = line.splitn(1, ':');
//             //             let key = parts.next().map(|txt| txt.trim().to_string());
//             //             let value = parts.next().map(|txt| txt.trim().to_string());

//             //             if let (Some(k), Some(v)) = (key, value) {
//             //                 map.insert(k, v);
//             //             }
//             //         }

//             //         map
//             //     };

//             //     mutator.add(EditRequest::new(idx, RequestEditType::Headers(headers)));
//             //     mutator.add(EditRequest::new(
//             //         idx,
//             //         RequestEditType::Body(self.body_editor.body_type().clone()),
//             //     ));
//             //     mutator.add(SetFocus::new(focus_navigation));
//             // };

//             match key.code {
//                 KeyCode::Tab => match self.tab {
//                     Tab::Headers => {
//                         if is_editing {
//                             self.headers_editor.handle_key(key)
//                         } else {
//                             self.tab = Tab::Body;
//                         }
//                     }
//                     Tab::Body => {
//                         if is_editing {
//                             // self.body_editor.on_key(key, mutator, state);
//                         } else {
//                             // mutate_on_blur(FocusNavigation::Next);
//                         }
//                     }
//                 },
//                 KeyCode::BackTab => match self.tab {
//                     Tab::Headers => {
//                         if is_editing {
//                             self.headers_editor.handle_key(key);
//                         } else {
//                             // mutate_on_blur(FocusNavigation::Prev);
//                         }
//                     }
//                     Tab::Body => {
//                         if is_editing {
//                             // self.body_editor.on_key(key, mutator, state);
//                         } else {
//                             self.tab = Tab::Headers;
//                         }
//                     }
//                 },
//                 _ => match self.tab {
//                     Tab::Headers => self.headers_editor.handle_key(key),
//                     Tab::Body => {
//                         // self.body_editor.on_key(key, mutator, state);
//                     }
//                 },
//             }
//         }
//     }
// }
