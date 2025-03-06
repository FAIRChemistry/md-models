---

id-field: true
repo: "https://www.github.com/my/repo/"
prefix: "tst"
prefixes:
  schema: http://schema.org/
nsmap:
  tst: http://example.com/test/
---

### TestGorm

Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.

- __name__
  - Type: Identifier
  - Term: schema:hello
  - Description: The name of the test. This is a unique identifier 
    that helps track individual test cases across the system. 
    It should be descriptive and follow the standard naming conventions.
  - XML: @name
- number
  - Type: float
  - Term: schema:one
  - XML: @number
  - Default: 1.0
- test2_multiple
  - Type: [Test2](#test2)[]
  - Term: schema:something
- test2_single
  - Type: [Test2](#test2)
  - Term: schema:something
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
  - PrimaryKey: true

## Enumerations

### Ontology

Ontology endpoints for different types of sequences.

```
GO = "https://amigo.geneontology.org/amigo/term/"
SIO = "http://semanticscience.org/resource/"
ECO = "https://www.evidenceontology.org/term/"
```
