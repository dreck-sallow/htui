use ratatui::layout::Rect;

pub struct Areas {
    viewport: Rect,
    project_tabs: Rect,
    pane: Rect,
    collections: Rect,
    method_url: Rect,
    request_builder: RequestBuilder,
    responses: Responses,
    env_contexts: EnvContexts,
    placeholder: Rect,
}

pub struct RequestBuilder {
    area: Rect,
    header: Rect,
    content: Rect,
}

pub struct Responses {
    area: Rect,
    // status_line: Rect,
    header: Rect,
    content: Rect,
}

pub struct EnvContexts {
    area: Rect,
    details: Rect,
}
