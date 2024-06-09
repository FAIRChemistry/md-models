---
hide:
    - navigation
---
# Model Reference

This page provides comprehensive information about the structure and components of the data model, including detailed descriptions of the types and their properties, information on enumerations, and an overview of the ontologies used and their associated prefixes. Below, you will find a graph that visually represents the overall structure of the data model.

??? quote "Graph"
    ``` mermaid
    flowchart TB
        test(Test)
        test2(Test2)
        ontology(Ontology)
        test(Test) --> test2(Test2)
        test(Test) --> ontology(Ontology)

        click test "#test" "Go to Test"
        click test2 "#test2" "Go to Test2"
        click ontology "#ontology" "Go to Ontology"
    ```


## Ontologies
- [schema](http://schema.org/)


## Types


### Test


__name__* `string`

- The name of the test.


__number__ `float`


- `Default`: 1.0

__test2__ [`list[Test2]`](#test2)


__ontology__ [`Ontology`](#ontology)


------

### Test2


__names__ `list[string]`


__number__ `float`

- `Minimum`: 0


## Enumerations

### Ontology

| Alias | Value |
|-------|-------|
| `ECO` | https://www.evidenceontology.org/term/ |
| `GO` | https://amigo.geneontology.org/amigo/term/ |
| `SIO` | http://semanticscience.org/resource/ |