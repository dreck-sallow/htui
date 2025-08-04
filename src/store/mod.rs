mod entity;
use async_trait::async_trait;
use models::ProjectModel;
use tokio::fs;

pub mod models;

use serde::{Deserialize, Serialize};

use crate::paths::{ensure_path, Paths};

#[derive(Deserialize, Serialize, Default)]
pub struct StoreMapping {
    pub projects: Vec<StoreProjectItem>,
}

#[derive(Deserialize, Serialize)]
pub struct StoreProjectItem {
    pub id: String,
    pub name: String,
}

#[derive(Debug)]
pub enum StoreError {
    Io(std::io::Error),
    Deserialize(serde_json::Error),
    NotFound,
}

impl From<std::io::Error> for StoreError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for StoreError {
    fn from(value: serde_json::Error) -> Self {
        Self::Deserialize(value)
    }
}

pub type StoreResult<T> = std::result::Result<T, StoreError>;

#[async_trait]
pub trait Store {
    async fn project_list(&self) -> StoreResult<Vec<StoreProjectItem>>;
    async fn get_project(&self, project_id: String) -> StoreResult<ProjectModel>;
    async fn save_project(&self, project: ProjectModel) -> StoreResult<()>;
}

const MAPPING_FILE: &str = "mapping.json";

pub struct LocalStore {
    paths: Paths,
    mapping_file: &'static str,
}

impl LocalStore {
    pub fn new(paths: Paths) -> Self {
        Self {
            paths,
            mapping_file: MAPPING_FILE,
        }
    }
}

#[async_trait]
impl Store for LocalStore {
    async fn project_list(&self) -> StoreResult<Vec<StoreProjectItem>> {
        let store_path = ensure_path(self.paths.store_folder().join(self.mapping_file)).unwrap();
        let contents = fs::read(store_path).await?;

        // TODO: define another way to handle parsing error (case 1: empty file)
        let store_mapping: StoreMapping = serde_json::from_slice(&contents).unwrap_or_default();
        Ok(store_mapping.projects)
    }

    async fn get_project(&self, project_id: String) -> StoreResult<ProjectModel> {
        let project_file = self
            .paths
            .store_folder()
            .join(format!("{}.json", project_id));

        if !project_file.exists() {
            return Err(StoreError::NotFound);
        }

        let contents = fs::read(project_file).await?;
        let project: ProjectModel = serde_json::from_slice(&contents)?;
        Ok(project)
    }

    async fn save_project(&self, project: ProjectModel) -> StoreResult<()> {
        let mut projects = self.project_list().await?;

        let exists_project = projects.iter().any(|p| project.id() == p.id);

        let project_file_path = self
            .paths
            .store_folder()
            .join(format!("{}.json", project.id()));

        let contents = serde_json::to_string(&project)?;

        if exists_project {
            fs::write(project_file_path, contents).await?;
        } else {
            projects.push(StoreProjectItem {
                id: project.id().to_string(),
                name: project.name().to_string(),
            });

            let store_path = self.paths.store_folder().join(self.mapping_file);
            let store_mapping = serde_json::to_string(&StoreMapping { projects })?;

            // TODO: add a transaction for avoid write in mapping and not save in josn file
            fs::write(store_path, store_mapping).await?;
            fs::write(project_file_path, contents).await?;
        }

        Ok(())
    }
}
