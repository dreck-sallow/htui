use std::fmt::Display;

pub const METHODS: [&'static str; 5] = ["GET", "POST", "PUT", "DELETE", "OPTIONS"];

/// SUGGEST: Add some other tabs?
pub enum ViewTab {
    Headers,
    Body,
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
}

impl RequestBuilderState {
    pub fn new() -> Self {
        Self {
            method: "GET",
            view_tab: ViewTab::Headers,
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
}
