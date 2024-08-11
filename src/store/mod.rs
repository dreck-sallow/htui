use std::{io::Result, path::Path};

mod entity;
use entity::Project;
use tokio::fs;

pub async fn load_data(path: impl AsRef<Path>) -> Result<Project> {
    let res = fs::read_to_string(path).await?;
    let project: Project = serde_json::from_slice(res.as_bytes()).unwrap();
    Ok(project)
}

pub async fn save_data(path: impl AsRef<Path>, project: Project) -> Result<()> {
    let serialized = serde_json::ser::to_string(&project).unwrap();
    Ok(fs::write(path, serialized).await?)
}
