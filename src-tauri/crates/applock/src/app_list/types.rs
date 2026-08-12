use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize, Serializer};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    #[serde(
        serialize_with = "serialize_path",
        deserialize_with = "deserialize_path"
    )]
    pub identifier: PathBuf,
    pub display_name: String,
}

impl AppInfo {
    pub fn new(identifier: PathBuf, display_name: String) -> Self {
        Self {
            identifier,
            display_name,
        }
    }
}

fn serialize_path<S>(path: &Path, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&path.to_string_lossy())
}

fn deserialize_path<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
where
    D: serde::Deserializer<'de>,
{
    String::deserialize(deserializer).map(PathBuf::from)
}
