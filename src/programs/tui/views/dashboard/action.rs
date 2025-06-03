use super::upsert_item::UpsertMethod;

#[derive(Clone)]
pub enum Action {
    UpsertItem(UpsertMethod, String),
    SaveUpsertItem(UpsertMethod, String),
    // NextFocus,
}
