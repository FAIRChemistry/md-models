---
hide:
  - navigation
---

# MD-Models

![Build Status](https://github.com/JR-1991/sdrdm.rs/actions/workflows/test.yml/badge.svg)

Markdown models are a way to define data models in a human-readable format. The models can be used to generate code, documentation and other formats from a single source of truth.

## Features

- **Human-readable** - Markdown models are easy to read and write for humans.
- **Extendable** - Extension is simple, diffable and mergable with other models.
- **Machine-readable** - Convert your model into other schema languages such as JSON Schema, XSD and more.
- **Code generation** - Generate code in different programming languages such as Python and Typescript.
- **Documentation** - Generate documentation pages for [mkdocs]().
- **Validation** - Validate data against the defined schema.
- **Semantic** - Define semantic relationships between data structures.

## Motivation

There exist many ways to formalize real-world structures into data schemes and make them machine-readable and thus applicable in software solutions. However, existing schema languages such [JSON Schema]() or [XML Schema]() are often hard to read and write for humans, especially for non-developers. Markdown models aim to provide a human-readable way to define data models and invite users from different backgrounds to contribute to the definition of data models.

> Wait, not another schema language!

We are aware that there are many schema languages out there, and we do not aim to replace them. Instead, we aim to provide a way to define data models in a human-readable format that can be used to convert into other schema languages and programming languages. Our goal is to provide a gateway for non-developers to contribute to the definition of data models and to make the process of defining data models more accessible to a broader audience, while ensuring the integrity into existing solutions.

## How it works

Markdown models are defined in a simple markdown format. The format is based on the [CommonMark](https://commonmark.org) specification and can be extended with custom syntax. The concept is simple: Level 3 headings initialize a new type and the following list items define the fields of the type.

=== "Person.md"
    ```markdown 
    ### Person

    This is a simple data model that describes a person. You can also add
    images, links and other markdown elements to your documentation.
    Feel free to be creative!

    - name
      - Type: string
      - Description: Name of the person
    - age
      - Type: integer
      - Description: Age of the person
    ```

=== "Person.json"
    ```json
    {
      "$schema": "http://json-schema.org/draft-07/schema#",
      "title": "Person",
      "description": "This is a simple data model that describes a person [...]",
      "type": "object",
      "properties": {
        "name": {
          "type": "string",
          "description": "Name of the person"
        },
        "age": {
          "type": "integer",
          "description": "Age of the person"
        }
      },
      "required": ["name", "age"]
    }
    ```
=== "Person.xsd"
    ```xml
    <?xml version="1.0" encoding="UTF-8"?>
    <xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
      <xs:element name="person">
        <xs:annotation>
          <xs:documentation>
            This is a simple data model that describes a person [...]
          </xs:documentation>
        </xs:annotation>
        <xs:complexType>
          <xs:sequence>
            <xs:element name="name" type="xs:string">
              <xs:annotation>
                <xs:documentation>Name of the person</xs:documentation>
              </xs:annotation>
            </xs:element>
            <xs:element name="age" type="xs:integer">
              <xs:annotation>
                <xs:documentation>Age of the person</xs:documentation>
              </xs:annotation>
            </xs:element>
          </xs:sequence>
        </xs:complexType>
      </xs:element>
    </xs:schema>
    ```

All three formats describe the same data model, but the markdown version offers more readability compared to the JSON and XML versions. In fact, you can even add text/image documentation to any part of your data model and thus communicate the purpose of the data model to other users. The MD-Models library will recognize non-structural elements that are not part of the data model and will not include them in the generated outputs.

!!! note ""
Want to learn more? Check out the [syntax documentation](/docs/syntax/index.md)!

## How to use

The MD-Models library is available as a Rust library/binary and can be used to convert and validate markdown data models. We currently support the following templates:

- [JSON Schema](https://json-schema.org)
- [XML Schema Definition (XSD)](https://www.w3.org/XML/Schema)
- [ShEx](https://shex.io)
- [SHACL](https://www.w3.org/TR/shacl/)
- [Python-Dataclasses](https://docs.python.org/3/library/dataclasses.html)
- [Python-Pydantic](https://docs.pydantic.dev/latest/)
- [Python-Pydantic-XML](https://pydantic-xml.readthedocs.io/en/latest/)
- [Typescript Interfaces](https://www.typescriptlang.org)
- [Markdown Documentation](https://www.mkdocs.org)

We are planning to add more templates in the future. If you have a specific template in mind, feel free to open an issue or a pull request. Alternatively, you can also write your own template and use it with the MD-Models library.

!!! note ""
Want to learn more? Check out the [library documentation](/docs/library/index.md)!
