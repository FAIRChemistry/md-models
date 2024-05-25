---
id-field: true
repo: "https://www.github.com/my/repo"
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
  - XML: @name
- number
  - Type: float
  - Term: schema:one
  - XML: @number
- test2
  - Type: Test2[]
  - Term: schema:something
  - XML: SomeTest2

### Test2

- names
  - Type: string[]
  - Term: schema:hello
  - XML: name
- number
  - Type: float
  - Term: schema:one
  - XML: @number
