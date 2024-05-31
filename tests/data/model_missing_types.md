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
  - Type: string
  - Term: schema:hello
  - XML: @name
- number
  - Type: float
  - Term: schema:one
  - XML: @number
- test2
  - Term: schema:something
  - XML: SomeTest2
- ontology
  - Type: Ontology
