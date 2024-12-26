use std::{
    fs, io,
    path::{Path, PathBuf},
};

use project::{Project, StoreMapping};
pub mod project;

#[derive(Debug)]
pub enum StoreError {
    NotFoundProject,
    IO(io::Error),
    Serde(serde_json::Error),
    TomlDeserialize(toml::de::Error),
    TomlSerialize(toml::ser::Error),
}

pub struct Store {
    mapping_store: MappingStore,
    projects_store: ProjectStore,
}

impl Store {
    pub fn new<P: AsRef<Path>>(config_file_path: P, config_dir_path: P) -> Self {
        Self {
            mapping_store: MappingStore::new(config_file_path.as_ref().to_path_buf()),
            projects_store: ProjectStore::new(config_dir_path.as_ref().to_path_buf()),
        }
    }

    pub fn get_project_by_name(&self, name: &str) -> Option<Project> {
        if let Ok(mapping) = self.mapping_store.load_mapping() {
            if let Some(itm) = mapping.find_by_name(name) {
                return self.projects_store.load_project(&itm.id).ok();
            }
        }

        None
    }
}

/// Datatype used for handling the ´store.toml´ file,
/// where are put the mapping of each project with its
/// own data file
pub struct MappingStore {
    store_file_path: PathBuf,
}

impl MappingStore {
    pub fn new(path: PathBuf) -> Self {
        Self {
            store_file_path: path,
        }
    }

    pub fn load_mapping(&self) -> Result<StoreMapping, StoreError> {
        let content = fs::read_to_string(&self.store_file_path)?;
        let store_mapping: StoreMapping = toml::from_str(&content)?;
        Ok(store_mapping)
    }

    pub fn save_mapping(&self, mapping: &StoreMapping) -> Result<(), StoreError> {
        fs::write(&self.store_file_path, toml::to_string(mapping)?)?;
        Ok(())
    }
}

pub struct ProjectStore {
    store_dir_path: PathBuf,
}

impl ProjectStore {
    pub fn new(path: PathBuf) -> Self {
        assert!(path.is_dir());

        Self {
            store_dir_path: path,
        }
    }

    pub fn load_project(&self, project_id: &str) -> Result<Project, StoreError> {
        let content = fs::read_to_string(self.store_dir_path.join(format!("{}.json", project_id)))?;
        let project: Project = serde_json::from_str(&content)?;
        Ok(project)
    }

    pub fn save_project(&self, project_id: &str, project: &Project) -> Result<(), StoreError> {
        fs::write(
            self.store_dir_path.join(format!("{}.json", project_id)),
            serde_json::ser::to_string(project)?,
        )?;
        Ok(())
    }
}

impl From<io::Error> for StoreError {
    fn from(value: io::Error) -> Self {
        Self::IO(value)
    }
}

impl From<toml::de::Error> for StoreError {
    fn from(value: toml::de::Error) -> Self {
        Self::TomlDeserialize(value)
    }
}

impl From<toml::ser::Error> for StoreError {
    fn from(value: toml::ser::Error) -> Self {
        Self::TomlSerialize(value)
    }
}

impl From<serde_json::Error> for StoreError {
    fn from(value: serde_json::Error) -> Self {
        Self::Serde(value)
    }
}
