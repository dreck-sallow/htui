#[derive(PartialEq, Eq)]
pub enum ElementFocus {
    Collections,
    MethodUrlBar,
    RequestBuilder,
    ResponseViewer,
}

#[derive(PartialEq, Eq)]
pub enum OverlayFocus {
    UpsertItem,
    MethodSelector,
    BodySelector,
}
