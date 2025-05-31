use std::fs;

use serde::{Deserialize, Serialize};

use super::{
    models::ProjectModel,
    paths::{ensure_path, Paths, ProjectPaths},
};

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

pub trait Store {
    fn project_list(&self) -> Result<Vec<StoreProjectItem>>;
    fn get_project(&self, project_id: String) -> Result<ProjectModel>;
    fn save_project(&self, project: ProjectModel) -> Result<()>;

    const RATA: &str = "";
}

#[derive(Deserialize, Serialize, Default)]
pub struct StoreMapping {
    pub projects: Vec<StoreProjectItem>,
}

#[derive(Deserialize, Serialize)]
pub struct StoreProjectItem {
    pub id: String,
    pub name: String,
}

pub struct LocalStore<P: Paths> {
    paths: P,
}

impl LocalStore<ProjectPaths> {
    pub fn new() -> Self {
        Self {
            paths: ProjectPaths::new(),
        }
    }

    /// FileName used for store the `Vec(ProjectId, ProjectName)`
    const MAPPING_FILE: &str = "mapping.json";
}

impl Store for LocalStore<ProjectPaths> {
    fn project_list(&self) -> Result<Vec<StoreProjectItem>> {
        let store_path = ensure_path(self.paths.store_folder().join(Self::MAPPING_FILE)).unwrap();
        let contents = fs::read(store_path)?;

        // TODO: define another way to handle parsing error (case 1: empty file)
        let store_mapping: StoreMapping = serde_json::from_slice(&contents).unwrap_or_default();
        Ok(store_mapping.projects)
    }

    fn get_project(&self, project_id: String) -> Result<ProjectModel> {
        let project_file = self
            .paths
            .store_folder()
            .join(format!("{}.json", project_id));

        if !project_file.exists() {
            return Err(StoreError::NotFound);
        }

        let contents = fs::read(project_file)?;
        let project: ProjectModel = serde_json::from_slice(&contents)?;
        Ok(project)
    }

    fn save_project(&self, project: ProjectModel) -> Result<()> {
        let mut projects = self.project_list()?;

        // Search for the project (on mapping), and update the name
        let mut found_project = false;
        for project_item in &mut projects {
            if project_item.id == project.id() {
                project_item.name = project.name.to_owned();
                found_project = true;
                break;
            }
        }

        if !found_project {
            projects.push(StoreProjectItem {
                id: project.id().to_string(),
                name: project.name().to_string(),
            });
        }

        // Mapping file & project file paths
        let store_path = self.paths.store_folder().join(Self::MAPPING_FILE);
        let project_file_path = self
            .paths
            .store_folder()
            .join(format!("{}.json", project.id()));

        // Serialize the mapping & project
        let store_mapping = serde_json::to_string(&StoreMapping { projects })?;
        let contents = serde_json::to_string_pretty(&project)?;

        fs::write(store_path, store_mapping)?;
        fs::write(project_file_path, contents)?;

        Ok(())
    }
}

// // Generic implementation
// impl<P: Paths> LocalStore<P> {
//     pub fn with_paths(paths: P) -> Self {
//         Self { paths }
//     }
// }
