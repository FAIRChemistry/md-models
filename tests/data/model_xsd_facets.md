---
prefix: "tst"
repo: "https://www.github.com/my/repo/"
---

### Facets

Exercises every validation option XML Schema can express as a facet, plus the
options it cannot.

- **bounded**
  - Type: float
  - Minimum: 0
  - Maximum: 100
- exclusive
  - Type: float
  - ExclusiveMinimum: 0
  - ExclusiveMaximum: 100
- constrained
  - Type: string
  - MinLength: 1
  - MaxLength: 100
  - Pattern: ^[a-zA-Z0-9 ]+$
- tagged
  - Type: string[]
  - MinLength: 2
- union
  - Type: string, integer, boolean
- unionarray
  - Type: string, float
  - Multiple: true
- unsupported
  - Type: integer
  - MultipleOf: 2
  - Unique: true
