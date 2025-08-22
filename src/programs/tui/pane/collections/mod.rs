use std::{collections::HashMap, rc::Rc};

use crossterm::event::KeyEvent;
use list::{CollectionList, Item};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::Style,
    widgets::{Block, BorderType, Clear},
};
use state::{CollectionsState, MutableList};
use tui_textarea::Input;
use upsert_item::{UpsertItemPopup, UpsertMethod};

use crate::{
    app_project::models::{
        BodyContent, CollectionsModel, HttpMethod, KeyValueParam, RequestModel, SendRequestKey,
    },
    programs::tui::{
        common::{
            action_history::{ActionHistory, History, TrackAction},
            component::{Drawable, Interactive, WithHistory},
        },
        config::{
            keybinding::{CollectionsKeyAction, GlobalKeyAction},
            Config,
        },
    },
};

use super::{action::PaneAction, ElementFocus};

mod list;
pub mod state;
mod upsert_item;

pub struct CollectionsComponent {
    render_area: Rect,
    state: CollectionsState,
    menu: UpsertItemPopup,
    config: Rc<Config>,
    show_popup: bool,
    _history: ActionHistory<CollectionAction>,
}

impl CollectionsComponent {
    pub fn new(collections: Vec<CollectionsModel>, config: Rc<Config>) -> Self {
        Self {
            state: CollectionsState::from_list(collections),
            render_area: Rect::default(),
            menu: UpsertItemPopup::new(Rc::clone(&config)),
            show_popup: false,
            config,
            _history: ActionHistory::new(),
        }
    }

    pub fn current_request(&self) -> Option<&RequestModel> {
        match self.state.selected_request_idx() {
            Some(idx) => self.state.get_request(idx),
            None => None,
        }
    }

    pub fn as_collections(&self) -> Vec<CollectionsModel> {
        let collections = self.state.collections();

        let mut model_collections = Vec::with_capacity(collections.len());

        for collection in collections {
            // QUESTION:  We need clone?
            model_collections.push(CollectionsModel::from_parts(
                collection.id().to_string(),
                collection.name.clone(),
                collection.requests.clone(),
            ));
        }

        model_collections
    }

    /// Returns the key (collection & request ids) for get the request
    pub fn current_request_key(&self) -> Option<SendRequestKey> {
        if let Some((idx, sub_idx)) = self.state.selected_request_idx() {
            let collection = self.state.get_collection(idx).unwrap();
            let request_id = collection.requests[sub_idx].id().to_string();

            return Some(SendRequestKey::from((
                collection.id().to_string(),
                request_id,
            )));
        }
        None
    }

    pub fn set_data_from_method_url(&mut self, method: HttpMethod, url: String) {
        if let Some(idx) = self.state.selected_request_idx() {
            // TODO: make multiple actions as a single transactions for undo this operation
            self._history.apply(
                CollectionAction::EditRequestMethod { idx, method },
                &mut self.state,
            );
            self._history.apply(
                CollectionAction::EditRequestUrl { idx, url },
                &mut self.state,
            );
        }
    }

    pub fn set_data_from_request_editor(
        &mut self,
        params: Vec<KeyValueParam>,
        headers: Vec<KeyValueParam>,
        body: BodyContent,
    ) {
        if let Some(idx) = self.state.selected_request_idx() {
            // TODO: make multiple actions as a single transactions for undo this operation
            self._history.apply(
                CollectionAction::EditRequestParams { idx, params },
                &mut self.state,
            );
            self._history.apply(
                CollectionAction::EditRequestBody { idx, body },
                &mut self.state,
            );

            let headers = {
                let mut map = HashMap::new();

                for key_value in headers {
                    map.insert(key_value.key, key_value.value);
                }

                map
            };

            self._history.apply(
                CollectionAction::EditRequestHeaders { idx, headers },
                &mut self.state,
            );
        }
    }
}

impl Drawable for CollectionsComponent {
    type Params = ElementFocus;

