# Markdown Models

![Build Status](https://github.com/JR-1991/sdrdm.rs/actions/workflows/test.yml/badge.svg)

This is a markdown parser and converter for Rust that can be used to parse markdown data model files and convert them to various formats, schemes and programming languages.

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
- `python-sdrdm`: Python PyDantic implementation with multiple output formats
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
