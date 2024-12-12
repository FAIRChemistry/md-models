use std::path::PathBuf;

use reqwest::Url;

use crate::datamodel::DataModel;

/// Represents different types of models that can be used.
///
/// `ModelType` can be constructed from a local file path, a remote URL, or a `DataModel` instance.
pub enum ModelType {
    Path(PathBuf),
    Remote(Url),
    Model(DataModel),
}

impl TryFrom<PathBuf> for ModelType {
    type Error = Box<dyn std::error::Error>;

    /// Attempts to create a `ModelType` from a `PathBuf`.
    ///
    /// Returns an error if the path does not exist.
    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        if !path.exists() {
            return Err(Box::from("Path does not exist"));
        }
        Ok(Self::Path(path))
    }
}

impl TryFrom<Url> for ModelType {
    type Error = Box<dyn std::error::Error>;

    /// Creates a `ModelType` from a `Url`.
    fn try_from(url: Url) -> Result<Self, Self::Error> {
        Ok(Self::Remote(url))
    }
}

impl TryFrom<DataModel> for ModelType {
    type Error = Box<dyn std::error::Error>;

    /// Creates a `ModelType` from a `DataModel`.
    fn try_from(model: DataModel) -> Result<Self, Self::Error> {
        Ok(Self::Model(model))
    }
}
