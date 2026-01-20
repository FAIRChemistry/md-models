# Code generation

MD-Models provides powerful code generation capabilities through the `convert` command, allowing you to transform your markdown data models into various formats including programming languages, schema definitions, API specifications, and documentation.

## The Convert Command

The `convert` command is the primary way to export your MD-Models data model to different formats. It reads your markdown model file and generates output in the specified template format.

### Basic Syntax

```bash
md-models convert -i <input> -t <template> [-o <output>] [-r <root>] [-O <options>]
```

### Command Parameters

#### Input (`-i` or `--input`)

Specifies the input markdown model file that contains your data model definition. The input can be provided in two ways:

- **Local file path**: Path to a markdown file on your local filesystem. Use this when working with models stored locally on your machine.

  ```bash
  md-models convert -i model.md -t python-pydantic
  ```
  
  This example reads the `model.md` file from the current directory and converts it to Python Pydantic models. The file path can be relative (like `model.md` or `../models/schema.md`) or absolute (like `/path/to/model.md`).

- **Remote URL**: URL to a markdown file hosted online. Use this to fetch and convert models directly from web repositories, GitHub, or other online sources.

  ```bash
  md-models convert -i https://example.com/model.md -t json-schema
  ```
  
  This example fetches the model from the specified URL and generates a JSON Schema. This is particularly useful when working with shared models or models hosted in version control systems like GitHub.

MD-Models automatically detects whether the input is a URL (starts with `http` or `https`) or a local file path, so you don't need to specify the input type explicitly.

#### Template (`-t` or `--template`)

Specifies the output format template. Available templates are organized into categories:

- **[Programming Languages](../exporters/languages.md)**: Generate type-safe code in Python, TypeScript, Rust, Go, and Julia with validation and serialization support
- **[Schema Languages](../exporters/schemas.md)**: Create validation schemas (JSON Schema, XML Schema, SHACL, ShEx, OWL) and semantic definitions (JSON-LD, LinkML)
- **[API Specifications](../exporters/apis.md)**: Generate API specification formats (GraphQL, Protobuf) for service contracts
- **[Documentation](../exporters/documentation.md)**: Produce documentation formats (MkDocs, Markdown) for your data models

See the [Exporters](../exporters/exporters.md) documentation for a complete list of available templates and their usage.

#### Output (`-o` or `--output`)

Specifies the destination file path where the generated output will be written. This parameter is optional but highly recommended for production use.

**Writing to a file:**

```bash
md-models convert -i model.md -t python-pydantic -o models.py
```

This example generates Python Pydantic models and writes them to `models.py`. The output file path can be relative or absolute. If the file already exists, it will be overwritten. The directory containing the output file will be created automatically if it doesn't exist.

**Printing to stdout:**

```bash
md-models convert -i model.md -t python-pydantic
```

When the output parameter is omitted, the generated content is printed directly to the terminal (stdout). This is useful for quick previews, piping to other commands, or when you want to inspect the output before saving it to a file. For example, you could pipe the output to `less` for paginated viewing: `md-models convert -i model.md -t python-pydantic | less`.

#### Root Object (`-r` or `--root`)

Specifies which object in your data model should be treated as the root or entry point for the generated output. This parameter is essential for certain export formats that need to know where to start traversing your model's object graph.

**When root object is required:**

- **JSON Schema**: The root object parameter is **required** for JSON Schema generation. JSON Schema needs to know which object represents the top-level structure of your data, as it generates a schema that validates documents starting from that root object.

  ```bash
  md-models convert -i model.md -t json-schema -r MyRootObject -o schema.json
  ```
  
  This example generates a JSON Schema where `MyRootObject` is the root. The schema will validate JSON documents that have `MyRootObject` as their top-level structure, and all referenced types will be included in the `$defs` section.

**When root object is optional:**

- **JSON-LD**: The root object parameter is optional. If not specified, MD-Models uses the first object defined in your model as the root.

  ```bash
  md-models convert -i model.md -t json-ld -r Document -o context.jsonld
  ```
  
  This example generates a JSON-LD context header specifically for the `Document` object. The context will include term definitions for all attributes and relationships starting from `Document`, making it suitable for serializing `Document` instances as JSON-LD.

