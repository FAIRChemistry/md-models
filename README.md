
# MD-Models

![Crates.io Version](https://img.shields.io/crates/v/mdmodels) ![NPM Version](https://img.shields.io/npm/v/mdmodels-core)
![PyPI - Version](https://img.shields.io/pypi/v/mdmodels-core)
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

## Installation options

The main Rust crate is compiled to Python and WebAssembly, allowing the usage beyond the command line tool. These are the main packages:

- **[Core Python Package](https://pypi.org/project/mdmodels-core/)**: Install via pip:
  ```bash
  # Mainly used to access the core functionality of the library
  pip install mdmodels-core
  ```

- **[Python Package](https://pypi.org/project/mdmodels/)**: Install via pip:
  ```bash
  # Provides in-memory data models, database support, LLM support, etc.
  pip install mdmodels
  ```

- **[NPM Package](https://www.npmjs.com/package/mdmodels-core)**: Install via npm:
  ```bash
  # Mainly used to access the core functionality of the library
  npm install mdmodels-core
  ```

## Development

This project uses GitHub Actions for continuous integration. The tests can be run using the following command:

```bash
cargo test
cargo clippy
```

### Using pre-commit hooks

This project uses [pre-commit](https://pre-commit.com/) to run the `rustfmt` and `clippy` commands on every commit. To install the pre-commit hooks, you can use the following command:

```bash
pip install pre-commit
pre-commit install
```

Once the pre-commit hooks are installed, they will run on every commit. This will ensure that the code is formatted and linted correctly. And the clippy CI will not complain about warnings.
