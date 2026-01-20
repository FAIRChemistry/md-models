# Pipelines

Pipelines provide a powerful way to generate multiple output files from one or more MD-Models files in a single command. Instead of running multiple `convert` commands manually, pipelines allow you to define a configuration file that specifies all the formats you want to generate, making it easy to automate code generation workflows and maintain consistency across your project.

## The Pipeline Command

The `pipeline` command reads a TOML configuration file and generates all specified output formats in one execution.

### Basic Usage

```bash
md-models pipeline -i <pipeline_config.toml>
```

**Example:**

```bash
md-models pipeline -i pipeline.toml
```

## Pipeline Configuration Format

Pipeline configurations are written in TOML format and consist of two main sections:

1. **`[meta]`**: Metadata and input file paths
2. **`[generate]`**: Output generation specifications

### Configuration Structure

```toml
[meta]
name = "My Project"
description = "Project data models"
paths = ["model1.md", "model2.md"]

[generate]
python-pydantic = { out = "models.py" }
json-schema = { out = "schema.json", root = "Document" }
graphql = { out = "schema.graphql" }
```

## Meta Section

The `[meta]` section defines metadata and input files for the pipeline.

### Fields

- **`name`** (optional): A name for the pipeline configuration
- **`description`** (optional): A description of what the pipeline generates
- **`paths`** (required): An array of paths to MD-Models markdown files

**Example:**

```toml
[meta]
name = "API Models"
description = "Generate API models and schemas"
paths = ["models/user.md", "models/product.md"]
```

### Path Resolution

Paths in the `paths` array are resolved relative to the pipeline configuration file's directory. For example, if your pipeline file is at `configs/pipeline.toml` and you specify `paths = ["../models/user.md"]`, the path will be resolved relative to `configs/`.

**Multiple Input Files:**

When multiple files are specified in `paths`, MD-Models will:

- **Merge mode** (default): Combine all models into a single unified model before generation
- **Per-spec mode**: Generate separate outputs for each input file (requires `per-spec = true`)

## Generate Section

The `[generate]` section defines what outputs to generate. Each key is a template name, and the value is a specification object.

### Basic Generation Specification

```toml
[generate]
python-pydantic = { out = "models.py" }
```

### Specification Fields

Each generation specification supports the following fields:

- **`out`** (required): Output file path or directory path
- **`root`** (optional): Root object name (required for JSON Schema, optional for JSON-LD)
- **`per-spec`** (optional): Boolean indicating whether to generate separate files per input (default: `false`)
- **`fname-case`** (optional): Case transformation for output filenames (`pascal`, `snake`, `kebab`, `camel`, `none`)
- **`description`** (optional): Description of this generation step
- **Template-specific options**: Additional options like `jsonld`, `gorm`, `astropy`, etc.

### Merge Mode (Default)

When `per-spec` is `false` or omitted, all input models are merged into a single unified model, and one output file is generated.

**Example:**

```toml
[meta]
paths = ["user.md", "product.md"]

[generate]
python-pydantic = { out = "all_models.py" }
json-schema = { out = "schema.json", root = "User" }
```

**What happens:**

- `user.md` and `product.md` are merged into one model
- A single `all_models.py` file is generated containing both User and Product classes
- A single `schema.json` file is generated with User as the root

### Per-Spec Mode

When `per-spec = true`, separate output files are generated for each input file. This requires using a wildcard (`*`) in the output path.

**Example:**

```toml
[meta]
paths = ["user.md", "product.md"]

[generate]
python-pydantic = { out = "models/*.py", per-spec = true }
json-schema = { out = "schemas/*.json", per-spec = true, root = "User" }
```

**What happens:**

- `user.md` generates `models/user.py` and `schemas/user.json`
- `product.md` generates `models/product.py` and `schemas/product.json`
- Each file contains only the objects from its corresponding input model

**Wildcard Requirements:**

When using `per-spec = true`, the output path **must** contain a wildcard (`*`). The wildcard will be replaced with the input filename (without extension).