**For other templates:** The root object parameter is typically ignored, as these templates generate code or schemas for all objects in your model rather than focusing on a specific root.

#### Options (`-O` or `--options`)

Passes template-specific configuration options to customize the generated output. These options enable additional features or modify the behavior of the template. Multiple options can be specified as a comma-separated list without spaces.

```bash
md-models convert -i model.md -t <template> -O option1,option2
```

**Available options by template:**

- **Rust** (`rust`):
  - `jsonld` - Adds JSON-LD header support to generated Rust structs, including `JsonLdHeader` with context management methods (`add_term`, `update_term`, `remove_term`). This enables semantic web integration for your Rust data structures.

- **TypeScript Zod** (`typescript-zod`):
  - `json-ld` - Includes JSON-LD schema definitions (`JsonLdSchema`, `JsonLdContextSchema`) in the generated TypeScript code, allowing you to validate and work with JSON-LD data structures.

- **Go** (`golang`):
  - `gorm` - Adds GORM (Go Object-Relational Mapping) tags to struct fields, enabling database integration. This includes primary key tags, foreign key relationships, many-to-many relationships, and JSON serializer tags for complex types.
  - `xml` - Adds XML serialization tags to struct fields, enabling XML marshaling/unmarshaling. Supports XML element names, attributes, and wrapped elements.

- **Python Pydantic** (`python-pydantic`):
  - `astropy` - Enables Astropy unit support for `UnitDefinition` types. This replaces standard `UnitDefinition` with `UnitDefinitionAnnot` and filters out unit-related objects that Astropy handles natively, making the generated code compatible with Astropy's unit system.

- **JSON Schema** (`json-schema`):
  - `openai` - Generates OpenAI-compatible JSON Schema by removing options from schema properties. OpenAI's function calling API doesn't support custom options, so this option ensures compatibility when using the schema with OpenAI's API.

## Exampless

```bash
# Generate Rust code with JSON-LD support
md-models convert -i model.md -t rust -O jsonld -o models.rs
```

This generates Rust structs with embedded JSON-LD header support. Each struct will include a `jsonld` field of type `Option<JsonLdHeader>`, along with helper methods for managing JSON-LD contexts. This is useful when you need to serialize your Rust data as JSON-LD for semantic web applications.

```bash
# Generate Go code with GORM tags
md-models convert -i model.md -t golang -O gorm -o models.go
```

This generates Go structs with GORM database tags. Fields marked as primary keys will have `gorm:"primaryKey"` tags, relationships will have foreign key tags, and arrays of objects will have many-to-many relationship tags. This enables direct database persistence using GORM.

```bash
# Generate Python Pydantic with Astropy support
md-models convert -i model.md -t python-pydantic -O astropy -o models.py
```

This generates Python Pydantic models optimized for use with Astropy's unit system. If your model includes `UnitDefinition` objects, they'll be replaced with Astropy-compatible annotations, making it easy to work with physical units in scientific computing applications.

## Export Categories

MD-Models supports exporting to multiple categories of formats:

- **[Programming Languages](../exporters/languages.md)**: Generate type-safe code with validation and serialization in Python, TypeScript, Rust, Go, and Julia
- **[Schema Languages](../exporters/schemas.md)**: Create validation schemas (JSON Schema, XML Schema) and semantic web formats (SHACL, ShEx, OWL, JSON-LD, LinkML)
- **[API Specifications](../exporters/apis.md)**: Generate API specification formats (GraphQL, Protobuf) for defining service contracts
- **[Documentation](../exporters/documentation.md)**: Produce documentation formats (MkDocs, Markdown) with interactive diagrams and cross-references

Each category includes multiple templates optimized for specific use cases. See the linked documentation pages for detailed information about available templates, features, and usage examples.

### Input from JSON Schema

MD-Models can also read JSON Schema files as input, allowing you to convert from JSON Schema to other formats. This enables workflows where you start with a JSON Schema and generate code or other schemas from it.

**Usage:**

```bash
# Convert JSON Schema to Python Pydantic
md-models convert -i schema.json -t python-pydantic -o models.py
```

