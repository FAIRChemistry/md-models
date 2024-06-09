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

- __name__
  - Type: Identifier
  - Term: schema:hello
  - Description: The name of the test.
  - XML: @name
- number
  - Type: float
  - Term: schema:one
  - XML: @number
  - Default: 1.0
- test2
  - Type: [Test2](#test2)[]
  - Term: schema:something
  - XML: SomeTest2
- ontology
  - Type: Ontology

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
