---
prefix: "tst"
repo: "https://www.github.com/my/repo/"
prefixes:
  schema: http://schema.org/
---

### Experiment

An object that references itself, both directly and through an array.

- name
  - Type: string
  - Term: schema:name
- parent
  - Type: [Experiment](#experiment)
  - Term: schema:parent
- children
  - Type: [Experiment](#experiment)[]
  - Term: schema:child
- conditions
  - Type: [Condition](#condition)[]

### Condition

- label
  - Type: string
- experiment
  - Type: [Experiment](#experiment)
