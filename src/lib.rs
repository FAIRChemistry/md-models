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

#[cfg(feature = "python")]
use crate::bindings::python;
#[cfg(feature = "python")]
use crate::exporters::Templates;
#[cfg(feature = "python")]
use pyo3::prelude::*;

pub mod attribute;
pub mod datamodel;
pub mod exporters;
pub mod object;
pub mod option;
pub mod pipeline;
pub mod tree;
pub mod validation;
pub mod xmltype;

pub mod prelude {
    pub use crate::datamodel::DataModel;
    pub use crate::exporters::Templates;
    pub use crate::validation::Validator;
}

pub mod json {
    mod datatype;
    pub mod export;
    pub mod schema;
    pub mod validation;
}

pub(crate) mod markdown {
    pub mod frontmatter;
    pub(crate) mod parser;
    pub(crate) mod position;
}

#[cfg(feature = "openai")]
pub mod llm {
    pub mod extraction;
    pub mod input;
}

pub mod bindings {
    #[cfg(feature = "python")]
    pub(crate) mod python;

    #[cfg(feature = "wasm")]
    pub(crate) mod wasm;
}

pub mod linkml {
    pub mod export;
    pub mod import;
    pub mod schema;
}

#[cfg(feature = "python")]
#[pymodule]
fn mdmodels_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_class::<python::DataModel>()?;
    m.add_class::<Templates>()?;
    Ok(())
}