    fn draw<'a: 'painter, 'painter>(
        &'a self,
        painter: &mut crate::programs::tui::common::component::Painter<'painter>,
        focus: Self::Params,
    ) {
        painter.render(move |frame| {
            let is_focus = focus == ElementFocus::Collections;
            let items: Vec<Item<'_>> = self
                .state
                .collections()
                .iter()
                .enumerate()
                .map(|(_i, coll)| {
                    let mut itm = Item::new(coll.name());

                    for (_sub_i, req) in coll.requests().iter().enumerate() {
                        // let name = match state.is_sending_request(SendRequestId(i, sub_i)) {
                        //     true => format!("pending {}", req.name()),
                        //     false => req.name().to_string(),
                        // };
                        itm.add_child(Item::new(req.name()));
                    }

                    itm
                })
                .collect();

            let collections = CollectionList::default()
                .set_items(items)
                .set_block(
                    Block::bordered()
                        .title(format!(
                            " Collections ({}) ",
                            self.state.collections().len()
                        ))
                        .border_type(if is_focus {
                            BorderType::Thick
                        } else {
                            BorderType::Plain
                        })
                        .border_style(
                            Style::default().fg(is_focus
                                .then_some(self.config.theme.border_focus)
                                .unwrap_or(self.config.theme.border)),
                        ),
                )
                .set_openeds(self.state.openeds().clone())
                .set_idx(self.state.idx())
                .set_highlight_style(
                    Style::default().fg(self.config.theme.selection.fg).bg(self
                        .config
                        .theme
                        .selection
                        .bg),
                );

            frame.render_widget(collections, self.render_area);
        });

        if self.show_popup {
            painter.render_last(|frame| {
                let area = {
                    let [area] = Layout::vertical([Constraint::Length(3)])
                        .flex(ratatui::layout::Flex::Center)
                        .areas(frame.area());

                    let [area] = Layout::horizontal([Constraint::Percentage(40)])
                        .flex(ratatui::layout::Flex::Center)
                        .areas(area);

                    area
                };

                frame.render_widget(Clear, area);
                self.menu.draw(frame, area);
            });
        }
    }

    fn set_area(&mut self, area: Rect) {
        self.render_area = area;
    }
}

impl Interactive for CollectionsComponent {
    type Effect = PaneAction;
    type Params = ();