**Valid wildcard examples:**

- `"models/*.py"` → `models/user.py`, `models/product.py`
- `"output/*_schema.json"` → `output/user_schema.json`, `output/product_schema.json`
- `"schemas/*"` → `schemas/user`, `schemas/product`

**Invalid (will cause error):**

- `"models/filename.py"` with `per-spec = true` → Error: must contain wildcard

### File Name Case Transformation

The `fname-case` option allows you to transform output filenames to different case conventions when using `per-spec` mode.

**Available cases:**

- **`pascal`**: PascalCase (e.g., `UserProfile.py`)
- **`snake`**: snake_case (e.g., `user_profile.py`)
- **`kebab`**: kebab-case (e.g., `user-profile.py`)
- **`camel`**: camelCase (e.g., `userProfile.py`)
- **`none`**: No transformation (default)

**Example:**

```toml
[generate]
python-pydantic = { 
    out = "models/*.py", 
    per-spec = true,
    fname-case = "snake"
}
```

If your input file is `UserProfile.md`, this will generate `models/user_profile.py` instead of `models/UserProfile.py`.

### Template-Specific Options

You can pass template-specific options in the generation specification, just like with the `convert` command's `-O` flag.

**Example:**

```toml
[generate]
rust = { out = "models.rs", jsonld = true }
golang = { out = "models.go", gorm = true, xml = true }
python-pydantic = { out = "models.py", astropy = true }
typescript-zod = { out = "schemas.ts", json-ld = true }
json-schema = { out = "schema.json", root = "Document", openai = true }
```

**Available options by template:**

- **Rust**: `jsonld`
- **Go**: `gorm`, `xml`
- **Python Pydantic**: `astropy`
- **TypeScript Zod**: `json-ld`
- **JSON Schema**: `openai`

Options are specified as boolean values (`true`/`false`) or as string values in the TOML configuration.

### Root Object Specification

For templates that require a root object (like JSON Schema and JSON-LD), you can specify it in the generation specification.

**Example:**

```toml
[generate]
json-schema = { out = "schema.json", root = "Document" }
json-ld = { out = "context.jsonld", root = "User" }
```

If `root` is not specified:

- **JSON Schema**: Will use the first object in the merged model
- **JSON-LD**: Will use the first object in the merged model

## Complete Examples

### Example 1: Single Model, Multiple Formats

Generate multiple formats from a single model file:

```toml
[meta]
name = "API Models"
description = "Generate API models and schemas"
paths = ["api/models.md"]

[generate]
python-pydantic = { out = "api/models.py" }
json-schema = { out = "api/schema.json", root = "Document" }
graphql = { out = "api/schema.graphql" }
protobuf = { out = "api/schema.proto" }
rust = { out = "api/models.rs", jsonld = true }
```

**Usage:**

```bash
md-models pipeline -i pipeline.toml
```

**Output:**

- `api/models.py` - Python Pydantic models
- `api/schema.json` - JSON Schema with Document as root
- `api/schema.graphql` - GraphQL schema
- `api/schema.proto` - Protobuf definitions
- `api/models.rs` - Rust structs with JSON-LD support

### Example 2: Multiple Models, Merged Output

Merge multiple model files and generate unified outputs:

```toml
[meta]
paths = ["models/user.md", "models/product.md", "models/order.md"]

[generate]
python-pydantic = { out = "lib/all_models.py" }
json-schema-all = { out = "schemas/" }
shacl = { out = "schemas/shapes.ttl" }
```

**What happens:**

- All three models are merged into one
- `all_models.py` contains User, Product, and Order classes
- `schemas/` directory contains separate JSON Schema files for each object type
- `shapes.ttl` contains SHACL shapes for all merged objects

### Example 3: Per-Spec Generation

Generate separate outputs for each input model:

```toml
[meta]
paths = ["user.md", "product.md"]

[generate]
python-pydantic = { 
    out = "models/*.py", 
    per-spec = true,
    fname-case = "snake"
}
json-schema = { 
    out = "schemas/*.json", 
    per-spec = true,
    root = "User"
}
xml-schema = { 
    out = "schemas/*.xsd", 
    per-spec = true
}
```

