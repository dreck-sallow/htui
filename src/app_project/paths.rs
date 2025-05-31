use std::{
    fs,
    path::{Path, PathBuf},
};

use directories::ProjectDirs;

pub trait Paths {
    /// Get the store folder path for this project
    fn store_folder(&self) -> PathBuf;

    /// Get the config folder path for this project
    fn config_folder(&self) -> PathBuf;
}

pub struct ProjectPaths {
    dirs: ProjectDirs,
}

impl ProjectPaths {
    pub fn new() -> Self {
        Self {
            dirs: ProjectDirs::from(
                Self::PROJECT_INFO.0,
                Self::PROJECT_INFO.1,
                Self::PROJECT_INFO.2,
            )
            .unwrap(),
        }
    }

    const STORE_FOLDER_NAME: &str = "store";
    const PROJECT_INFO: (&str, &str, &str) = ("com", "dreck", env!("CARGO_PKG_NAME"));
}

impl Paths for ProjectPaths {
    fn store_folder(&self) -> PathBuf {
        ensure_path(self.dirs.data_local_dir().join(Self::STORE_FOLDER_NAME)).unwrap()
    }

    fn config_folder(&self) -> PathBuf {
        ensure_path(self.dirs.config_local_dir()).unwrap()
    }
}

/// Ensure a path exists returning the path as `PathBuf` if exits
/// end create the file/folder when no already exists
pub fn ensure_path<P: AsRef<Path>>(path: P) -> Option<PathBuf> {
    let path = path.as_ref();

    if !path.exists() {
        if path.extension().is_some() {
            return fs::create_dir_all(path.parent().unwrap())
                .and_then(|_| fs::File::create(path))
                .ok()
                .map(|_| path.to_path_buf());
        } else {
            return fs::create_dir_all(path).ok().map(|_| path.to_path_buf());
        }
    }

    Some(path.to_path_buf())
}
