pub mod collections;
pub mod method_selector;
pub mod url_input;

#[derive(Debug)]
pub enum ElementType {
    Collections,
    MethodSelector,
    UrlInput,
}

impl ElementType {
    pub fn next(&self) -> Self {
        match self {
            ElementType::Collections => ElementType::MethodSelector,
            ElementType::MethodSelector => ElementType::UrlInput,
            ElementType::UrlInput => ElementType::Collections,
        }
    }

    pub fn prev(&mut self) -> Self {
        match self {
            ElementType::Collections => ElementType::UrlInput,
            ElementType::MethodSelector => ElementType::Collections,
            ElementType::UrlInput => ElementType::MethodSelector,
        }
    }
}
