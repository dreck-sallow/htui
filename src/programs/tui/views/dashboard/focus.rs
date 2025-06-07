#[derive(PartialEq, Eq)]
pub enum ElementFocus {
    Collections,
    RequestBuilder,
    ResponseViewer,
}

#[derive(PartialEq, Eq)]
pub enum OverlayFocus {
    UpsertItem,
    MethodSelector,
}
