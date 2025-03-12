# MD-Models 🚀

[![Crates.io Version](https://img.shields.io/crates/v/mdmodels)](https://crates.io/crates/mdmodels)
[![NPM Version](https://img.shields.io/npm/v/mdmodels-core)](https://www.npmjs.com/package/mdmodels-core)
[![PyPI - Version](https://img.shields.io/pypi/v/mdmodels-core)](https://pypi.org/project/mdmodels-core/)
[![Build Status](https://github.com/JR-1991/sdrdm.rs/actions/workflows/test.yml/badge.svg)](https://github.com/JR-1991/sdrdm.rs/actions/workflows/test.yml)

*Welcome to Markdown Models (MD-Models)!* 📝 

We've created this framework to make research data management more intuitive and accessible while maintaining professional standards. Our approach uses markdown-based schema definitions to transform complex data modeling into something you'll actually enjoy working with.

The framework does the heavy lifting for you - automatically generating technical schemas and programming language implementations from your markdown files. This means you can focus on designing your data structures in a format that makes sense, while we handle the technical translations. ⚙️

## Core Philosophy 💡

We built MD-Models to solve a common frustration in data modeling: juggling multiple versions of the same information. Here's what typically happens in traditional approaches:

1. Technical Schema Definitions 📊
   - You need JSON Schema, XSD, ShEx, or SHACL
   - Each format has its own complexity
   - Changes need to be replicated across formats

2. Language-Specific Implementations 💻
   - Different programming languages need different implementations
   - Each requires maintenance and updates
   - Keeping everything in sync is challenging

3. Documentation 📚
   - Technical docs for developers
   - Simplified explanations for domain experts
   - API documentation that needs constant updates

Instead of dealing with all these separate pieces, MD-Models gives you one clear source of truth. Write it once, use it everywhere! ✨

Ready to see it in action? Check out our [book](https://fairchemistry.github.io/md-models/) for a deeper dive into the framework and [graph editor](https://mdmodels.vercel.app/?about) to get started.

### Schema Design 🎨

Our schema syntax makes the most of markdown's natural readability. Here's what you can do:

- Define objects with clear, descriptive headers
- Specify attributes with all the details you need
- Add rich descriptions that everyone can understand
- Include semantic annotations when you need them
- Define relationships between objects easily

We've designed this approach to work for everyone on your team - whether they're technical experts or domain specialists. You get all the precision you need for automatic code generation, while keeping things clear and approachable. 🤝

Here is an example of a markdown model definition:

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

Lets break down the example:

We define an object `Person` with two attributes: `name` and `age`. We also define an object `Address` with one attribute: `street`. An object can be defined as a list of attributes, which can be either primitive types, other objects, or lists of other objects.

Objects are defined by using the `###` header and a list of attributes. Attributes are defined by using the `-` prefix. The type of the attribute is specified after the `:`. The description of the attribute is specified after the `-`. The term of the attribute is specified after the `-`.

Attributes can hold any key-value pair as metadata. For instance, the `age` attribute has the following metadata:

```markdown
- age
  - Type: integer
  - Description: The age of the person
```

The `age` attribute is of type `integer` and has the following description: `The age of the person`. You could also add more metadata to the attribute, such as `minValue` and `maxValue` for JSON Schema. If your application needs more metadata, you can add it to the attribute as well - There are no restrictions on the metadata.

> [!NOTE]
> All JSON-Schema validation keywords are supported, except for `readOnly` and `writeOnly`.

### Large Language Model Integration 🤖

Our framework also supports large language model guided extraction of information from natural language text into a structured format. Typically you would use a JSON schema as an intermediate format for this or use specialized libraries such as [Instructor](https://github.com/jxnl/instructor) or [LangChain](https://github.com/langchain-ai/langchain) to accomplish this.

We have wrapped all of this functionality into a single command:

```bash
export OPENAI_API_KEY="sk-..."
md-models extract -i text.txt -m mymodel.md -o structured.json
```

This will read the input text file and extract the information into the structured format defined in the markdown model. The output will be written to the `structured.json` file. You can even pass an existing JSON dataset and let the LLM update the dataset with the new information. By utilizing JSON patch, we can ensure that the original dataset is kept intact and only the new information is added.

## Installation 🛠️

MD-Models is primarily a command line tool. In order to install the command line tool, you can use the following command:

```bash
git clone https://github.com/FAIRChemistry/md-models
cd md-models
cargo install --path .
```

Checkout our releases, where you can find pre-compiled binaries for the command line tool!

## Command line usage 📝

The command line tool can be used to convert markdown files to various formats. For instance, the following command will convert a markdown file to Python code:

```bash
md-models convert -i model.md -o lib.py -t python-dataclass
```

This will read the input file `model.md` and write the output to `lib.py` using the Python dataclass template. Alternatively, you can also pass a URL as input to fetch the model remotely.

Here is a list of all available sub commands:

- `convert`: Convert a markdown file to a specific format
- `validate`: Validate and check if a markdown file conforms our specification
- `pipeline`: Pipeline for generating multiple files
- `extract`: Large Language Model Extraction guided by a markdown model
- `dataset`: Validate a dataset against a markdown model

## Available templates

The following templates are available for the `convert` command:

- `python-dataclass`: Python dataclass implementation with JSON-LD support
- `python-pydantic`: PyDantic implementation with JSON-LD support
- `python-pydantic-xml`: PyDantic implementation with XML support
- `typescript`: TypeScript interface definitions with JSON-LD support
- `typescript-zod`: TypeScript Zod schema definitions
- `rust`: Rust struct definitions with serde support
- `golang`: Go struct definitions
- `julia`: Julia struct definitions
- `protobuf`: Protocol Buffer schema definition
- `graphql`: GraphQL schema definition
- `xml-schema`: XML schema definition
- `json-schema`: JSON schema definition
- `json-schema-all`: Multiple JSON schema definitions (one per object)
- `shacl`: SHACL shapes definition
- `shex`: ShEx shapes definition
- `compact-markdown`: Compact markdown representation
- `mkdocs`: MkDocs documentation format
- `linkml`: LinkML schema definition

## Installation options 📦

We've made our core Rust library incredibly versatile by compiling it to both Python and WebAssembly! This means you can use our model conversion tools not just from the command line, but directly in your Python applications or web browsers.

We provide several packages to make integration seamless:

- **[Core Python Package](https://pypi.org/project/mdmodels-core/)**: Install via pip:
  ```bash
  # Mainly used to access the core functionality of the library
  pip install mdmodels-core
  ```

- **[Python Package](https://github.com/FAIRChemistry/py-mdmodels/tree/master)**: Install via pip:
  ```bash
  # Provides in-memory data models, database support, LLM support, etc.
  pip install mdmodels
  ```

- **[NPM Package](https://www.npmjs.com/package/mdmodels-core)**: Install via npm:
  ```bash
  # Mainly used to access the core functionality of the library
  npm install mdmodels-core
  ```

## Development 🔧

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