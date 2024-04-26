# Software-driven Research Data Management (sdRDM)
## Markdown Data Model parser and converter for Rust

> [!IMPORTANT]
> This is a work in progress and does not cover all featurs yet, but it is already usable.

![Build Status](https://github.com/JR-1991/sdrdm.rs/actions/workflows/test.yml/badge.svg)

This is a markdown parser and converter for Rust that can be used to parse markdown data model files and convert them to different JSON schema and the sdRDM format used for code generation.

### Data model

Contains a list of objects that represent the data model
written in the markdown format

#### Examples

```rust
let model = DataModel::parse("data_model.md");

// Generate a JSON schema for the object named "ObjectName"
let json_schema = model.json_schema("ObjectName");

// Generate a JSON schema for all objects in the data model
// and save them to a file
model.json_schema_all("path/to/dir");

// Generate a SDRDM schema for the object named "ObjectName"
let sdrdm_schema = model.sdrdm_schema();
```

#### Fields

* `objects` - A list of objects

#### Methods

* `new` - Create a new data model
* `parse` - Parse a markdown file and create a data model
* `json_schema` - Generate a JSON schema from the data model
* `json_schema_all` - Generate JSON schemas for all objects in the data model
* `sdrdm_schema` - Generate a SDRDM schema from the data model
