use std::{
    fs,
    path::{Path, PathBuf},
};

use directories::ProjectDirs;

pub struct Paths {
    dirs: ProjectDirs,
    store_folder: &'static str,
}

impl Paths {
    pub fn new(store_folder: &'static str) -> Self {
        Self {
            dirs: ProjectDirs::from("com", "dreck", "htui").unwrap(),
            store_folder,
        }
    }

    pub fn store_folder(&self) -> PathBuf {
        ensure_path(self.dirs.data_local_dir().join(self.store_folder)).unwrap()
    }
}

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
