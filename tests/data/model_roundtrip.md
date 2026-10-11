---
prefix: "tst"
repo: "https://www.github.com/my/repo/"
prefixes:
  schema: http://schema.org/
---

# My Test Model

### Measurement

A measurement record.

- **name**
  - Type: string
  - Term: schema:name
  - Description: The name of the measurement
- value
  - Type: string, float, boolean
  - Description: A multi-typed value
- score
  - Type: float
  - Minimum: 0.5
  - Maximum: 99.9
- labels
  - Type: string[]
- related
  - Type: [Measurement](#measurement)

### Other

- note
  - Type: string
