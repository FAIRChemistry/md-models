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

use serde::{Deserialize, Serialize};

/// Represents an Avro schema definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AvroSchema {
    /// The type of the schema
    pub schema_type: SchemaType,
    /// Optional name for named types (record, enum, fixed)
    pub name: Option<String>,
    /// Optional namespace for named types
    pub namespace: Option<String>,
    /// Optional documentation
    pub doc: Option<String>,
    /// Optional aliases for named types
    pub aliases: Option<Vec<String>>,
    /// Fields specific to the schema type
    pub type_fields: TypeFields,
}

/// Represents the specific type of an Avro schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SchemaType {
    Record,
    Enum,
    Array,
    Map,
    Union,
    Fixed,
    String,
    Bytes,
    Int,
    Long,
    Float,
    Double,
    Boolean,
    Null,
}

/// Contains the fields specific to each schema type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeFields {
    Record { fields: Vec<RecordField> },
    Enum { symbols: Vec<String> },
    Array { items: Box<AvroSchema> },
    Map { values: Box<AvroSchema> },
    Union { types: Vec<Box<AvroSchema>> },
    Fixed { size: usize },
    Primitive, // For primitive types that don't have additional fields
}

/// Represents a field in a record type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordField {
    pub name: String,
    pub doc: Option<String>,
    pub schema: Box<AvroSchema>,
    pub default: Option<serde_json::Value>,
    pub order: Option<Order>,
    pub aliases: Option<Vec<String>>,
}

/// Represents the sort order for a record field
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Order {
    Ascending,
    Descending,
    Ignore,
}

impl AvroSchema {
    /// Creates a new primitive schema
    pub fn primitive(schema_type: SchemaType) -> Self {
        AvroSchema {
            schema_type,
            name: None,
            namespace: None,
            doc: None,
            aliases: None,
            type_fields: TypeFields::Primitive,
        }
    }

    /// Creates a new record schema
    pub fn record<S: Into<String>>(name: S, fields: Vec<RecordField>) -> Self {
        AvroSchema {
            schema_type: SchemaType::Record,
            name: Some(name.into()),
            namespace: None,
            doc: None,
            aliases: None,
            type_fields: TypeFields::Record { fields },
        }
    }
}
