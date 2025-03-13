# Property Options

When defining properties in your data model, you can apply various options to control their behavior, validation, and representation. These options are defined using the `- option: value` syntax. In the following sections, we will look at the different options that are available.

## General Options

| Option        | Description                                | Example                                  |
| ------------- | ------------------------------------------ | ---------------------------------------- |
| `description` | Provides a description for the property    | `- description "The name of the person"` |
| `example`     | Provides an example value for the property | `- example "John Doe"`                   |

## JSON Schema Validation Options

These options map to standard JSON Schema validation constraints, allowing you to enforce data integrity and validation rules in your models. When you use these options, they will be translated into corresponding JSON Schema properties during schema generation, ensuring that your data adheres to the specified constraints. This provides a standardized way to validate data across different systems and implementations that support JSON Schema.

| Option               | Description                                                              | Example                       |
| -------------------- | ------------------------------------------------------------------------ | ----------------------------- |
| `minimum`            | Specifies the minimum value for a numeric property                       | `- minimum: 0`                |
| `maximum`            | Specifies the maximum value for a numeric property                       | `- maximum: 100`              |
| `minitems`           | Specifies the minimum number of items for an array property              | `- minitems: 1`               |
| `maxitems`           | Specifies the maximum number of items for an array property              | `- maxitems: 10`              |
| `minlength`          | Specifies the minimum length for a string property                       | `- minlength: 3`              |
| `maxlength`          | Specifies the maximum length for a string property                       | `- maxlength: 50`             |
| `pattern` or `regex` | Specifies a regular expression pattern that a string property must match | `- pattern: "^[a-zA-Z0-9]+$"` |
| `unique`             | Specifies whether array items must be unique                             | `- unique: true`              |
| `multipleof`         | Specifies that a numeric value must be a multiple of this number         | `- multipleof: 5`             |
| `exclusiveminimum`   | Specifies an exclusive minimum value for a numeric property              | `- exclusiveminimum: 0`       |
| `exclusivemaximum`   | Specifies an exclusive maximum value for a numeric property              | `- exclusivemaximum: 100`     |

## Format Options

The following options are used to define how the property should be represented in different formats.

| Option | Description                                                     | Example           |
| ------ | --------------------------------------------------------------- | ----------------- |
| `xml`  | Specifies that the property should be represented in XML format | `- xml: someName` |

### A note on the `xml` option

The `xml` option has multiple effects:

- `Element` will be set as an element in the XML Schema.
- `@Name` will be set as an attribute in the XML Schema.
- `someWrapper/Element` will wrap the element in a parent element called `someWrapper`.

## Semantic Options

The following options are used to define semantic annotations. Read more about semantic annotations in the [Semantics](./semantics.md) section.

| Option | Description                                         | Example               |
| ------ | --------------------------------------------------- | --------------------- |
| `term` | Specifies the term for the property in the ontology | `- term: schema:name` |

## SQL Database Options

Database options allow you to specify how properties should be represented in relational database systems. MD-Models supports the following options:

| Option | Description                                                   | Example               |
| ------ | ------------------------------------------------------------- | --------------------- |
| `pk`   | Indicates whether the property is a primary key in a database | `- primary key: true` |

## LinkML Specific Options

Options specific to the LinkML specification:

| Option        | Description                                   | Example               |
| ------------- | --------------------------------------------- | --------------------- |
| `readonly`    | Indicates whether the property is read-only   | `- readonly: true`    |
| `recommended` | Indicates whether the property is recommended | `- recommended: true` |

## Custom Options

You can also define custom options that aren't covered by the predefined ones:

```markdown
- name
  - MyKey: my value
```

## Example Usage

Here's how you might use these options in a data model:

```markdown
### Person (schema:object)

- id
  - type: string
  - primary key: true
  - description: The unique identifier for the person
- name
  - type: string
  - description: The name of the person
  - example: "John Doe"
- age
  - type: integer
  - description: The age of the person
  - minimum: 0
```

These options help to define constraints, provide validation rules, and give hints to code generators about how properties should be treated in the resulting applications and schemas.
