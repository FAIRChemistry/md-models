# Command Line Interface

The MD-Models command-line interface (CLI) provides a comprehensive set of tools for working with markdown data models. It enables you to validate models, generate code in multiple formats, extract data using AI, and automate workflows through pipelines.

## Installation

Install the MD-Models CLI using Cargo:

```bash
cargo install mdmodels
```

Once installed, you can use the `md-models` command from anywhere in your terminal.

## Available Commands

The MD-Models CLI provides the following commands:

### `validate`

Validates markdown model files for structural integrity, naming conventions, and type consistency.

**Quick start:**

```bash
md-models validate -i model.md
```

**Use cases:**
- Check models before code generation
- Validate models in CI/CD pipelines
- Ensure models meet naming and structure requirements

**Learn more:** [Schema Validation](schema_validation.md)

---

### `convert`

Converts markdown models to various output formats including programming languages, schema definitions, API specifications, and documentation.

**Quick start:**

```bash
# Generate Python Pydantic models
md-models convert -i model.md -t python-pydantic -o models.py

# Generate JSON Schema
md-models convert -i model.md -t json-schema -r Document -o schema.json
```

**Use cases:**
- Generate type-safe code for your application
- Create API schemas and specifications
- Produce documentation from models
- Export to semantic web formats

**Learn more:** [Code Generation](generation.md)

**Available formats:** See [Exporters](../exporters/exporters.md) for a complete list of supported templates.

---

### `pipeline`

Generates multiple output files from one or more models using a TOML configuration file. Ideal for automating code generation workflows.

**Quick start:**

```bash
md-models pipeline -i pipeline.toml
```

**Use cases:**
- Generate multiple formats in one command
- Automate code generation for entire projects
- Maintain consistency across generated outputs
- Integrate with CI/CD workflows

**Learn more:** [Pipelines](pipelines.md)

---

### `extract`

Uses Large Language Models (LLMs) to extract structured data from unstructured text based on your data model.

**Quick start:**

```bash
md-models extract -m model.md -i text.txt -o output.json
```

**Use cases:**
- Extract structured data from documents
- Parse unstructured text into typed objects
- Convert legacy data formats to structured JSON
- Automate data entry tasks

**Learn more:** [Large Language Models](llm.md)

---

### `dataset validate`

Validates JSON datasets against markdown models to ensure data conforms to the model structure.

**Quick start:**

```bash
md-models dataset validate -i data.json -m model.md
```

**Use cases:**
- Validate API request/response data
- Check data quality in ETL pipelines
- Ensure data consistency before processing
- Validate user-submitted data

---

## Common Workflows

### Development Workflow

```bash
# 1. Validate your model
md-models validate -i model.md

# 2. Generate code for your application
md-models convert -i model.md -t python-pydantic -o models.py

# 3. Generate API schemas
md-models convert -i model.md -t json-schema -r Document -o schema.json
md-models convert -i model.md -t graphql -o schema.graphql
```

### Automated Pipeline Workflow

```bash
# Use a pipeline configuration to generate everything at once
md-models pipeline -i pipeline.toml
```

### Data Extraction Workflow

```bash
# Extract structured data from unstructured text
md-models extract -m model.md -i document.txt -o extracted.json

# Validate the extracted data
md-models dataset validate -i extracted.json -m model.md
```

## Input Sources

All commands that accept input files support:

- **Local file paths**: `md-models validate -i model.md`
- **Remote URLs**: `md-models validate -i https://example.com/model.md`

MD-Models automatically detects whether the input is a URL (starts with `http`/`https`) or a local file path.

## Getting Help

Get help for any command using the `--help` flag:

```bash
# General help
md-models --help

# Command-specific help
md-models convert --help
md-models validate --help
md-models pipeline --help
```

## Command Reference

| Command | Purpose | Documentation |
|---------|---------|---------------|
| `validate` | Validate model structure and syntax | [Schema Validation](schema_validation.md) |
| `convert` | Generate code and schemas | [Code Generation](generation.md) |
| `pipeline` | Batch generation from config | [Pipelines](pipelines.md) |
| `extract` | LLM-powered data extraction | [Large Language Models](llm.md) |
| `dataset validate` | Validate data against models | See `md-models dataset validate --help` |

## Next Steps

- **New to MD-Models?** Start with the [Quickstart](../quickstart.md) guide
- **Want to generate code?** Read [Code Generation](generation.md) for detailed usage
- **Need to validate models?** See [Schema Validation](schema_validation.md) for validation rules
- **Automating workflows?** Check out [Pipelines](pipelines.md) for batch processing
- **Working with AI?** Explore [Large Language Models](llm.md) for data extraction

For detailed information about available export formats and templates, see the [Exporters](../exporters/exporters.md) documentation.