use std::fmt::Display;

pub const METHODS: [&'static str; 5] = ["GET", "POST", "PUT", "DELETE", "OPTIONS"];

/// SUGGEST: Add some other tabs?
pub enum ViewTab {
    Headers,
    Body,
}

#[derive(PartialEq, Eq)]
pub enum Focus {
    Method,
    UrlInput,
    Tabs,
    TabContent,
}

impl Display for ViewTab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ViewTab::Headers => write!(f, "Headers"),
            ViewTab::Body => write!(f, "Body"),
        }
    }
}

pub struct RequestBuilderState {
    pub method: &'static str,
    view_tab: ViewTab,
    focus: Focus,
}

impl RequestBuilderState {
    pub fn new() -> Self {
        Self {
            method: "GET",
            view_tab: ViewTab::Headers,
            focus: Focus::Method,
        }
    }

    pub fn next_tab(&mut self) {
        self.view_tab = match self.view_tab {
            ViewTab::Headers => ViewTab::Body,
            ViewTab::Body => ViewTab::Headers,
        };
    }

    pub fn prev_tab(&mut self) {
        self.view_tab = match self.view_tab {
            ViewTab::Headers => ViewTab::Body,
            ViewTab::Body => ViewTab::Headers,
        };
    }

    pub fn view_tab(&mut self) -> &ViewTab {
        &self.view_tab
    }

    pub fn view_tab_index(&self) -> u8 {
        match self.view_tab {
            ViewTab::Headers => 0,
            ViewTab::Body => 1,
        }
    }

    pub fn focus(&self) -> &Focus {
        &self.focus
    }

    pub fn next_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Method => Focus::UrlInput,
            Focus::UrlInput => Focus::Tabs,
            Focus::Tabs => Focus::TabContent,
            Focus::TabContent => Focus::Method,
        };
    }

    pub fn prev_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Method => Focus::TabContent,
            Focus::UrlInput => Focus::Method,
            Focus::Tabs => Focus::UrlInput,
            Focus::TabContent => Focus::Tabs,
        };
    }
}
