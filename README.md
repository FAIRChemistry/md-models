
# MD-Models

![Build Status](https://github.com/JR-1991/sdrdm.rs/actions/workflows/test.yml/badge.svg)

Welcome to Markdown Models (MD-Models), a powerful framework for research data management that prioritizes flexibility and efficiency.

With an adaptable markdown-based schema language, MD-Models automatically generates schemas and programming language representations. This markdown schema forms the foundation for object-oriented models, enabling seamless cross-format compatibility and simplifying modifications to data structures.

Check out the [documentation](https://fairchemistry.github.io/md-models/) for more information.

### Example

The schema syntax uses Markdown to define data models in a clear and structured way. Each object is introduced with a header, followed by its attributes. Attributes are described with their type, a brief explanation, and optional metadata like terms. Nested or related objects are represented using array types or references to other objects.

```markdown
---
prefixes:
  schema: http://schema.org/
---

### Person

- **name**
  - Type: string
  - Description: The name of the person
  - Term: schema:name
- age
  - Type: integer
  - Description: The age of the person
  - Term: schema:age
- addresses
  - Type: Address[]
  - Description: The address of the person

### Address

- street
  - Type: string
  - Description: The street of the address
```

## Installation

In order to install the command line tool, you can use the following command:

```bash
git clone https://github.com/JR-1991/md-models
cd md-models
cargo install --path .
```

## Command line usage

The command line tool can be used to convert markdown files to various formats. The following command will convert a markdown file to Python code:

```bash
md-models convert -i model.md -o lib.py -l python-dataclass
```

This will read the input file `model.md` and write the output to `lib.py` using the Python dataclass template. Alternatively, you can also pass a URL as input to fetch the model remotely. For an overview of all available templates, you can use the following command:

```bash
md-models --help
```

## Available templates

The following templates are available:

- `python-dataclass`: Python dataclass implementation with JSON-LD support
- `python-pydantic`: PyDantic implementation with JSON-LD support
- `python-pydantic-xml`: PyDantic implementation with XML support
- `xml-schema`: XML schema definition
- `json-schema`: JSON schema definition
- `shacl`: SHACL shapes definition
- `shex`: ShEx shapes definition

## Development

This project uses GitHub Actions for continuous integration. The tests can be run using the following command:

```bash
cargo test
cargo clippy
```
