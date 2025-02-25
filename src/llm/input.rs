/*
 * Copyright (c) 2025 Jan Range
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
 * THE SOFTWARE.
 *
 */

use std::path::PathBuf;

use reqwest::Url;

use crate::datamodel::DataModel;

/// Represents different types of models that can be used.
///
/// `ModelType` can be constructed from a local file path, a remote URL, or a `DataModel` instance.
#[allow(clippy::large_enum_variant)]
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
