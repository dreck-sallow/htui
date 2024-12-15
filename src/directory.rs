use std::{env, path::PathBuf};

use directories::ProjectDirs;

#[derive(Debug)]
pub enum DirectoryError {
    NotFounPath,
}

pub struct Directory {
    qualifier: &'static str,
    org: &'static str,
    app_name: &'static str,
}

impl Directory {
    pub fn new(qualifier: &'static str, org: &'static str, app_name: &'static str) -> Self {
        Self {
            qualifier,
            org,
            app_name,
        }
    }

    pub fn config_dir(&self) -> Result<PathBuf, DirectoryError> {
        if let Ok(path) = env::var("HTUI_CONFIG") {
            Ok(PathBuf::from(path))
        } else {
            return match ProjectDirs::from(&self.qualifier, &self.org, &self.app_name) {
                Some(p) => Ok(p.config_local_dir().to_path_buf()),
                None => Err(DirectoryError::NotFounPath),
            };
        }
    }

    pub fn config_file_path(&self) -> Result<PathBuf, DirectoryError> {
        Ok(self.config_dir()?.join("config.toml"))
    }
}
