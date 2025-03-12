# Quickstart

In order to get started with MD-Models, you can follow the steps below.

## Installation

In order to install the command line tool, you can use the following command:

```bash
cargo install mdmodels
```

### Writing your first MD-Models file

MD-Models files can be written in any editor that supports markdown. In the following is a list of recommended editors:

- [Cursor](https://www.cursor.com/)
- [VSCode](https://code.visualstudio.com/)
- [Obsidian](https://obsidian.md/)
- [Notion](https://www.notion.so/)

We also provide a web-editor at [mdmodels.vercel.app](https://mdmodels.vercel.app) that can be used to write and validate MD-Models files. This editor not only features a syntax higlighted editor, but also ...

- Live preview of the rendered MD-Models file
- Graph editor to visualize the relationships between objects
- Automatic validation of the MD-Models file
- Export to various schema languages and programming languages

## Packages

The main Rust crate is compiled to Python and WebAssembly, allowing the usage beyond the command line tool. These are the main packages:

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

## Examples

The following projects are examples of how to use MD-Models in practice:

- [EnzymeML](https://github.com/enzymeml/enzymeml-specifications)
- [STRENDA Biocatalysis](https://github.com/STRENDA-Biocatalysis/STRENDA-Biocatalysis)