**What happens:**

- `user.md` → `models/user.py`, `schemas/user.json`, `schemas/user.xsd`
- `product.md` → `models/product.py`, `schemas/product.json`, `schemas/product.xsd`
- Each output file contains only the objects from its corresponding input

### Example 4: Complex Pipeline with Options

A comprehensive pipeline with various options and configurations:

```toml
[meta]
name = "Full Stack API"
description = "Generate models for frontend, backend, and documentation"
paths = ["api/models.md"]

[generate]
# Backend - Python
python-pydantic = { 
    out = "backend/models.py",
    description = "Python API models"
}

# Backend - Rust
rust = { 
    out = "backend/models.rs",
    jsonld = true
}

# Frontend - TypeScript
typescript-zod = { 
    out = "frontend/schemas.ts",
    json-ld = true
}

# API Schemas
json-schema = { 
    out = "api/schema.json",
    root = "Document",
    openai = true
}
graphql = { out = "api/schema.graphql" }
protobuf = { out = "api/schema.proto" }

# Documentation
mk-docs = { out = "docs/api.md" }

# Semantic Web
shacl = { out = "schemas/shapes.ttl" }
json-ld = { out = "schemas/context.jsonld", root = "Document" }
```

## Special Templates

### JSON Schema All

The `json-schema-all` template generates separate JSON Schema files for each object. The output must be a directory path.

**Merge mode:**

```toml
[generate]
json-schema-all = { out = "schemas/" }
```

Generates one `.json` file per object in the `schemas/` directory.

**Per-spec mode:**

```toml
[generate]
json-schema-all = { out = "schemas/", per-spec = true }
```

Generates separate directories for each input file, each containing JSON Schema files for that file's objects.

### MkDocs

The MkDocs template automatically disables navigation when in merge mode (unless explicitly enabled via `nav` option).

**Example:**

```toml
[generate]
mk-docs = { out = "docs.md" }  # Navigation disabled automatically
mk-docs = { out = "docs.md", nav = true }  # Navigation enabled
```

## Path Resolution

All paths in the pipeline configuration are resolved relative to the pipeline configuration file's directory:

- **Input paths** (`paths`): Relative to the pipeline file's directory
- **Output paths** (`out`): Relative to the pipeline file's directory

**Example:**

If your pipeline file is at `configs/pipeline.toml`:

```toml
[meta]
paths = ["../models/user.md"]  # Resolved as configs/../models/user.md

[generate]
python-pydantic = { out = "output/models.py" }  # Resolved as configs/output/models.py
```

## Error Handling

The pipeline command will:

- **Stop on first error**: If any generation step fails, the pipeline stops
- **Validate inputs**: Ensures all input files exist before processing
- **Create directories**: Automatically creates output directories if they don't exist
- **Report errors**: Provides clear error messages for missing files, invalid configurations, or generation failures

## Best Practices

1. **Use descriptive names**: Give your pipeline a meaningful name and description
2. **Organize output paths**: Use consistent directory structures for outputs
3. **Version control**: Include pipeline configuration files in version control
4. **Test incrementally**: Start with a few templates and add more as needed
5. **Use per-spec for modular models**: When models are independent, use `per-spec = true` for separate outputs
6. **Use merge for related models**: When models share types or should be combined, use merge mode (default)
7. **Document options**: Use the `description` field to document why specific options are used

## Integration with CI/CD

Pipelines are ideal for CI/CD workflows. You can:

- Generate all formats automatically on model changes
- Ensure consistency across all generated outputs
- Version control both models and generated code together
- Automate documentation generation

**Example CI/CD usage:**

```bash
# In your CI/CD pipeline
md-models validate -i models/*.md
md-models pipeline -i pipeline.toml
# Generated files are ready for deployment
```

For more information about individual templates and their options, see the [Exporters](../exporters/exporters.md) documentation.
