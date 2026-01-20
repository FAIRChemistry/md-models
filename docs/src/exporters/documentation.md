# Documentation

MD-Models can export your data models to documentation formats, making it easy to generate comprehensive, interactive documentation for your data structures.

## MkDocs

The MkDocs exporter generates Markdown documentation files formatted for use with [MkDocs](https://www.mkdocs.org/), a static site generator for project documentation. This exporter creates human-readable documentation that includes:

- Visual graph representation of your data model structure
- Detailed type definitions with all attributes and their properties
- Enumeration documentation with value mappings
- Cross-references between related types
- Ontology and prefix information

### Usage

To generate MkDocs documentation from your MD-Models file:

```bash
md-models convert -i <model> -t mk-docs
```

For example:

```bash
md-models convert -i model.md -t mk-docs -o documentation.md
```

### Generated Output

The MkDocs exporter generates a comprehensive documentation page that includes:

#### Visual Graph

An interactive Mermaid flowchart that visualizes:

- All object types and enumerations in your model
- Relationships between types (when one type references another)
- Clickable links to navigate to each type's documentation section

The graph is displayed in a collapsible quote block using MkDocs' `??? quote` syntax.

#### Ontologies Section

If your model includes ontology prefixes, they are listed with links to their namespace URIs. This helps users understand the semantic context of your data model.

#### Types Section

Each object in your model is documented with:

- **Type name**: The name of the object type
- **Description**: The docstring from your model
- **Attributes**: A detailed list of all attributes including:
  - Attribute name (marked with `*` if required)
  - Data type(s) with automatic linking to referenced types
  - Array notation (`list[Type]`) for multiple values
  - Attribute description/docstring
  - Default values (if specified)
  - Additional options (e.g., primary key, unique constraints)

Types are automatically cross-referenced - when an attribute references another type in your model, it becomes a clickable link to that type's section.

#### Enumerations Section

All enumerations are documented in table format showing:

- Enumeration name and description
- A table mapping each enum alias to its corresponding value

### Integration with MkDocs

The generated Markdown file can be directly included in your MkDocs site. To use it:

1. Add the generated file to your `docs/` directory
2. Include it in your `mkdocs.yml` navigation:

```yaml
nav:
  - Home: index.md
  - API Reference: documentation.md
```

1. The documentation will automatically render with:
   - Syntax highlighting for code blocks
   - Interactive Mermaid diagrams (requires `mkdocs-mermaid2-plugin`)
   - Cross-referenced links between types
   - Responsive tables for enumerations

### Configuration Options

The MkDocs exporter supports a `nav` configuration option. When set, the generated documentation will include navigation elements. Without it, the navigation is hidden by default, making it suitable for embedding in existing MkDocs sites.

---

## Documentation Format Comparison

| Format     | Output Format | Interactive Features | Best For                        |
| ---------- | ------------- | -------------------- | ------------------------------- |
| **MkDocs** | Markdown      | ✅ Diagrams, Links    | Project documentation, API docs |
