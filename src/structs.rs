use std::path::PathBuf;

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Clone, Debug)]
pub struct Game {
    pub name: String,
    pub author: String,
    pub location: PathBuf,
}
