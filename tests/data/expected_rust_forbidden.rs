//! This file contains Rust struct definitions with serde serialization.
//!
//! WARNING: This is an auto-generated file.
//! Do not edit directly - any changes will be overwritten.

use derive_builder::Builder;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use derivative::Derivative;

//
// Type definitions
//
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Builder, Derivative)]
#[derivative(Default)]
#[serde(default)]
#[allow(non_snake_case)]
pub struct Test {
    /// enumeration
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(into))]
    #[derivative(Default)]
    pub enumeration: Option<SomeEnum>,
}

//
// Enum definitions
//
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default, PartialEq, Eq)]
pub enum SomeEnum {
    #[default]
    #[serde(rename = "yield")]
    Yield_,
}