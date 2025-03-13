
# Semantics

MD-Models supports a variety of semantic annotations to help you add meaning to your data model. Most commonly, you want to annotate objects and properties with a semantic type to allow for better interoperability and discoverability. For this, ontologies are used:

## Ontologies

Ontologies are a way to add semantic meaning to your data model. They are a collection of concepts and relationships between them and are specific to the domain of your data model. For instance, the [schema.org](https://schema.org/) ontology is a collection of concepts and relationships that span across many domains. This is very useful when you want to connect to other data models that employ similar concepts, but use different names for them.

Typically these relations are defined as triples, consisting of a subject, predicate and object. For instance, the statement *"John is a person"* can be represented as the triple `(John, is a, person)`. The first element of the triple is the subject, the second is the predicate and the third is the object.

With MD-Models, you can define the `is a` predicate as an object annotation for an object definition. On the other hand, you can define the predicate as a property annotation for a property definition.

## How to annotate objects

Objects are annotated at the level 3 heading of the object definition. The annotation is followed by a whitespace and enclosed in parentheses. Typically, these annotations are expressed in the form of a URI, which points to a definition of the concept in the ontology. But this is a verbose way and can be simplified by using a prefix. We will be using the `schema` prefix in the following examples. More on how to use prefixes can be found in the [preambles section](./preambles.md).

We want to express - *"A `Product` is a `schema:Product`"*.

```markdown
### Product (schema:Product)

- name
  - type: string
```

## How to annotate properties

Properties are annotated using an option, as defined in the [Property Options](./property-options.md) section. We utilize the keyword `term` to add a semantic type to the property. Properties can function in one of two ways:

1. If the type of the property is a primitive type, the `term` option describes an `is a` relationship and thus the *object* in the sense of the triple.
2. If the type of the property is an object or an array of objects, the `term` option describes the relationship (predicate) between the subject (object) and the object (type).

### Object-valued properties

We want to express - *"A `Product` is ordered by a `Person`"*.

```markdown
### Product

- orders
  - type: Person[]
  - term: schema:orderedBy
```

The annotation effectively describes the relationship between the `orders` property and the `Person` type. Given that a `Person` is also annotated with a term, one can then build a Knowledge Graph that connects the `orders` property to the `Person` type in a semantically rich way, which can be used for a variety of purposes, such as semantic search and discovery.

### Primitive-valued properties

We want to express - *"The `name` of a `Product` is a `schema:name`"*.

```markdown
### Product

- name
  - type: string
  - term: schema:name
```

> Naturally, since the `name` property is part of the `Product` object, it builds the relationship "A `Product` has a `name`". In terms of triples, this is represented as `(Product, has, name)`.

Once these annotations are defined, they are automatically added to the generated code and schemes, if supported. Semantic annotations are currently supported in the following language templates:

- `python-dataclass` (JSON-LD)
- `python-pydantic` (JSON-LD)
- `typescript` (JSON-LD)
- `shacl` (Shapes Constraint Language)
- `shex` (Shape Expressions)
