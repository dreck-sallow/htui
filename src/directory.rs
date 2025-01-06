use std::{env, fs, io, path::PathBuf};

use directories::ProjectDirs;

pub const ENV_HTUI_CONFIG: &str = "HTUI_CONFIG";
pub const HTUI_CONFIG_FILE: &str = "config.toml";

pub const ENV_HTUI_STORE: &str = "HTUI_STORE";
pub const HTUI_STORE_FILE: &str = "store.toml";

#[derive(Debug)]
pub enum DirectoryV2Error {
    NotProjectDirs,
    IO(io::Error),
}

pub struct DirectoryV2 {
    // dirs: ProjectDirs,
    config_dir: PathBuf,
    data_dir: PathBuf,
}

impl DirectoryV2 {
    pub fn from(qualifier: &str, org: &str, name: &str) -> Result<Self, DirectoryV2Error> {
        if let Some(project_dirs) = ProjectDirs::from(qualifier, org, name) {
            let config_dir = env::var(ENV_HTUI_CONFIG)
                .map(PathBuf::from)
                .unwrap_or(project_dirs.config_local_dir().to_path_buf());

            let data_dir = env::var(ENV_HTUI_STORE)
                .map(PathBuf::from)
                .unwrap_or(project_dirs.data_local_dir().to_path_buf());

            if !config_dir.exists() {
                fs::create_dir_all(&config_dir)?;
            }

            if !data_dir.exists() {
                fs::create_dir_all(&data_dir)?;
            }

            return Ok(Self {
                // dirs: project_dirs,
                config_dir,
                data_dir,
            });
        }

        Err(DirectoryV2Error::NotProjectDirs)
    }

    pub fn config_path(&self) -> &PathBuf {
        &self.config_dir
    }

    pub fn data_path(&self) -> &PathBuf {
        &self.data_dir
    }

    pub fn config_file_path(&self) -> io::Result<PathBuf> {
        let path = self.config_dir.join(HTUI_CONFIG_FILE);

        if !path.exists() {
            fs::File::create_new(&path)?;
        }

        Ok(path)
    }

    pub fn data_file_path(&self) -> io::Result<PathBuf> {
        let path = self.data_dir.join(HTUI_STORE_FILE);

        if !path.exists() {
            fs::File::create_new(&path)?;
        }

        Ok(path)
    }
}

impl From<io::Error> for DirectoryV2Error {
    fn from(value: io::Error) -> Self {
        DirectoryV2Error::IO(value)
    }
}
