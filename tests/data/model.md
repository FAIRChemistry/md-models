---
id-field: true
prefixes:
  schema: http://schema.org/
nsmap:
  tst: http://example.com/test/
---

# Test

## Objects

### Test

- __name__
  - Type: string
  - Term: schema:hello
- number
  - Type: float
  - Term: schema:one
- test2
  - Type: Test2
  - Term: schema:something

### Test2

- names
  - Type: string[]
  - Term: schema:hello
- number
  - Type: float
  - Term: schema:one
