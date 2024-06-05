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

- <details>
  <summary>name</summary>

  - Type: string
  - Term: schema:hello

  </details>
- <details>
  <summary>number</summary>

  - Type: float
  - Term: schema:one

  </details>
- <details>
  <summary>test2</summary>

  - Type: [Test2](#test2)
  - Term: schema:something

  </details>
- <details>
  <summary>ontology</summary>

  - Type: Ontology

  </details>

### Test2

- <details>
  <summary>names</summary>

  - Type: string
  - Term: schema:hello

  </details>
- <details>
  <summary>number</summary>

  - Type: float
  - Term: schema:one
  - minimum: 0

  </details>

## Enumerations

### Ontology

```
ECO = https://www.evidenceontology.org/term/
GO = https://amigo.geneontology.org/amigo/term/
SIO = http://semanticscience.org/resource/
```
