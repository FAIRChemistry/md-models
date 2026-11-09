---
id: required-test
prefixes:
  schema: http://schema.org/
nsmap:
  tst: http://example.com/test/
---

### Sample

A model whose required attributes have no default value. The Python
generators must mark them as required rather than emit a default.

- **sample_id**
  - Type: Identifier
  - Description: Required identifier without a default.
- **name**
  - Type: string
  - Description: Required element without a default.
- **code**
  - Type: string
  - Description: Required XML attribute without a default.
  - XML: @code
- **count**
  - Type: integer
  - Description: Required element with a default.
  - Default: 3
- note
  - Type: string
  - Description: Optional element.
- parts
  - Type: Part[]
  - Description: Optional list.

### Part

- **label**
  - Type: string
  - Description: Required element in a nested object.
