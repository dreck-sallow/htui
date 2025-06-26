#[derive(Eq, PartialEq, Copy, Clone)]
pub enum ViewerTabs {
    Headers,
    Body,
}

impl Into<&str> for ViewerTabs {
    fn into(self) -> &'static str {
        match self {
            ViewerTabs::Headers => "Headers",
            ViewerTabs::Body => "Body",
        }
    }
}

pub struct ResponseViewerState {
    pub tab: ViewerTabs,
    pub is_tab_focus: bool,
}

impl ResponseViewerState {
    pub fn new() -> Self {
        Self {
            tab: ViewerTabs::Headers,
            is_tab_focus: true,
        }
    }

    pub fn next_tab(&mut self) {
        if self.tab == ViewerTabs::Headers {
            self.tab = ViewerTabs::Body
        }
    }

    pub fn prev_tab(&mut self) {
        if self.tab == ViewerTabs::Body {
            self.tab = ViewerTabs::Headers
        }
    }
}
