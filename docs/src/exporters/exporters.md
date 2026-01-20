# Exporters

MD-Models provides a comprehensive set of exporters that allow you to convert your data models into various formats, including programming languages, schema definitions, API specifications, and documentation formats. This enables you to:

- **Generate type-safe code** in multiple programming languages
- **Create validation schemas** for data validation
- **Build API specifications** for service contracts
- **Produce documentation** for your data models
- **Enable semantic web integration** with RDF and JSON-LD support

## Export Categories

MD-Models exporters are organized into several categories:

### [Programming Languages](languages.md)

Generate type-safe code and data structures in various programming languages:

- **Python**: Dataclasses, Pydantic models (with XML support)
- **TypeScript**: io-ts and Zod schemas
- **Rust**: Structs with serde serialization
- **Go**: Structs with GORM and XML support
- **Julia**: Type definitions with JSON serialization

[View all programming language exporters →](languages.md)

### [Schema Languages](schemas.md)

Create validation schemas and semantic definitions:

- **JSON Schema**: JSON data validation (Draft 2020-12)
- **XML Schema (XSD)**: XML document validation
- **SHACL & ShEx**: RDF graph validation
- **OWL**: Web Ontology Language for knowledge graphs
- **JSON-LD**: Linked data context generation
- **LinkML**: Multi-format schema generation

[View all schema language exporters →](schemas.md)

### [API Specifications](apis.md)

Generate API specification formats:

- **GraphQL**: GraphQL Schema Definition Language (SDL)
- **Protobuf**: Protocol Buffer message definitions

[View all API specification exporters →](apis.md)

### [Documentation](documentation.md)

Generate documentation formats:

- **MkDocs**: Markdown documentation with interactive diagrams

[View all documentation exporters →](documentation.md)

## Quick Start

To export your MD-Models file to any format, use the `convert` command:

```bash
md-models convert -i <model> -t <template> -o <output>
```

**Example:**

```bash
# Generate Python Pydantic models
md-models convert -i model.md -t python-pydantic -o models.py

# Generate JSON Schema
md-models convert -i model.md -t json-schema -r MyObject -o schema.json

# Generate GraphQL schema
md-models convert -i model.md -t graphql -o schema.graphql
```

## Template Options

Many exporters support additional options via the `-O` or `--options` flag:

```bash
md-models convert -i <model> -t <template> -O option1,option2
```

Common options include:
- `jsonld` / `json-ld`: Enable JSON-LD support (Rust, TypeScript Zod)
- `gorm`: Enable GORM tags (Go)
- `xml`: Enable XML serialization (Go)
- `astropy`: Enable Astropy unit support (Python Pydantic)
- `openai`: OpenAI-compatible schema (JSON Schema)

See the individual exporter pages for specific options available for each template.

## Choosing the Right Exporter

| Use Case | Recommended Exporters |
|----------|----------------------|
| **API Development** | GraphQL, Protobuf, JSON Schema |
| **Data Validation** | JSON Schema, XML Schema, SHACL, ShEx |
| **Code Generation** | Python Pydantic, TypeScript Zod, Rust, Go |
| **Semantic Web** | OWL, SHACL, ShEx, JSON-LD |
| **Documentation** | MkDocs |
| **Multi-format Support** | LinkML |

For detailed information about each exporter, including features, usage examples, and configuration options, visit the dedicated pages linked above.