    fn on_key(&mut self, key: KeyEvent, _params: Self::Params) -> Option<Self::Effect> {
        if self.show_popup {
            if let Some(key_action) = self.config.keymap.match_global_action(key) {
                match key_action {
                    GlobalKeyAction::ClosePopup => {
                        self.show_popup = false;
                    }
                    GlobalKeyAction::SubmitPopup => {
                        let text = self.menu.text();

                        match self.menu.method_type() {
                            UpsertMethod::CreateRequest => {
                                self._history.apply(
                                    CollectionAction::CreateRequest {
                                        coll_idx: self.state.idx().parent_idx(),
                                        name: text,
                                    },
                                    &mut self.state,
                                );
                            }
                            UpsertMethod::CreateCollection => {
                                self._history.apply(
                                    CollectionAction::CreateCollection(text),
                                    &mut self.state,
                                );
                            }
                            UpsertMethod::EditRequest => {
                                self._history.apply(
                                    CollectionAction::EditRequestName {
                                        idx: self.state.idx().child_idx(),
                                        name: text,
                                    },
                                    &mut self.state,
                                );
                            }
                            UpsertMethod::EditCollection => {
                                self._history.apply(
                                    CollectionAction::EditCollectionName {
                                        idx: self.state.idx().parent_idx(),
                                        new_name: text,
                                    },
                                    &mut self.state,
                                );
                            }
                        }

                        self.show_popup = false;
                    }
                    _ => {
                        self.menu.handle_input(Input::from(key));
                    }
                }
            } else {
                self.menu.handle_input(Input::from(key));
            }
        } else {
            let mut pane_action = None;

            let is_consumed_action =
                self.config
                    .keymap
                    .match_global_action(key)
                    .map_or(false, |key_action| {
                        match key_action {
                            GlobalKeyAction::NextFocus => {
                                pane_action = Some(PaneAction::NextFocus);
                            }
                            GlobalKeyAction::PreviousFocus => {
                                pane_action = Some(PaneAction::PreviousFocus);
                            }
                            GlobalKeyAction::MoveDown => {
                                self.state.next();
                            }
                            GlobalKeyAction::MoveUp => {
                                self.state.prev();
                            }
                            GlobalKeyAction::MoveLeft => {
                                self.state.close_collection(true);
                            }
                            GlobalKeyAction::MoveRight => {
                                self.state.open_collection(true);
                            }
                            _ => return false,
                        }
                        true
                    });

            if !is_consumed_action {
                if let Some(key_action) = self.config.keymap.match_collections_action(key) {
                    match key_action {
                        CollectionsKeyAction::Delete => match self.state.idx() {
                            state::Idx::None => {}
                            state::Idx::Parent(i) => {
                                self._history.apply(
                                    CollectionAction::DeleteCollection { idx: i },
                                    &mut self.state,
                                );
                            }
                            state::Idx::Child(i, sub_i) => {
                                self._history.apply(
                                    CollectionAction::DeleteRequest((i, sub_i)),
                                    &mut self.state,
                                );
                            }
                        },
                        CollectionsKeyAction::Edit => match self.state.idx() {
                            state::Idx::None => {}
                            state::Idx::Parent(i) => {
                                let collection = self.state.collections().get(i).unwrap();
                                self.menu
                                    .set_state(UpsertMethod::EditCollection, &collection.name);
                                self.show_popup = true;
                            }
                            state::Idx::Child(_, _) => {
                                let name = self.state.current_request().unwrap().name();
                                self.menu.set_state(UpsertMethod::EditRequest, name);
                                self.show_popup = true;
                            }
                        },
                        CollectionsKeyAction::SelectRequest => {
                            if let state::Idx::Child(i, sub_i) = self.state.idx() {
                                self._history.apply(
                                    CollectionAction::SelectRequestIdx(Some((i, sub_i))),
                                    &mut self.state,
                                );
                                // return Some(CollectionEffect::ChangeCurrentRequest);
                                return Some(PaneAction::ChangeRequest);
                            }
                        }
                        CollectionsKeyAction::CreateCollection => {
                            self.show_popup = true;
                            self.menu.set_state(UpsertMethod::CreateCollection, "");
                        }
                        CollectionsKeyAction::CreateRequest => {
                            if !self.state.idx().is_none() {
                                self.show_popup = true;
                                self.menu.set_state(UpsertMethod::CreateRequest, "");
                            }
                        }
                    }
                }
            }

            return pane_action;
        }

        None
    }
}

enum CollectionAction {
    SelectRequestIdx(Option<(usize, usize)>),
    // Collection actions
    CreateCollection(String),
    InsertCollection {
        idx: usize,
        collection: CollectionsModel,
    },
    DeleteCollection {
        idx: usize,
    },
    EditCollectionName {
        idx: usize,
        new_name: String,
    },

    // Request actions
    CreateRequest {
        coll_idx: usize,
        name: String,
    },
    InsertRequest {
        idx: (usize, usize),
        request: RequestModel,
    },
    DeleteRequest((usize, usize)),
    EditRequestName {
        idx: (usize, usize),
        name: String,
    },
    EditRequestMethod {
        idx: (usize, usize),
        method: HttpMethod,
    },
    EditRequestUrl {
        idx: (usize, usize),
        url: String,
    },
    EditRequestHeaders {
        idx: (usize, usize),
        headers: HashMap<String, String>,
    },
    EditRequestParams {
        idx: (usize, usize),
        params: Vec<KeyValueParam>,
    },
    EditRequestBody {
        idx: (usize, usize),
        body: BodyContent,
    },
}

impl TrackAction for CollectionAction {
    type State = CollectionsState;

