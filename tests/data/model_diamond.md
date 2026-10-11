---
prefix: "tst"
repo: "https://www.github.com/my/repo/"
---

### A

Two independent paths reach D. This is a diamond, not a cycle.

- b
  - Type: [B](#b)
- c
  - Type: [C](#c)

### B

- d
  - Type: [D](#d)

### C

- d
  - Type: [D](#d)

### D

- name
  - Type: string
