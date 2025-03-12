# Preamble

The preamble is the first section of your data model. It is used to provide metadata about the data model, such as the name, version, and author.

```markdown
---
id: my-data-model
prefix: md
repo: http://mdmodel.net/
prefixes:
  schema: http://schema.org/
nsmap:
  tst: http://example.com/test/
imports:
  common.md: common.md
---
```

## Frontmatter Keys

The frontmatter section of your MD-Models document supports several configuration keys that control how your data model is processed and interpreted. Here's a detailed explanation of each available key:

### `id`

- **Type**: String (Optional)
- **Description**: A unique identifier for your data model. This can be used to reference your model from other models or systems.
- **Example**: `id: my-data-model`

### `prefixes`

- **Type**: Map of String to String (Optional)
- **Description**: Defines namespace prefixes that can be used throughout your model to reference external vocabularies or schemas. This is particularly useful for semantic annotations.
- **Example**:
  ```yaml
  prefixes:
    schema: http://schema.org/
    foaf: http://xmlns.com/foaf/0.1/
  ```

### `nsmap`

- **Type**: Map of String to String (Optional)
- **Description**: Similar to prefixes, defines namespace mappings that can be used in your model. This is often used for XML-based formats or when integrating with systems that use namespaces.
- **Example**:
  ```yaml
  nsmap:
    tst: http://example.com/test/
    ex: http://example.org/
  ```

### `repo`

- **Type**: String
- **Default**: `http://mdmodel.net/`
- **Description**: Specifies the base repository URL for your model. This can be used to generate absolute URIs for your model elements.
- **Example**: `repo: https://github.com/myorg/myrepo/`

### `prefix`

- **Type**: String
- **Default**: `md`
- **Description**: Defines the default prefix to use for your model elements when generating URIs or qualified names.
- **Example**: `prefix: mymodel`

### `imports`

- **Type**: Map of String to String
- **Default**: Empty map
- **Description**: Specifies other models to import into your current model. The key is the alias or name to use for the import, and the value is the location of the model to import. The location can be either a local file path or a remote URL.
- **Example**:
  ```yaml
  imports:
    common: common.md
    external: https://example.com/models/external.md
  ```

## Import Types

The `imports` key supports two types of imports:

1. **Local Imports**: References to local files on your filesystem
   ```yaml
   imports:
     common: ./common/base.md
   ```

2. **Remote Imports**: References to models hosted on remote servers (URLs)
   ```yaml
   imports:
     external: https://example.com/models/external.md
   ```

When importing models, the definitions from the imported models become available in your current model, allowing you to reference and extend them. This is useful for creating modular and reusable data models.