    fn apply(&self, state: &mut Self::State) -> Option<Self>
    where
        Self: Sized,
    {
        match self {
            CollectionAction::SelectRequestIdx(idx) => {
                let previous_selection = state.selected_request_idx();

                state.select_request_idx(*idx);

                Some(CollectionAction::SelectRequestIdx(previous_selection))
            }
            CollectionAction::CreateCollection(name) => {
                let collection = CollectionsModel::new(name.to_string());

                let inserted_index = state.collections().len();
                state.insert_collection(inserted_index, collection);

                Some(CollectionAction::DeleteCollection {
                    idx: inserted_index,
                })
            }
            CollectionAction::InsertCollection { idx, collection } => {
                state.insert_collection(*idx, collection.clone());

                Some(CollectionAction::DeleteCollection { idx: *idx })
            }
            CollectionAction::DeleteCollection { idx } => {
                let collection = state.remove_collection(*idx).unwrap();
                Some(CollectionAction::InsertCollection {
                    idx: *idx,
                    collection,
                })
            }
            CollectionAction::EditCollectionName { idx, new_name } => {
                let collection = state.get_collection_mut(*idx).unwrap();
                let previous_name = collection.name.to_string();

                collection.name = new_name.to_string();

                Some(CollectionAction::EditCollectionName {
                    idx: *idx,
                    new_name: previous_name,
                })
            }
            CollectionAction::CreateRequest { coll_idx, name } => {
                let collection = state.get_collection(*coll_idx).unwrap();
                let req = RequestModel::new(name.to_string());
                let inserted_i = collection.requests.len();

                state.insert_request(*coll_idx, inserted_i, req);

                Some(CollectionAction::DeleteRequest((*coll_idx, inserted_i)))
            }
            CollectionAction::InsertRequest { idx, request } => {
                state.insert_request(idx.0, idx.1, request.clone());

                Some(CollectionAction::DeleteRequest(*idx))
            }
            CollectionAction::DeleteRequest(idx) => {
                let request = state.remove_request(idx.to_owned()).unwrap();

                Some(CollectionAction::InsertRequest { idx: *idx, request })
            }
            CollectionAction::EditRequestName { idx, name } => {
                let idx = idx.to_owned();
                let request = state.get_request_mut(idx).unwrap();
                let previous_name = request.name().to_string();

                state.edit_request(idx, |req| {
                    req.set_name(name.clone());
                });

                Some(CollectionAction::EditRequestName {
                    idx,
                    name: previous_name,
                })
            }
            CollectionAction::EditRequestMethod { idx, method } => {
                let idx = idx.to_owned();
                let request = state.get_request_mut(idx).unwrap();
                let previous_method = request.method();

                state.edit_request(idx, |req| {
                    req.set_method(*method);
                });

                Some(CollectionAction::EditRequestMethod {
                    idx,
                    method: previous_method,
                })
            }
            CollectionAction::EditRequestUrl { idx, url } => {
                let idx = idx.to_owned();
                let request = state.get_request_mut(idx).unwrap();
                let previous_url = request.url().to_string();

                state.edit_request(idx, |req| {
                    req.set_url(url.clone());
                });

                Some(CollectionAction::EditRequestUrl {
                    idx,
                    url: previous_url,
                })
            }
            CollectionAction::EditRequestHeaders { idx, headers } => {
                let idx = idx.to_owned();
                let request = state.get_request_mut(idx).unwrap();
                let previous_headers = request.headers_map().clone();

                state.edit_request(idx, |req| {
                    req.set_headers(headers.clone());
                });

                Some(CollectionAction::EditRequestHeaders {
                    idx,
                    headers: previous_headers,
                })
            }
            CollectionAction::EditRequestBody { idx, body } => {
                let idx = idx.to_owned();
                let request = state.get_request_mut(idx).unwrap();
                let previous_body = request.body().clone();

                state.edit_request(idx, |req| {
                    req.set_body(body.clone());
                });

                Some(CollectionAction::EditRequestBody {
                    idx,
                    body: previous_body,
                })
            }
            CollectionAction::EditRequestParams { idx, params } => {
                let idx = idx.to_owned();
                let request = state.get_request_mut(idx).unwrap();
                let previous_params = request.params.clone();

                state.edit_request(idx, |req| {
                    req.params = params.clone();
                });

                Some(CollectionAction::EditRequestParams {
                    idx,
                    params: previous_params,
                })
            }
        }
    }
}

impl WithHistory for CollectionsComponent {
    fn undo(&mut self) {
        self._history.undo(&mut self.state);
    }

    fn redo(&mut self) {
        self._history.redo(&mut self.state);
    }
}

// pub enum CollectionEffect {
//     NextFocus,
//     PreviousFocus,
//     ChangeCurrentRequest,
// }
