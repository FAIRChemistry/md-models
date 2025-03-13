
# Semantics

MD-Models supports a variety of semantic annotations to help you add meaning to your data model. Most commonly, you want to annotate objects and properties with a semantic type to allow for better interoperability and discoverability. For this, ontologies are used:

## Ontologies

Ontologies are a way to add semantic meaning to your data model. They are a collection of concepts and relationships between them and are specific to the domain of your data model. For instance, the [schema.org](https://schema.org/) ontology is a collection of concepts and relationships that span across many domains. This is very useful when you want to connect to other data models that employ similar concepts, but use different names for them.

Typically these relations are defined as triples, consisting of a subject, predicate and object. For instance, the statement "John is a person" can be represented as the triple `(John, is a, person)`. The first element of the triple is the subject, the second is the predicate and the third is the object. With our properties and objects, we already have a subject and a predicate, we just need to add the object.

## How to annotate objects

Objects are annotated at the level 3 heading of the object definition. The annotation is followed by a whitespace and enclosed in parentheses. Typically, these annotations are expressed in the form of a URI, which points to a definition of the concept in the ontology. But this is a verbose way and can be simplified by using a prefix. We will be using the `schema` prefix in the following examples. More on how to use prefixes can be found in the [preambles section](./preambles.md).

```markdown
### Product (schema:Product)

- name
  - type: string
```

## How to annotate properties

Properties are annotated using an option, as defined in the [Property Options](./property-options.md) section. We utilize the keyword `term` to add a semantic type to the property.

```markdown
### Product

- name
  - type: string
  - term: schema:name
```

Once these annotations are added, they are automatically added to the generated code and schemes, if supported. Semantic annotations are currently supported in the following language templates:

- `python-dataclass` (JSON-LD)
- `python-pydantic` (JSON-LD)
- `typescript` (JSON-LD)
- `shacl` (Shapes Constraint Language)
- `shex` (Shape Expressions)