The tool automatically detects JSON Schema files by parsing the file content. If the file contains valid JSON Schema syntax, MD-Models will parse it as a JSON Schema rather than a markdown model. This allows you to use JSON Schema as a source format and convert it to any supported output format, making it easy to migrate from JSON Schema-based workflows to MD-Models or generate code from existing JSON Schema definitions.

## Complete Examples

The following examples demonstrate common use cases for the `convert` command with detailed explanations:

### Generate Python Code with JSON-LD

```bash
md-models convert -i model.md -t python-pydantic -o models.py
```

**What this does:** Converts your markdown model to Python Pydantic classes with built-in JSON-LD support. The generated `models.py` file will contain:

- Pydantic model classes for each object in your model
- Type hints and runtime validation
- Automatic JSON serialization/deserialization
- JSON-LD fields (`@id`, `@type`, `@context`) for semantic web integration
- Helper methods for managing JSON-LD contexts

**Use case:** Ideal for Python web APIs (especially FastAPI), data validation pipelines, or applications that need semantic web integration.

### Generate JSON Schema for API Validation

```bash
md-models convert -i model.md -t json-schema -r Document -o api-schema.json
```

**What this does:** Generates a JSON Schema Draft 2020-12 compliant schema file with `Document` as the root object. The schema includes:

- Complete type definitions for `Document` and all referenced objects
- Validation rules (required fields, data types, constraints)
- Enumeration definitions
- All referenced types in the `$defs` section

**Use case:** Perfect for API documentation, request/response validation, form generation, or ensuring data consistency across services. The `-r Document` parameter specifies that `Document` is the root type that API consumers will send/receive.

### Generate Multiple Formats with Options

```bash
# Rust with JSON-LD support
md-models convert -i model.md -t rust -O jsonld -o models.rs
```

**What this does:** Generates Rust structs with serde serialization and JSON-LD header support. Each struct includes:

- Builder pattern support via `derive_builder`
- JSON-LD header field with context management
- Helper methods for managing JSON-LD contexts (`add_term`, `update_term`, `remove_term`)
- Default JSON-LD header functions for each object type

**Use case:** Ideal for Rust web servers, high-performance APIs, or systems programming applications that need semantic web capabilities.

```bash
# Go with GORM and XML support
md-models convert -i model.md -t golang -O gorm,xml -o models.go
```

**What this does:** Generates Go structs with both GORM database tags and XML serialization tags. The structs include:

- GORM tags for database relationships (primary keys, foreign keys, many-to-many)
- XML tags for XML marshaling/unmarshaling
- JSON tags for JSON serialization
- Custom marshaling for union types

**Use case:** Perfect for Go microservices that need both database persistence (via GORM) and XML data exchange (for SOAP services or legacy system integration).

```bash
# TypeScript Zod with JSON-LD
md-models convert -i model.md -t typescript-zod -O json-ld -o schemas.ts
```

**What this does:** Generates TypeScript Zod schemas with JSON-LD support. The output includes:

- Zod schema definitions with type inference
- Runtime validation functions
- JSON-LD schema types (`JsonLdSchema`, `JsonLdContextSchema`)
- TypeScript types inferred from the schemas

**Use case:** Excellent for TypeScript/JavaScript applications that need runtime validation with semantic web support, such as React applications, Node.js APIs, or TypeScript-based frontend frameworks.

### Generate from Remote Model

```bash
md-models convert -i https://raw.githubusercontent.com/user/repo/main/model.md -t graphql -o schema.graphql
```

**What this does:** Fetches a markdown model from a remote URL (in this case, a GitHub raw file) and generates a GraphQL Schema Definition Language (SDL) file. The generated schema includes:

- Type definitions for all objects
- Enum definitions
- Union types for multi-type attributes
- Query type with automatically generated query operations

**Use case:** Useful when working with shared models hosted in version control, or when you want to generate schemas from models maintained by other teams or in public repositories. This enables collaborative data modeling workflows where models are versioned and shared via Git.

For more detailed information about specific exporters, their features, and configuration options, see the [Exporters](../exporters/exporters.md) documentation.
