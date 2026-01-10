use models::{ProjectModel, TimeId};
use serde::{Deserialize, Serialize};

use crate::paths::{ensure_path, ProjectPaths};

pub mod models;

#[derive(Deserialize, Serialize, Default)]
pub struct StoreMapping {
    pub projects: Vec<ProjectItem>,
}

#[derive(Deserialize, Serialize)]
pub struct ProjectItem {
    pub id: String,
    pub name: String,
}

pub struct Store {
    paths: ProjectPaths,
}

impl Store {
    pub fn new() -> Self {
        Self {
            paths: ProjectPaths::new(),
        }
    }

    pub async fn list_projects(&self) -> Result<Vec<ProjectItem>> {
        let store_path = ensure_path(self.paths.store_folder().join("mapping.json")).unwrap();
        let contents = tokio::fs::read(store_path).await?;

        // TODO: define another way to handle parsing error (case 1: empty file)
        let store_mapping: StoreMapping = serde_json::from_slice(&contents).unwrap_or_default();
        Ok(store_mapping.projects)
    }

    pub async fn find_one_project(&self, id: TimeId) -> Result<ProjectModel> {
        let project_file = self.paths.store_folder().join(format!("{}.json", id));

        if !project_file.exists() {
            return Err(StoreError::NotFound);
        }

        let contents = tokio::fs::read(project_file).await?;
        let project: ProjectModel = serde_json::from_slice(&contents)?;
        Ok(project)
    }

    pub async fn save_project(&self, project: ProjectModel) -> Result<()> {
        let mut projects = self.list_projects().await?;

        // Search for the project (on mapping), and update the name
        let mut found_project = false;
        for project_item in &mut projects {
            if project_item.id == project.id {
                project_item.name = project.name.to_owned();
                found_project = true;
                break;
            }
        }

        if !found_project {
            projects.push(ProjectItem {
                id: project.id.to_string(),
                name: project.name.to_string(),
            });
        }

        // Mapping file & project file paths
        let store_path = self.paths.store_folder().join("mapping.json");
        let project_file_path = self
            .paths
            .store_folder()
            .join(format!("{}.json", project.id));

        // Serialize the mapping & project
        let store_mapping = serde_json::to_string(&StoreMapping { projects })?;
        let contents = serde_json::to_string_pretty(&project)?;

        tokio::fs::write(store_path, store_mapping).await?;
        tokio::fs::write(project_file_path, contents).await?;

        Ok(())
    }
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

pub type Result<T> = std::result::Result<T, StoreError>;
