use std::path::{Path, PathBuf};

use directories::ProjectDirs;

pub struct ProjectPaths {
    dirs: ProjectDirs,
}

impl ProjectPaths {
    pub fn new() -> Self {
        Self {
            dirs: ProjectDirs::from("com", "dreck", env!("CARGO_PKG_NAME")).unwrap(),
        }
    }

    pub fn store_folder(&self) -> PathBuf {
        ensure_path(self.dirs.data_local_dir().join("store")).unwrap()
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
            return std::fs::create_dir_all(path.parent().unwrap())
                .and_then(|_| std::fs::File::create(path))
                .ok()
                .map(|_| path.to_path_buf());
        } else {
            return std::fs::create_dir_all(path)
                .ok()
                .map(|_| path.to_path_buf());
        }
    }

    Some(path.to_path_buf())
}
