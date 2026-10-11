---
prefix: "tst"
repo: "https://www.github.com/my/repo/"
---

### Agent (foaf:Agent)

An object whose term uses a well-known prefix that is never declared in the
frontmatter. Every prefix below must still be declared in the RDF output.

- name
  - Type: string
  - Term: schema:name
- created
  - Type: date
  - Term: dcterms:created
- label
  - Type: string
  - Term: skos:prefLabel
- derived
  - Type: string
  - Term: prov:wasDerivedFrom
- quantity
  - Type: float
  - Term: qudt:hasUnit
- entity
  - Type: string
  - Term: sio:SIO_000008
- relation
  - Type: string
  - Term: obo:RO_0000056
- knows
  - Type: string
  - Term: foaf:knows
