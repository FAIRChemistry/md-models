---
repo: "https://example.org/jsonld/"
prefix: "ex"
prefixes:
  schema: "https://schema.org/"
  rdfs: "http://www.w3.org/2000/01/rdf-schema#"
  CHEBI: "http://purl.obolibrary.org/obo/CHEBI_"
---

### Document

- **id**
  - Type: Identifier
  - Term: schema:identifier
  - Description: Local key of the document.
- **version**
  - Type: string
  - Pattern: "^(\\d+)\\.\\d+$"
  - Default: "2.0"
  - Description: Version string; the trailing zero is significant.
- label
  - Type: string
  - Term: rdfs:label
- homepage
  - Type: IRI
  - Term: schema:url
- molecules
  - Type: Molecule[]
- created
  - Type: date
- kind
  - Type: Kind

### Molecule

- **id**
  - Type: Identifier
- **name**
  - Type: string
  - Term: schema:name
- vessel_id
  - Type: Identifier
- references
  - Type: IRI[]
  - Term: schema:citation
- values
  - Type: float[]

## Enumerations

### Kind

```
ASSAY = "assay"
SIMULATION = "simulation"
```
