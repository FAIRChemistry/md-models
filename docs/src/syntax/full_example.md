# Full example

The following is a full example of an MD-Models files that defines a data model for a research publication.

```md
---
id: research-publication
prefix: rpub
prefixes:
  - schema: https://schema.org/
---

### ResearchPublication (schema:Publication)

This model represents a scientific publication with its core metadata, authors, 
and citations.

- __doi__
  - Type: Identifier
  - Term: schema:identifier
  - Description: Digital Object Identifier for the publication
  - XML: @doi
- title
  - Type: string
  - Term: schema:name
  - Description: The main title of the publication
- authors
  - Type: [Author](#author)[]
  - Term: schema:authored
  - Description: List of authors who contributed to the publication
- publication_year
  - Type: integer
  - Term: schema:datePublished
  - Description: Year when the publication was published
  - Minimum: 1900
  - Maximum: 2100
- citations
  - Type: integer
  - Term: schema:citation
  - Description: Number of times this publication has been cited
  - Default: 0


### Author (schema:Person)

The `Author` object is a simple object that has a name and an email address.

- __name__
  - Type: string
  - Term: schema:name
  - Description: The name of the author
- __email__
  - Type: string
  - Term: schema:email
  - Description: The email address of the author
```