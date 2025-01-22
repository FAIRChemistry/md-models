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

- __empty_attribute__
  - Type: string
  - Term: schema:hello
  - Description: The name of the test.
  - XML: @
- __empty_element__
  - Type: string
  - Term: schema:hello
  - Description: The name of the test.
  - XML: 
- __special_character_element__
  - Type: string
  - Term: schema:hello
  - Description: The name of the test.
  - XML: schema:hello
- __special_character_attribute__
  - Type: string
  - Term: schema:hello
  - Description: The name of the test.
  - XML: @schema:hello
- __multiple_types__
  - Type: string, float
  - Term: schema:hello
  - Description: The name of the test.
  - XML: fine, bad:
