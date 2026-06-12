---
id-field: true
iri: "https://www.github.com/my/repo/"
prefix: "tst"
prefixes:
  schema: http://schema.org/
nsmap:
  tst: http://example.com/test/
---

### Configuration

A configuration object whose attributes use dashes in their names.

- name
  - Type: string
  - Description: The plain name of the configuration.
- coupling-scheme
  - Type: string
  - Description: A dashed scalar attribute.
- max-iterations
  - Type: integer
  - Description: A dashed optional attribute.
- data-value
  - Type: float, string
  - Description: A dashed union-typed attribute.
- sub-items
  - Type: [Item](#item)[]
  - Description: A dashed list of nested objects.

### Item

- item-name
  - Type: string
  - Description: A dashed attribute on the nested object.
