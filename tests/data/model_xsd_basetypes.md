---
prefix: "tst"
repo: "https://www.github.com/my/repo/"
---

### Bases

Facets that do not apply to the attribute's base type. XML Schema rejects the
whole document if they are emitted, so they must be dropped.

- stringy
  - Type: string
  - Minimum: 0
- numeric
  - Type: integer
  - MinLength: 3
- reference
  - Type: [Child](#child)
  - MinLength: 2

### Child

- c
  - Type: string
