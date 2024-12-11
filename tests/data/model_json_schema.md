---
id-field: true
repo: "https://www.github.com/my/repo/"
prefix: "tst"
prefixes:
  schema: http://schema.org/
nsmap:
  tst: http://example.com/test/
---

### Test

- **name**
  - Type: Identifier
  - Term: schema:hello
  - Description: A test description
- number
  - Type: float
  - Term: schema:one
- array_valued
  - Type: [Test2](#test2)[]
  - Term: schema:something
- single_valued
  - Type: [Test2](#test2)
- ontology
  - Type: Ontology
- multiple_types
  - Type: float, Test2
- multiple_types_array
  - Type: float, Test2
  - Multiple: true

### Test2

- names
  - Type: string[]
  - Term: schema:hello
  - XML: name
- number
  - Type: float
  - Term: schema:one
  - XML: @number
  - Minimum: 0

## Enumerations

### Ontology

Ontology endpoints for different types of sequences.

```
GO = "https://amigo.geneontology.org/amigo/term/"
SIO = "http://semanticscience.org/resource/"
ECO = "https://www.evidenceontology.org/term/"
```
