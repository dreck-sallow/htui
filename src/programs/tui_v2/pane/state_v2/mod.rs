use std::os::unix::fs::MetadataExt;

use collections::{
    BodyContent, BodyForm, CollectionItem, CollectionsList, FileContent, FileInfo, RequestItem,
};
use responses::Responses;
use table::TableState;

use crate::store::models::ProjectModel;

pub mod collections;
pub mod param_item;
pub mod responses;
pub mod table;

#[derive(PartialEq, Eq)]
pub enum SectionFocus {
    Collections,
    RequestBar,
    RequestBuilder,
    ResponseViewer,
}

pub struct PaneState {
    pub(crate) focus: SectionFocus,
    pub(crate) collections: CollectionsList,
    // pub(crate) environments: Environments,
    // pub(crate) selected_env_context: Option<usize>,
    pub(crate) responses: Responses,
}

impl PaneState {
    pub fn new(list: CollectionsList) -> Self {
        Self {
            focus: SectionFocus::Collections,
            collections: list,
            responses: Responses::new(),
        }
    }
}

pub async fn from_project_model(project: ProjectModel) -> PaneState {
    let mut list = Vec::new();
    for coll in project.collections {
        let mut reqs = Vec::new();

        for req in coll.requests {
            let body = match req.body {
                crate::store::models::RequestBody::None => BodyContent::None,
                crate::store::models::RequestBody::Text(st) => BodyContent::Text(st),
                crate::store::models::RequestBody::Json(v) => BodyContent::Text(v.to_string()),
                crate::store::models::RequestBody::FormUrlEncoded(m) => {
                    let mut list = Vec::new();
                    for param in m {
                        list.push(param_item::ParamItem {
                            enable: param.enable,
                            key: param.key,
                            value: param.value,
                        });
                    }

                    BodyContent::FormUrlEncoded(TableState::from(list))
                }
                crate::store::models::RequestBody::FormData(m) => {
                    let mut list = Vec::new();
                    for param in m {
                        list.push((
                            param.is_file,
                            param_item::ParamItem {
                                enable: param.enable,
                                key: param.key,
                                value: param.value,
                            },
                        ));
                    }

                    BodyContent::FormData(BodyForm::from(list))
                }
                crate::store::models::RequestBody::File(path) => {
                    if path.is_file() {
                        match tokio::fs::metadata(&path).await {
                            Ok(m) => {
                                let name = path.file_name().unwrap().to_str().unwrap().to_string();
                                let path_str = path.to_str().unwrap().to_string();

                                BodyContent::File(FileContent::Content {
                                    path,
                                    info: FileInfo {
                                        name,
                                        size: m.size().to_string(),
                                        path: path_str,
                                    },
                                })
                            }
                            Err(_) => BodyContent::File(FileContent::None),
                        }
                    } else {
                        BodyContent::File(FileContent::None)
                    }
                }
            };

            let headers = {
                let mut table = TableState::new();
                for header in req.headers {
                    table.add_row(param_item::ParamItem {
                        enable: header.enable,
                        key: header.key,
                        value: header.value,
                    });
                }
                table
            };

            let params = {
                let mut table = TableState::new();
                for param in req.params {
                    table.add_row(param_item::ParamItem {
                        enable: param.enable,
                        key: param.key,
                        value: param.value,
                    });
                }
                table
            };

            reqs.push(RequestItem::new_v2(
                req.id, req.name, req.url, headers, params, req.method, body,
            ));
        }
        list.push(CollectionItem::new_v2(coll.id, coll.name).with_reqs(reqs));
    }

    PaneState::new(CollectionsList::new().with_collections(list))
}
