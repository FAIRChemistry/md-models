---
repo: "https://www.github.com/my/repo/"
prefix: "tst"
---

### Test

- **name**
  - Type: Identifier
  - Description: A test description
- number
  - Type: float
  - Minimum: 0
- array_valued
  - Type: [Test2](#test2)[]
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
  - XML: name
- number
  - Type: float
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
