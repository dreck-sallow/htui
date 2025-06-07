use crate::store::models::HttpMethod;

use super::upsert_item::UpsertMethod;

#[derive(Clone)]
pub enum Action {
    UpsertItem(UpsertMethod, String),
    SaveUpsertItem(UpsertMethod, String),
    SelectMethod((u16, u16), HttpMethod),
    SelectedMethod(HttpMethod),
}
