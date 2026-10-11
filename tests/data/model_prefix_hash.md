---
prefix: "tst"
repo: "https://www.github.com/my/repo/"
prefixes:
  qudt: http://qudt.org/schema/qudt#
  xsd: http://www.w3.org/2001/XMLSchema#
  rdfs: http://www.w3.org/2000/01/rdf-schema#
  owl: http://www.w3.org/2002/07/owl#
  sh: http://www.w3.org/ns/shacl#
  slashed: http://example.org/slashed
---

### Test

Hash namespaces are already delimited and must not have a further `/` appended.
Prefixes the RDF templates declare inline must not be declared a second time.

- name
  - Type: string
  - Term: qudt:hasUnit
