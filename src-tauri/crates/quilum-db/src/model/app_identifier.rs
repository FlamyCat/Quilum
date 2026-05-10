use serde::{Deserialize, Serialize};
use serde::{Deserializer, Serializer};
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

/// Platform-specific application identifier for blocking.
/// Serializes/deserializes to/from string for SurrealDB compatibility.
#[derive(Debug, Clone)]
pub enum AppIdentifier {
    Path(PathBuf),    // Windows/Linux: full exe path
    BundleId(String), // macOS: immutable bundle ID
}

impl Serialize for AppIdentifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            AppIdentifier::Path(p) => {
                // Serialize as a path string
                let s = p.to_string_lossy().to_string();
                serializer.serialize_str(&s)
            }
            AppIdentifier::BundleId(s) => serializer.serialize_str(s),
        }
    }
}

impl<'de> Deserialize<'de> for AppIdentifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        // Try to detect if it's a path (contains / or \ or .exe etc.)
        // Bundle IDs are reverse-DNS (contain dots but no slashes)
        if s.contains(std::path::MAIN_SEPARATOR) || (cfg!(windows) && s.contains('\\')) {
            Ok(AppIdentifier::Path(PathBuf::from(s)))
        } else if s.contains('/') {
            Ok(AppIdentifier::Path(PathBuf::from(s)))
        } else {
            Ok(AppIdentifier::BundleId(s))
        }
    }
}

// Manual Eq/Hash for AppIdentifier
impl PartialEq for AppIdentifier {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (AppIdentifier::Path(a), AppIdentifier::Path(b)) => a.eq(b),
            (AppIdentifier::BundleId(a), AppIdentifier::BundleId(b)) => a.eq(b),
            _ => false,
        }
    }
}
impl Eq for AppIdentifier {}

impl Hash for AppIdentifier {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            AppIdentifier::Path(p) => p.hash(state),
            AppIdentifier::BundleId(s) => s.hash(state),
        }
    }
}
