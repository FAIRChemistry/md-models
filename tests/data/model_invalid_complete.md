---
id-field: true
repo: "https://www.github.com/my/repo/"
prefix: "tst"
prefixes:
  schema: http://schema.org/
nsmap:
  tst: http://example.com/test/
---

### Test Object

- 1number
  - Type: string
- some name
  - Type: string
- undefined_type
  - Type: Undefined

### 1Test

- 1number
  - Type: string

### Duplicate

- value
  - Type: string

### Duplicate

- value
  - Type: string

### DuplicateAttributes

- some_name
  - Type: string
- some_name
  - Type: string

### NoType

- some_name
  - DType: string
